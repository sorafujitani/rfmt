# frozen_string_literal: true

require 'prism'
require 'json'
require_relative 'prism_node_extractor'

module Rfmt
  # PrismBridge provides the Ruby-side integration with the Prism parser
  # It parses Ruby source code and converts the AST to a JSON format
  # that can be consumed by the Rust formatter
  class PrismBridge
    extend PrismNodeExtractor

    class ParseError < StandardError; end

    # Parse Ruby source code and return serialized AST
    # @param source [String] Ruby source code to parse
    # @return [String] JSON-serialized AST with comments
    # @raise [ParseError] if parsing fails
    def self.parse(source)
      result = Prism.parse(source)

      handle_parse_errors(result) if result.failure?

      serialize_ast_with_comments(result)
    end

    # Parse Ruby source code from a file
    # @param file_path [String] Path to Ruby file
    # @return [String] JSON-serialized AST
    # @raise [ParseError] if parsing fails
    # @raise [Errno::ENOENT] if file doesn't exist
    def self.parse_file(file_path)
      source = File.read(file_path)
      parse(source)
    rescue Errno::ENOENT
      raise ParseError, "File not found: #{file_path}"
    end

    # Handle parsing errors from Prism
    def self.handle_parse_errors(result)
      errors = result.errors.map do |error|
        {
          line: error.location.start_line,
          column: error.location.start_column,
          message: error.message
        }
      end

      error_messages = errors.map do |err|
        "#{err[:line]}:#{err[:column]}: #{err[:message]}"
      end.join("\n")

      raise ParseError, "Parse errors:\n#{error_messages}"
    end

    # Serialize the Prism AST to JSON
    def self.serialize_ast(node)
      JSON.generate(convert_node(node))
    end

    # Serialize the Prism AST with comments to JSON
    def self.serialize_ast_with_comments(result)
      comments = result.comments.map do |comment|
        {
          comment_type: comment.class.name.split('::').last.downcase.gsub('comment', ''),
          location: {
            start_line: comment.location.start_line,
            start_column: comment.location.start_column,
            end_line: comment.location.end_line,
            end_column: comment.location.end_column,
            start_offset: comment.location.start_offset,
            end_offset: comment.location.end_offset
          },
          text: comment.location.slice,
          position: 'leading' # Default position, will be refined by Rust
        }
      end

      JSON.generate({
                      ast: convert_node(result.value),
                      comments: comments
                    })
    end

    # Convert a Prism node to our internal representation
    def self.convert_node(node)
      return nil if node.nil?

      {
        node_type: node_type_name(node),
        location: extract_location(node),
        children: extract_children(node),
        metadata: extract_metadata(node),
        comments: extract_comments(node),
        formatting: extract_formatting(node)
      }
    end

    # Get the node type name from Prism node
    def self.node_type_name(node)
      # Prism node class names are like "Prism::ProgramNode"
      # We want just "program_node" in snake_case
      node.class.name.split('::').last.gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2')
          .gsub(/([a-z\d])([A-Z])/, '\1_\2').downcase
    end

    # Extract location information from node
    def self.extract_location(node)
      loc = node.location
      {
        start_line: loc.start_line,
        start_column: loc.start_column,
        end_line: loc.end_line,
        end_column: loc.end_column,
        start_offset: loc.start_offset,
        end_offset: loc.end_offset
      }
    end

    # Extract child nodes
    def self.extract_children(node)
      children = []

      begin
        # Different node types have different child accessors
        children = case node
                   when Prism::ProgramNode
                     node.statements ? node.statements.body : []
                   when Prism::StatementsNode
                     node.body || []
                   when Prism::ClassNode
                     [
                       node.constant_path,
                       node.superclass,
                       node.body
                     ].compact
                   when Prism::ModuleNode
                     [
                       node.constant_path,
                       node.body
                     ].compact
                   when Prism::DefNode
                     params = if node.parameters
                                node.parameters.child_nodes.compact
                              else
                                []
                              end
                     params + [node.body].compact
                   when Prism::CallNode
                     result = []
                     result << node.receiver if node.receiver
                     result.concat(node.arguments.child_nodes.compact) if node.arguments
                     result << node.block if node.block
                     result
                   when Prism::IfNode, Prism::UnlessNode
                     [
                       node.predicate,
                       node.statements,
                       node.consequent
                     ].compact
                   when Prism::ElseNode
                     [node.statements].compact
                   when Prism::ArrayNode
                     node.elements || []
                   when Prism::HashNode
                     node.elements || []
                   when Prism::BlockNode
                     params = if node.parameters
                                node.parameters.child_nodes.compact
                              else
                                []
                              end
                     params + [node.body].compact
                   when Prism::BeginNode
                     [
                       node.statements,
                       node.rescue_clause,
                       node.ensure_clause
                     ].compact
                   when Prism::EnsureNode
                     [node.statements].compact
                   when Prism::LambdaNode
                     params = if node.parameters
                                node.parameters.child_nodes.compact
                              else
                                []
                              end
                     params + [node.body].compact
                   when Prism::RescueNode
                     result = []
                     result.concat(node.exceptions) if node.exceptions
                     result << node.reference if node.reference
                     result << node.statements if node.statements
                     result << node.subsequent if node.subsequent
                     result
                   when Prism::SymbolNode, Prism::LocalVariableReadNode, Prism::InstanceVariableReadNode
                     []
                   when Prism::LocalVariableWriteNode, Prism::InstanceVariableWriteNode
                     [node.value].compact
                   when Prism::ReturnNode
                     node.arguments ? node.arguments.child_nodes.compact : []
                   when Prism::OrNode
                     [node.left, node.right].compact
                   when Prism::AssocNode
                     [node.key, node.value].compact
                   when Prism::KeywordHashNode
                     node.elements || []
                   when Prism::InterpolatedStringNode
                     node.parts || []
                   when Prism::EmbeddedStatementsNode
                     [node.statements].compact
                   when Prism::CaseNode
                     [node.predicate, *node.conditions, node.else_clause].compact
                   when Prism::WhenNode
                     [*node.conditions, node.statements].compact
                   when Prism::WhileNode, Prism::UntilNode
                     [node.predicate, node.statements].compact
                   when Prism::ForNode
                     [node.index, node.collection, node.statements].compact
                   when Prism::BreakNode, Prism::NextNode
                     node.arguments ? node.arguments.child_nodes.compact : []
                   when Prism::RedoNode, Prism::RetryNode
                     []
                   when Prism::YieldNode
                     node.arguments ? node.arguments.child_nodes.compact : []
                   when Prism::SuperNode
                     result = []
                     result.concat(node.arguments.child_nodes.compact) if node.arguments
                     result << node.block if node.block
                     result
                   when Prism::ForwardingSuperNode
                     node.block ? [node.block] : []
                   when Prism::RescueModifierNode
                     [node.expression, node.rescue_expression].compact
                   when Prism::RangeNode
                     [node.left, node.right].compact
                   when Prism::RegularExpressionNode
                     []
                   when Prism::SplatNode
                     [node.expression].compact
                   when Prism::AndNode
                     [node.left, node.right].compact
                   when Prism::NotNode
                     [node.expression].compact
                   when Prism::InterpolatedRegularExpressionNode, Prism::InterpolatedSymbolNode,
                        Prism::InterpolatedXStringNode
                     node.parts || []
                   when Prism::XStringNode
                     []
                   when Prism::ClassVariableReadNode, Prism::GlobalVariableReadNode, Prism::SelfNode
                     []
                   when Prism::ClassVariableWriteNode, Prism::GlobalVariableWriteNode
                     [node.value].compact
                   when Prism::ClassVariableOrWriteNode, Prism::ClassVariableAndWriteNode,
                        Prism::GlobalVariableOrWriteNode, Prism::GlobalVariableAndWriteNode,
                        Prism::LocalVariableOrWriteNode, Prism::LocalVariableAndWriteNode,
                        Prism::InstanceVariableOrWriteNode, Prism::InstanceVariableAndWriteNode,
                        Prism::ConstantOrWriteNode, Prism::ConstantAndWriteNode
                     [node.value].compact
                   when Prism::ClassVariableOperatorWriteNode, Prism::GlobalVariableOperatorWriteNode,
                        Prism::LocalVariableOperatorWriteNode, Prism::InstanceVariableOperatorWriteNode,
                        Prism::ConstantOperatorWriteNode
                     [node.value].compact
                   when Prism::ConstantPathOrWriteNode, Prism::ConstantPathAndWriteNode,
                        Prism::ConstantPathOperatorWriteNode
                     [node.target, node.value].compact
                   when Prism::ConstantPathWriteNode
                     [node.target, node.value].compact
                   when Prism::CaseMatchNode
                     [node.predicate, *node.conditions, node.else_clause].compact
                   when Prism::InNode
                     [node.pattern, node.statements].compact
                   when Prism::MatchPredicateNode, Prism::MatchRequiredNode
                     [node.value, node.pattern].compact
                   when Prism::ParenthesesNode
                     [node.body].compact
                   when Prism::DefinedNode
                     [node.value].compact
                   when Prism::SingletonClassNode
                     [node.expression, node.body].compact
                   when Prism::AliasMethodNode
                     [node.new_name, node.old_name].compact
                   when Prism::AliasGlobalVariableNode
                     [node.new_name, node.old_name].compact
                   when Prism::UndefNode
                     node.names || []
                   when Prism::AssocSplatNode
                     [node.value].compact
                   when Prism::BlockArgumentNode
                     [node.expression].compact
                   when Prism::MultiWriteNode
                     [*node.lefts, node.rest, *node.rights, node.value].compact
                   when Prism::MultiTargetNode
                     [*node.lefts, node.rest, *node.rights].compact
                   else
                     # For unknown types, try to get child nodes if they exist
                     []
                   end
      rescue StandardError => e
        # Log warning in debug mode but continue processing
        warn "Warning: Failed to extract children from #{node.class}: #{e.message}" if $DEBUG
        children = []
      end

      children.compact.map { |child| convert_node(child) }
    end

    # Extract metadata specific to node type
    def self.extract_metadata(node)
      metadata = {}

      case node
      when Prism::ClassNode
        if (name = extract_class_or_module_name(node))
          metadata['name'] = name
        end
        if (superclass = extract_superclass_name(node))
          metadata['superclass'] = superclass
        end
      when Prism::ModuleNode
        if (name = extract_class_or_module_name(node))
          metadata['name'] = name
        end
      when Prism::DefNode
        if (name = extract_node_name(node))
          metadata['name'] = name
        end
        metadata['parameters_count'] = extract_parameter_count(node).to_s
        # Check if this is a class method (def self.method_name)
        if node.respond_to?(:receiver) && node.receiver
          receiver = node.receiver
          if receiver.is_a?(Prism::SelfNode)
            metadata['receiver'] = 'self'
          elsif receiver.respond_to?(:slice)
            metadata['receiver'] = receiver.slice
          end
        end
      when Prism::CallNode
        if (name = extract_node_name(node))
          metadata['name'] = name
        end
        if (message = extract_message_name(node))
          metadata['message'] = message
        end
      when Prism::StringNode
        if (content = extract_string_content(node))
          metadata['content'] = content
        end
      when Prism::IntegerNode
        if (value = extract_literal_value(node))
          metadata['value'] = value
        end
      when Prism::FloatNode
        if (value = extract_literal_value(node))
          metadata['value'] = value
        end
      when Prism::SymbolNode
        if (value = extract_literal_value(node))
          metadata['value'] = value
        end
      when Prism::IfNode, Prism::UnlessNode
        # Detect ternary operator: if_keyword_loc is nil for ternary
        metadata['is_ternary'] = node.if_keyword_loc.nil?.to_s if node.respond_to?(:if_keyword_loc)
      end

      metadata
    end

    # Extract comments associated with the node
    def self.extract_comments(_node)
      # Prism attaches comments to the parse result, not individual nodes
      # For Phase 1, we'll return empty array and implement in Phase 2
      []
    end

    # Extract formatting information
    def self.extract_formatting(node)
      loc = node.location
      {
        indent_level: 0, # Will be calculated during formatting
        needs_blank_line_before: false,
        needs_blank_line_after: false,
        preserve_newlines: false,
        multiline: loc.start_line != loc.end_line,
        original_formatting: nil # Can store original text if needed
      }
    end
  end
end

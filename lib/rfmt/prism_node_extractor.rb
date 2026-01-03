# frozen_string_literal: true

module Rfmt
  # PrismNodeExtractor provides safe methods to extract information from Prism nodes
  # This module encapsulates the logic for accessing Prism node properties,
  # making the code resilient to Prism API changes
  module PrismNodeExtractor
    # Extract the name from a node
    # @param node [Prism::Node] The node to extract name from
    # @return [String, nil] The node name or nil if not available
    def extract_node_name(node)
      return nil unless node.respond_to?(:name)

      node.name.to_s
    end

    # Extract full name from class or module node (handles namespaced names like Foo::Bar::Baz)
    # @param node [Prism::ClassNode, Prism::ModuleNode] The class or module node
    # @return [String, nil] The full name or nil if not available
    def extract_class_or_module_name(node)
      return nil unless node.respond_to?(:constant_path)

      cp = node.constant_path
      return node.name.to_s if cp.nil?

      case cp
      when Prism::ConstantReadNode
        cp.name.to_s
      when Prism::ConstantPathNode
        if cp.respond_to?(:full_name)
          cp.full_name.to_s
        elsif cp.respond_to?(:slice)
          cp.slice
        else
          cp.location.slice
        end
      else
        node.name.to_s
      end
    end

    # Extract superclass name from a class node
    # @param class_node [Prism::ClassNode] The class node
    # @return [String, nil] The superclass name or nil if not available
    def extract_superclass_name(class_node)
      return nil unless class_node.respond_to?(:superclass)

      sc = class_node.superclass
      return nil if sc.nil?

      case sc
      when Prism::ConstantReadNode
        sc.name.to_s
      when Prism::ConstantPathNode
        # Try full_name first, fall back to slice for original source
        if sc.respond_to?(:full_name)
          sc.full_name.to_s
        elsif sc.respond_to?(:slice)
          sc.slice
        else
          sc.location.slice
        end
      when Prism::CallNode
        # Handle cases like ActiveRecord::Migration[8.1]
        # Use slice to get the original source text
        sc.slice
      else
        # Fallback: try to get original source text
        if sc.respond_to?(:slice)
          sc.slice
        else
          sc.location.slice
        end
      end
    end

    # Extract parameter count from a method definition node
    # @param def_node [Prism::DefNode] The method definition node
    # @return [Integer] The number of parameters (0 if none)
    def extract_parameter_count(def_node)
      return 0 unless def_node.respond_to?(:parameters)
      return 0 if def_node.parameters.nil?
      return 0 unless def_node.parameters.respond_to?(:child_nodes)

      def_node.parameters.child_nodes.compact.length
    end

    # Extract message name from a call node
    # @param call_node [Prism::CallNode] The call node
    # @return [String, nil] The message name or nil if not available
    def extract_message_name(call_node)
      return nil unless call_node.respond_to?(:message)

      call_node.message.to_s
    end

    # Extract content from a string node
    # @param string_node [Prism::StringNode] The string node
    # @return [String, nil] The string content or nil if not available
    def extract_string_content(string_node)
      return nil unless string_node.respond_to?(:content)

      string_node.content
    end

    # Extract value from a literal node (Integer, Float, Symbol)
    # @param node [Prism::Node] The literal node
    # @return [String, nil] The value as string or nil if not available
    def extract_literal_value(node)
      return nil unless node.respond_to?(:value)

      node.value.to_s
    end
  end
end

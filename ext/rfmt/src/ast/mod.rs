use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal AST representation
/// This structure is designed to work seamlessly with Prism parser output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_type: NodeType,
    pub location: Location,
    pub children: Vec<Node>,
    pub metadata: HashMap<String, String>,
    pub comments: Vec<Comment>,
    pub formatting: FormattingInfo,
}

/// Location information for a node in the source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Node types supported by rfmt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // Program & Statements
    ProgramNode,
    StatementsNode,

    // Definitions
    ClassNode,
    ModuleNode,
    DefNode,

    // Expressions
    CallNode,
    IfNode,
    ElseNode,
    UnlessNode,

    // Exception handling
    BeginNode,
    RescueNode,

    // Literals
    StringNode,
    IntegerNode,
    FloatNode,
    ArrayNode,
    HashNode,
    TrueNode,
    FalseNode,
    NilNode,
    SymbolNode,

    // Blocks
    BlockNode,

    // Case/When
    CaseNode,
    WhenNode,

    // HashNode
    AssocNode,
    KeywordHashNode,

    // Variables
    LocalVariableReadNode,
    LocalVariableWriteNode,
    InstanceVariableReadNode,
    InstanceVariableWriteNode,

    // Lambda
    LambdaNode,

    // Control flow
    ReturnNode,
    EnsureNode,

    // Strings
    InterpolatedStringNode,
    EmbeddedStatementsNode,

    // Logical
    OrNode,
    AndNode,
    NotNode,

    // Loop constructs
    WhileNode,
    UntilNode,
    ForNode,

    // Control flow
    BreakNode,
    NextNode,
    RedoNode,
    RetryNode,
    YieldNode,
    SuperNode,
    ForwardingSuperNode,
    RescueModifierNode,

    // Ranges
    RangeNode,

    // Other literals
    RegularExpressionNode,
    SplatNode,
    InterpolatedRegularExpressionNode,
    InterpolatedSymbolNode,
    XStringNode,
    InterpolatedXStringNode,

    // Class variables
    ClassVariableReadNode,
    ClassVariableWriteNode,
    ClassVariableOrWriteNode,
    ClassVariableAndWriteNode,
    ClassVariableOperatorWriteNode,

    // Global variables
    GlobalVariableReadNode,
    GlobalVariableWriteNode,
    GlobalVariableOrWriteNode,
    GlobalVariableAndWriteNode,
    GlobalVariableOperatorWriteNode,

    // Compound assignment
    LocalVariableOrWriteNode,
    LocalVariableAndWriteNode,
    LocalVariableOperatorWriteNode,
    InstanceVariableOrWriteNode,
    InstanceVariableAndWriteNode,
    InstanceVariableOperatorWriteNode,
    ConstantOrWriteNode,
    ConstantAndWriteNode,
    ConstantOperatorWriteNode,
    ConstantPathOrWriteNode,
    ConstantPathAndWriteNode,
    ConstantPathOperatorWriteNode,
    ConstantPathWriteNode,

    // Pattern matching
    CaseMatchNode,
    InNode,
    MatchPredicateNode,
    MatchRequiredNode,

    // Other common nodes
    SelfNode,
    ParenthesesNode,
    DefinedNode,
    SingletonClassNode,
    AliasMethodNode,
    AliasGlobalVariableNode,
    UndefNode,
    AssocSplatNode,
    BlockArgumentNode,
    MultiWriteNode,
    MultiTargetNode,

    // Constants (structural nodes, part of definitions)
    ConstantReadNode,
    ConstantWriteNode,
    ConstantPathNode,

    // Parameters (structural nodes, part of method definitions)
    RequiredParameterNode,
    OptionalParameterNode,
    RestParameterNode,
    KeywordParameterNode,
    RequiredKeywordParameterNode,
    OptionalKeywordParameterNode,
    KeywordRestParameterNode,
    BlockParameterNode,

    // Source metadata nodes
    SourceFileNode,
    SourceLineNode,
    SourceEncodingNode,

    // Pre/Post execution
    PreExecutionNode,
    PostExecutionNode,

    // Numeric literals
    RationalNode,
    ImaginaryNode,

    // String interpolation
    EmbeddedVariableNode,

    // Pattern matching patterns
    ArrayPatternNode,
    HashPatternNode,
    FindPatternNode,
    CapturePatternNode,
    AlternationPatternNode,
    PinnedExpressionNode,
    PinnedVariableNode,

    // Forwarding
    ForwardingArgumentsNode,
    ForwardingParameterNode,
    NoKeywordsParameterNode,

    // References
    BackReferenceReadNode,
    NumberedReferenceReadNode,

    // Call/Index compound assignment
    CallAndWriteNode,
    CallOrWriteNode,
    CallOperatorWriteNode,
    IndexAndWriteNode,
    IndexOrWriteNode,
    IndexOperatorWriteNode,

    // Match
    MatchWriteNode,
    MatchLastLineNode,
    InterpolatedMatchLastLineNode,

    // Other
    FlipFlopNode,
    ImplicitNode,
    ImplicitRestNode,

    Unknown(String),
}

impl NodeType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "program_node" => Self::ProgramNode,
            "statements_node" => Self::StatementsNode,
            "class_node" => Self::ClassNode,
            "module_node" => Self::ModuleNode,
            "def_node" => Self::DefNode,
            "call_node" => Self::CallNode,
            "if_node" => Self::IfNode,
            "else_node" => Self::ElseNode,
            "unless_node" => Self::UnlessNode,
            "begin_node" => Self::BeginNode,
            "rescue_node" => Self::RescueNode,
            "string_node" => Self::StringNode,
            "integer_node" => Self::IntegerNode,
            "float_node" => Self::FloatNode,
            "array_node" => Self::ArrayNode,
            "hash_node" => Self::HashNode,
            "true_node" => Self::TrueNode,
            "false_node" => Self::FalseNode,
            "nil_node" => Self::NilNode,
            "block_node" => Self::BlockNode,
            "constant_read_node" => Self::ConstantReadNode,
            "constant_write_node" => Self::ConstantWriteNode,
            "constant_path_node" => Self::ConstantPathNode,
            "required_parameter_node" => Self::RequiredParameterNode,
            "optional_parameter_node" => Self::OptionalParameterNode,
            "rest_parameter_node" => Self::RestParameterNode,
            "keyword_parameter_node" => Self::KeywordParameterNode,
            "required_keyword_parameter_node" => Self::RequiredKeywordParameterNode,
            "optional_keyword_parameter_node" => Self::OptionalKeywordParameterNode,
            "keyword_rest_parameter_node" => Self::KeywordRestParameterNode,
            "block_parameter_node" => Self::BlockParameterNode,
            "symbol_node" => Self::SymbolNode,
            "case_node" => Self::CaseNode,
            "when_node" => Self::WhenNode,
            "assoc_node" => Self::AssocNode,
            "keyword_hash_node" => Self::KeywordHashNode,
            "local_variable_read_node" => Self::LocalVariableReadNode,
            "local_variable_write_node" => Self::LocalVariableWriteNode,
            "instance_variable_read_node" => Self::InstanceVariableReadNode,
            "instance_variable_write_node" => Self::InstanceVariableWriteNode,
            "lambda_node" => Self::LambdaNode,
            "return_node" => Self::ReturnNode,
            "ensure_node" => Self::EnsureNode,
            "interpolated_string_node" => Self::InterpolatedStringNode,
            "embedded_statements_node" => Self::EmbeddedStatementsNode,
            "or_node" => Self::OrNode,
            "and_node" => Self::AndNode,
            "not_node" => Self::NotNode,
            "while_node" => Self::WhileNode,
            "until_node" => Self::UntilNode,
            "for_node" => Self::ForNode,
            "break_node" => Self::BreakNode,
            "next_node" => Self::NextNode,
            "redo_node" => Self::RedoNode,
            "retry_node" => Self::RetryNode,
            "yield_node" => Self::YieldNode,
            "super_node" => Self::SuperNode,
            "forwarding_super_node" => Self::ForwardingSuperNode,
            "rescue_modifier_node" => Self::RescueModifierNode,
            "range_node" => Self::RangeNode,
            "regular_expression_node" => Self::RegularExpressionNode,
            "splat_node" => Self::SplatNode,
            "interpolated_regular_expression_node" => Self::InterpolatedRegularExpressionNode,
            "interpolated_symbol_node" => Self::InterpolatedSymbolNode,
            "x_string_node" => Self::XStringNode,
            "interpolated_x_string_node" => Self::InterpolatedXStringNode,
            "class_variable_read_node" => Self::ClassVariableReadNode,
            "class_variable_write_node" => Self::ClassVariableWriteNode,
            "class_variable_or_write_node" => Self::ClassVariableOrWriteNode,
            "class_variable_and_write_node" => Self::ClassVariableAndWriteNode,
            "class_variable_operator_write_node" => Self::ClassVariableOperatorWriteNode,
            "global_variable_read_node" => Self::GlobalVariableReadNode,
            "global_variable_write_node" => Self::GlobalVariableWriteNode,
            "global_variable_or_write_node" => Self::GlobalVariableOrWriteNode,
            "global_variable_and_write_node" => Self::GlobalVariableAndWriteNode,
            "global_variable_operator_write_node" => Self::GlobalVariableOperatorWriteNode,
            "local_variable_or_write_node" => Self::LocalVariableOrWriteNode,
            "local_variable_and_write_node" => Self::LocalVariableAndWriteNode,
            "local_variable_operator_write_node" => Self::LocalVariableOperatorWriteNode,
            "instance_variable_or_write_node" => Self::InstanceVariableOrWriteNode,
            "instance_variable_and_write_node" => Self::InstanceVariableAndWriteNode,
            "instance_variable_operator_write_node" => Self::InstanceVariableOperatorWriteNode,
            "constant_or_write_node" => Self::ConstantOrWriteNode,
            "constant_and_write_node" => Self::ConstantAndWriteNode,
            "constant_operator_write_node" => Self::ConstantOperatorWriteNode,
            "constant_path_or_write_node" => Self::ConstantPathOrWriteNode,
            "constant_path_and_write_node" => Self::ConstantPathAndWriteNode,
            "constant_path_operator_write_node" => Self::ConstantPathOperatorWriteNode,
            "constant_path_write_node" => Self::ConstantPathWriteNode,
            "case_match_node" => Self::CaseMatchNode,
            "in_node" => Self::InNode,
            "match_predicate_node" => Self::MatchPredicateNode,
            "match_required_node" => Self::MatchRequiredNode,
            "self_node" => Self::SelfNode,
            "parentheses_node" => Self::ParenthesesNode,
            "defined_node" => Self::DefinedNode,
            "singleton_class_node" => Self::SingletonClassNode,
            "alias_method_node" => Self::AliasMethodNode,
            "alias_global_variable_node" => Self::AliasGlobalVariableNode,
            "undef_node" => Self::UndefNode,
            "assoc_splat_node" => Self::AssocSplatNode,
            "block_argument_node" => Self::BlockArgumentNode,
            "multi_write_node" => Self::MultiWriteNode,
            "multi_target_node" => Self::MultiTargetNode,
            "source_file_node" => Self::SourceFileNode,
            "source_line_node" => Self::SourceLineNode,
            "source_encoding_node" => Self::SourceEncodingNode,
            "pre_execution_node" => Self::PreExecutionNode,
            "post_execution_node" => Self::PostExecutionNode,
            // Numeric literals
            "rational_node" => Self::RationalNode,
            "imaginary_node" => Self::ImaginaryNode,
            // String interpolation
            "embedded_variable_node" => Self::EmbeddedVariableNode,
            // Pattern matching patterns
            "array_pattern_node" => Self::ArrayPatternNode,
            "hash_pattern_node" => Self::HashPatternNode,
            "find_pattern_node" => Self::FindPatternNode,
            "capture_pattern_node" => Self::CapturePatternNode,
            "alternation_pattern_node" => Self::AlternationPatternNode,
            "pinned_expression_node" => Self::PinnedExpressionNode,
            "pinned_variable_node" => Self::PinnedVariableNode,
            // Forwarding
            "forwarding_arguments_node" => Self::ForwardingArgumentsNode,
            "forwarding_parameter_node" => Self::ForwardingParameterNode,
            "no_keywords_parameter_node" => Self::NoKeywordsParameterNode,
            // References
            "back_reference_read_node" => Self::BackReferenceReadNode,
            "numbered_reference_read_node" => Self::NumberedReferenceReadNode,
            // Call/Index compound assignment
            "call_and_write_node" => Self::CallAndWriteNode,
            "call_or_write_node" => Self::CallOrWriteNode,
            "call_operator_write_node" => Self::CallOperatorWriteNode,
            "index_and_write_node" => Self::IndexAndWriteNode,
            "index_or_write_node" => Self::IndexOrWriteNode,
            "index_operator_write_node" => Self::IndexOperatorWriteNode,
            // Match
            "match_write_node" => Self::MatchWriteNode,
            "match_last_line_node" => Self::MatchLastLineNode,
            "interpolated_match_last_line_node" => Self::InterpolatedMatchLastLineNode,
            // Other
            "flip_flop_node" => Self::FlipFlopNode,
            "implicit_node" => Self::ImplicitNode,
            "implicit_rest_node" => Self::ImplicitRestNode,
            _ => Self::Unknown(s.to_string()),
        }
    }

    /// Check if this node type is a definition (class, module, or method)
    #[cfg(test)]
    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            NodeType::ClassNode | NodeType::ModuleNode | NodeType::DefNode
        )
    }
}

/// Comment attached to a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub text: String,
    pub location: Location,
    pub comment_type: CommentType,
    pub position: CommentPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentType {
    Line,  // # comment
    Block, // =begin...=end
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentPosition {
    Leading,  // Comment before the node
    Trailing, // Comment after the node (same line)
    Inner,    // Comment inside the node
}

/// Formatting information attached to a node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormattingInfo {
    pub indent_level: usize,
    pub needs_blank_line_before: bool,
    pub needs_blank_line_after: bool,
    pub preserve_newlines: bool,
    pub multiline: bool,
    pub original_formatting: Option<String>,
}

impl Node {
    /// Create a new node with the given type and location
    #[cfg(test)]
    pub fn new(node_type: NodeType, location: Location) -> Self {
        Self {
            node_type,
            location,
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    /// Add children to the node
    #[cfg(test)]
    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    /// Add metadata to the node
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add comments to the node
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn with_comments(mut self, comments: Vec<Comment>) -> Self {
        self.comments = comments;
        self
    }

    /// Check if the node spans multiple lines
    #[cfg(test)]
    pub fn is_multiline(&self) -> bool {
        self.location.start_line != self.location.end_line
    }

    /// Get the number of lines this node spans
    #[cfg(test)]
    pub fn line_count(&self) -> usize {
        self.location.end_line - self.location.start_line + 1
    }

    /// Check if this is an unknown node type
    #[cfg(test)]
    pub fn is_unknown(&self) -> bool {
        matches!(self.node_type, NodeType::Unknown(_))
    }

    /// Get the unknown node type name if this is an unknown node
    #[cfg(test)]
    pub fn unknown_type(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Unknown(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl Location {
    /// Create a new location
    pub fn new(
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            start_offset,
            end_offset,
        }
    }

    /// Create a zero location (for testing)
    #[cfg(test)]
    pub fn zero() -> Self {
        Self {
            start_line: 0,
            start_column: 0,
            end_line: 0,
            end_column: 0,
            start_offset: 0,
            end_offset: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(NodeType::ProgramNode, Location::zero());
        assert_eq!(node.node_type, NodeType::ProgramNode);
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_node_with_children() {
        let child = Node::new(NodeType::ClassNode, Location::zero());
        let node = Node::new(NodeType::ProgramNode, Location::zero()).with_children(vec![child]);

        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].node_type, NodeType::ClassNode);
    }

    #[test]
    fn test_node_type_from_str() {
        assert_eq!(NodeType::from_str("program_node"), NodeType::ProgramNode);
        assert_eq!(NodeType::from_str("class_node"), NodeType::ClassNode);
        assert_eq!(NodeType::from_str("def_node"), NodeType::DefNode);

        // Unknown type should return Unknown variant
        match NodeType::from_str("unknown_node") {
            NodeType::Unknown(s) => assert_eq!(s, "unknown_node"),
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_node_type_is_definition() {
        assert!(NodeType::ClassNode.is_definition());
        assert!(NodeType::ModuleNode.is_definition());
        assert!(NodeType::DefNode.is_definition());
        assert!(!NodeType::CallNode.is_definition());
        assert!(!NodeType::IntegerNode.is_definition());
    }

    #[test]
    fn test_location_zero() {
        let loc = Location::zero();
        assert_eq!(loc.start_line, 0);
        assert_eq!(loc.start_offset, 0);
    }

    #[test]
    fn test_node_is_multiline() {
        let single_line = Node::new(NodeType::CallNode, Location::new(1, 0, 1, 10, 0, 10));
        assert!(!single_line.is_multiline());

        let multi_line = Node::new(NodeType::ClassNode, Location::new(1, 0, 5, 3, 0, 50));
        assert!(multi_line.is_multiline());
    }

    #[test]
    fn test_node_line_count() {
        let node = Node::new(NodeType::DefNode, Location::new(10, 0, 15, 3, 100, 200));
        assert_eq!(node.line_count(), 6);
    }

    #[test]
    fn test_node_is_unknown() {
        let known_node = Node::new(NodeType::ClassNode, Location::zero());
        assert!(!known_node.is_unknown());
        assert_eq!(known_node.unknown_type(), None);

        let unknown_node = Node::new(
            NodeType::Unknown("custom_node".to_string()),
            Location::zero(),
        );
        assert!(unknown_node.is_unknown());
        assert_eq!(unknown_node.unknown_type(), Some("custom_node"));
    }

    #[test]
    fn test_node_type_from_str_returns_unknown_for_unsupported() {
        let node_type = NodeType::from_str("unsupported_node_type");
        match node_type {
            NodeType::Unknown(name) => assert_eq!(name, "unsupported_node_type"),
            _ => panic!("Expected Unknown variant"),
        }
    }
}

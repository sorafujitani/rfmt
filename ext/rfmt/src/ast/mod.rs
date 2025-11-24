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
    UnlessNode,

    // Literals
    StringNode,
    IntegerNode,
    FloatNode,
    ArrayNode,
    HashNode,
    TrueNode,
    FalseNode,
    NilNode,

    // Blocks
    BlockNode,

    // Constants (structural nodes, part of definitions)
    ConstantReadNode,
    ConstantWriteNode,
    ConstantPathNode,

    // Parameters (structural nodes, part of method definitions)
    RequiredParameterNode,
    OptionalParameterNode,
    RestParameterNode,
    KeywordParameterNode,
    KeywordRestParameterNode,
    BlockParameterNode,

    Unknown(String),
}

impl NodeType {
    /// Parse node type from Prism type string
    /// Returns Unknown variant for unsupported node types
    pub fn from_str(s: &str) -> Self {
        match s {
            "program_node" => Self::ProgramNode,
            "statements_node" => Self::StatementsNode,
            "class_node" => Self::ClassNode,
            "module_node" => Self::ModuleNode,
            "def_node" => Self::DefNode,
            "call_node" => Self::CallNode,
            "if_node" => Self::IfNode,
            "unless_node" => Self::UnlessNode,
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
            "keyword_rest_parameter_node" => Self::KeywordRestParameterNode,
            "block_parameter_node" => Self::BlockParameterNode,
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
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add comments to the node
    #[cfg(test)]
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

use crate::ast::{Comment, CommentPosition, CommentType, FormattingInfo, Location, Node, NodeType};
use crate::error::{Result, RfmtError};
use crate::parser::RubyParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prism parser adapter
/// This integrates with Ruby Prism parser via Magnus FFI
pub struct PrismAdapter;

impl PrismAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse JSON from Ruby's `PrismBridge`
    fn parse_json(json: &str) -> Result<(PrismNode, Vec<PrismComment>)> {
        // Try to parse as new format with comments first
        if let Ok(wrapper) = serde_json::from_str::<PrismWrapper>(json) {
            return Ok((wrapper.ast, wrapper.comments));
        }

        // Fall back to old format (single node without comments)
        let node: PrismNode = serde_json::from_str(json)
            .map_err(|e| RfmtError::PrismError(format!("Failed to parse Prism JSON: {}", e)))?;
        Ok((node, Vec::new()))
    }

    /// Convert `PrismNode` to internal `Node` representation
    fn convert_node(prism_node: &PrismNode) -> Result<Node> {
        // Convert node type (always succeeds, returns Unknown for unsupported types)
        let node_type = NodeType::from_str(&prism_node.node_type);

        // Convert location
        let location = Location::new(
            prism_node.location.start_line,
            prism_node.location.start_column,
            prism_node.location.end_line,
            prism_node.location.end_column,
            prism_node.location.start_offset,
            prism_node.location.end_offset,
        );

        // Convert children recursively
        let children: Result<Vec<Node>> =
            prism_node.children.iter().map(Self::convert_node).collect();
        let children = children?;

        // Convert comments
        let comments: Vec<Comment> = prism_node
            .comments
            .iter()
            .map(Self::convert_comment)
            .collect();

        // Convert formatting info
        let formatting = FormattingInfo {
            indent_level: prism_node.formatting.indent_level,
            needs_blank_line_before: prism_node.formatting.needs_blank_line_before,
            needs_blank_line_after: prism_node.formatting.needs_blank_line_after,
            preserve_newlines: prism_node.formatting.preserve_newlines,
            multiline: prism_node.formatting.multiline,
            original_formatting: prism_node.formatting.original_formatting.clone(),
        };

        Ok(Node {
            node_type,
            location,
            children,
            metadata: prism_node.metadata.clone(),
            comments,
            formatting,
        })
    }

    /// Convert `PrismComment` to internal `Comment`
    fn convert_comment(comment: &PrismComment) -> Comment {
        Comment {
            text: comment.text.clone(),
            location: Location::new(
                comment.location.start_line,
                comment.location.start_column,
                comment.location.end_line,
                comment.location.end_column,
                comment.location.start_offset,
                comment.location.end_offset,
            ),
            comment_type: comment.comment_type.into(),
            position: comment.position.into(),
        }
    }
}

impl RubyParser for PrismAdapter {
    fn parse(&self, json: &str) -> Result<Node> {
        let (prism_ast, top_level_comments) = Self::parse_json(json)?;
        let mut node = Self::convert_node(&prism_ast)?;

        // Attach top-level comments to the root node
        if !top_level_comments.is_empty() {
            node.comments
                .extend(top_level_comments.iter().map(Self::convert_comment));
        }

        Ok(node)
    }
}

impl Default for PrismAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for JSON containing both AST and comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismWrapper {
    pub ast: PrismNode,
    pub comments: Vec<PrismComment>,
}

/// JSON representation of a Prism node from Ruby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismNode {
    pub node_type: String,
    pub location: PrismLocation,
    pub children: Vec<PrismNode>,
    pub metadata: HashMap<String, String>,
    pub comments: Vec<PrismComment>,
    pub formatting: PrismFormattingInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismLocation {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismComment {
    pub text: String,
    pub location: PrismLocation,
    #[serde(rename = "type", default)]
    pub comment_type: PrismCommentType,
    #[serde(default)]
    pub position: PrismCommentPosition,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrismCommentType {
    #[default]
    Line,
    Block,
}

impl From<PrismCommentType> for CommentType {
    fn from(t: PrismCommentType) -> Self {
        match t {
            PrismCommentType::Line => CommentType::Line,
            PrismCommentType::Block => CommentType::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrismCommentPosition {
    #[default]
    Leading,
    Trailing,
    Inner,
}

impl From<PrismCommentPosition> for CommentPosition {
    fn from(p: PrismCommentPosition) -> Self {
        match p {
            PrismCommentPosition::Leading => CommentPosition::Leading,
            PrismCommentPosition::Trailing => CommentPosition::Trailing,
            PrismCommentPosition::Inner => CommentPosition::Inner,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismFormattingInfo {
    pub indent_level: usize,
    pub needs_blank_line_before: bool,
    pub needs_blank_line_after: bool,
    pub preserve_newlines: bool,
    pub multiline: bool,
    pub original_formatting: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_program() {
        let adapter = PrismAdapter::new();
        let json = r#"{
            "node_type": "program_node",
            "location": {
                "start_line": 1,
                "start_column": 0,
                "end_line": 1,
                "end_column": 12,
                "start_offset": 0,
                "end_offset": 12
            },
            "children": [],
            "metadata": {},
            "comments": [],
            "formatting": {
                "indent_level": 0,
                "needs_blank_line_before": false,
                "needs_blank_line_after": false,
                "preserve_newlines": false,
                "multiline": false,
                "original_formatting": null
            }
        }"#;

        let result = adapter.parse(json);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.node_type, NodeType::ProgramNode);
        assert_eq!(node.location.start_line, 1);
        assert_eq!(node.location.end_line, 1);
    }

    #[test]
    fn test_parse_with_children() {
        let adapter = PrismAdapter::new();
        let json = r#"{
            "node_type": "program_node",
            "location": {
                "start_line": 1,
                "start_column": 0,
                "end_line": 1,
                "end_column": 14,
                "start_offset": 0,
                "end_offset": 14
            },
            "children": [
                {
                    "node_type": "class_node",
                    "location": {
                        "start_line": 1,
                        "start_column": 0,
                        "end_line": 1,
                        "end_column": 14,
                        "start_offset": 0,
                        "end_offset": 14
                    },
                    "children": [],
                    "metadata": {"name": "Foo"},
                    "comments": [],
                    "formatting": {
                        "indent_level": 0,
                        "needs_blank_line_before": false,
                        "needs_blank_line_after": false,
                        "preserve_newlines": false,
                        "multiline": false,
                        "original_formatting": null
                    }
                }
            ],
            "metadata": {},
            "comments": [],
            "formatting": {
                "indent_level": 0,
                "needs_blank_line_before": false,
                "needs_blank_line_after": false,
                "preserve_newlines": false,
                "multiline": false,
                "original_formatting": null
            }
        }"#;

        let result = adapter.parse(json);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.node_type, NodeType::ProgramNode);
        assert_eq!(node.children.len(), 1);

        let child = &node.children[0];
        assert_eq!(child.node_type, NodeType::ClassNode);
        assert_eq!(child.metadata.get("name"), Some(&"Foo".to_string()));
    }

    #[test]
    fn test_parse_with_metadata() {
        let adapter = PrismAdapter::new();
        let json = r#"{
            "node_type": "def_node",
            "location": {
                "start_line": 1,
                "start_column": 0,
                "end_line": 1,
                "end_column": 10,
                "start_offset": 0,
                "end_offset": 10
            },
            "children": [],
            "metadata": {
                "name": "hello",
                "parameters_count": "1"
            },
            "comments": [],
            "formatting": {
                "indent_level": 0,
                "needs_blank_line_before": false,
                "needs_blank_line_after": false,
                "preserve_newlines": false,
                "multiline": false,
                "original_formatting": null
            }
        }"#;

        let result = adapter.parse(json);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.node_type, NodeType::DefNode);
        assert_eq!(node.metadata.get("name"), Some(&"hello".to_string()));
        assert_eq!(
            node.metadata.get("parameters_count"),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn test_parse_multiline() {
        let adapter = PrismAdapter::new();
        let json = r#"{
            "node_type": "class_node",
            "location": {
                "start_line": 1,
                "start_column": 0,
                "end_line": 3,
                "end_column": 3,
                "start_offset": 0,
                "end_offset": 20
            },
            "children": [],
            "metadata": {"name": "Foo"},
            "comments": [],
            "formatting": {
                "indent_level": 0,
                "needs_blank_line_before": false,
                "needs_blank_line_after": false,
                "preserve_newlines": false,
                "multiline": true,
                "original_formatting": null
            }
        }"#;

        let result = adapter.parse(json);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.node_type, NodeType::ClassNode);
        assert_eq!(node.formatting.multiline, true);
        assert!(node.is_multiline());
        assert_eq!(node.line_count(), 3);
    }

    #[test]
    fn test_parse_invalid_json() {
        let adapter = PrismAdapter::new();
        let json = "invalid json";

        let result = adapter.parse(json);
        assert!(result.is_err());

        match result {
            Err(RfmtError::PrismError(msg)) => {
                assert!(msg.contains("Failed to parse Prism JSON"));
            }
            _ => panic!("Expected PrismError"),
        }
    }

    #[test]
    fn test_parse_unknown_node_type() {
        let adapter = PrismAdapter::new();
        let json = r#"{
            "node_type": "totally_unknown_node",
            "location": {
                "start_line": 1,
                "start_column": 0,
                "end_line": 1,
                "end_column": 10,
                "start_offset": 0,
                "end_offset": 10
            },
            "children": [],
            "metadata": {},
            "comments": [],
            "formatting": {
                "indent_level": 0,
                "needs_blank_line_before": false,
                "needs_blank_line_after": false,
                "preserve_newlines": false,
                "multiline": false,
                "original_formatting": null
            }
        }"#;

        let result = adapter.parse(json);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(
            node.node_type,
            NodeType::Unknown("totally_unknown_node".to_string())
        );
        assert!(node.is_unknown());
        assert_eq!(node.unknown_type(), Some("totally_unknown_node"));
    }
}

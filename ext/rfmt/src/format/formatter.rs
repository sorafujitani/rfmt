//! Formatter - Main entry point for the rule-based formatting system
//!
//! The Formatter coordinates the formatting process:
//! 1. Initialize FormatContext with source and config
//! 2. Collect comments from AST
//! 3. Apply rules to generate Doc IR
//! 4. Print Doc IR to string using Printer

use crate::ast::{Node, NodeType};
use crate::config::Config;
use crate::doc::{concat, hardline, Doc, Printer};
use crate::error::Result;

use super::context::FormatContext;
use super::registry::RuleRegistry;
use super::rule::format_remaining_comments;

/// Main formatter that coordinates the formatting process.
///
/// The formatter uses a rule-based architecture where each node type
/// can have a specific formatting rule. Unhandled node types fall back
/// to source extraction.
pub struct Formatter {
    /// Configuration for formatting
    config: Config,
    /// Registry of formatting rules
    registry: RuleRegistry,
}

impl Formatter {
    /// Creates a new formatter with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: RuleRegistry::default_registry(),
        }
    }

    /// Creates a new formatter with a custom registry.
    pub fn with_registry(config: Config, registry: RuleRegistry) -> Self {
        Self { config, registry }
    }

    /// Formats Ruby source code.
    ///
    /// # Arguments
    /// * `source` - The original Ruby source code
    /// * `ast` - The parsed AST root node
    ///
    /// # Returns
    /// The formatted source code as a string
    pub fn format(&self, source: &str, ast: &Node) -> Result<String> {
        // 1. Initialize context
        let mut ctx = FormatContext::new(&self.config, source);

        // 2. Collect comments from AST
        ctx.collect_comments(ast);

        // 3. Generate Doc IR
        let doc = self.format_node(ast, &mut ctx)?;

        // 4. Handle remaining comments
        let last_code_line = FormatContext::find_last_code_line(ast);
        let remaining = format_remaining_comments(&mut ctx, last_code_line);

        let final_doc = if remaining.is_empty() {
            doc
        } else {
            concat(vec![doc, remaining])
        };

        // 5. Print to string
        let mut printer = Printer::new(&self.config);
        let result = printer.print(&final_doc);

        Ok(result)
    }

    /// Formats a single node.
    pub fn format_node(&self, node: &Node, ctx: &mut FormatContext) -> Result<Doc> {
        match &node.node_type {
            NodeType::ProgramNode => self.format_program(node, ctx),
            NodeType::StatementsNode => self.format_statements(node, ctx),
            _ => {
                // Use the rule registry for specific node types
                let rule = self.registry.get_rule(&node.node_type);
                rule.format(node, ctx, &self.registry)
            }
        }
    }

    /// Returns a reference to the registry for recursive formatting.
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Formats the program node (root).
    fn format_program(&self, node: &Node, ctx: &mut FormatContext) -> Result<Doc> {
        self.format_children_with_spacing(&node.children, ctx)
    }

    /// Formats a statements node (body of class/module/def).
    fn format_statements(&self, node: &Node, ctx: &mut FormatContext) -> Result<Doc> {
        self.format_children_with_spacing(&node.children, ctx)
    }

    /// Format a sequence of child nodes with appropriate line breaks.
    fn format_children_with_spacing(
        &self,
        children: &[Node],
        ctx: &mut FormatContext,
    ) -> Result<Doc> {
        if children.is_empty() {
            return Ok(Doc::Empty);
        }

        let mut docs: Vec<Doc> = Vec::with_capacity(children.len() * 2);

        for (i, child) in children.iter().enumerate() {
            let child_doc = self.format_node(child, ctx)?;
            docs.push(child_doc);

            // Add newlines between statements
            if let Some(next_child) = children.get(i + 1) {
                let current_end_line = child.location.end_line;
                let next_start_line = next_child.location.start_line;
                let line_diff = next_start_line.saturating_sub(current_end_line);

                // Add 1 hardline if consecutive, 2 hardlines (1 blank line) if there was a gap
                docs.push(hardline());
                if line_diff > 1 {
                    docs.push(hardline());
                }
            }
        }

        Ok(concat(docs))
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location};
    use std::collections::HashMap;

    fn make_program_node(children: Vec<Node>, end_line: usize) -> Node {
        Node {
            node_type: NodeType::ProgramNode,
            location: Location::new(1, 0, end_line, 0, 0, 100),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_class_node(
        name: &str,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());

        Node {
            node_type: NodeType::ClassNode,
            location: Location::new(start_line, 0, end_line, 3, start_offset, end_offset),
            children: Vec::new(),
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_format_simple_class() {
        let source = "class Foo\nend";
        let class_node = make_class_node("Foo", 1, 2, 0, 13);
        let ast = make_program_node(vec![class_node], 2);

        let formatter = Formatter::default();
        let result = formatter.format(source, &ast).unwrap();

        assert_eq!(result, "class Foo\nend\n");
    }

    #[test]
    fn test_format_multiple_classes() {
        let source = "class Foo\nend\n\nclass Bar\nend";
        let class1 = make_class_node("Foo", 1, 2, 0, 13);
        let class2 = make_class_node("Bar", 4, 5, 15, 28);
        let ast = make_program_node(vec![class1, class2], 5);

        let formatter = Formatter::default();
        let result = formatter.format(source, &ast).unwrap();

        // Should preserve blank line between classes
        assert!(result.contains("class Foo\nend"));
        assert!(result.contains("class Bar\nend"));
        assert!(result.contains("\n\n")); // blank line preserved
    }

    #[test]
    fn test_formatter_with_custom_config() {
        let mut config = Config::default();
        config.formatting.indent_width = 4;

        let source = "class Foo\nend";
        let class_node = make_class_node("Foo", 1, 2, 0, 13);
        let ast = make_program_node(vec![class_node], 2);

        let formatter = Formatter::new(config);
        let result = formatter.format(source, &ast).unwrap();

        assert_eq!(result, "class Foo\nend\n");
    }
}

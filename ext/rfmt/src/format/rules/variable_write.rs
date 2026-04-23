//! VariableWriteRule - Formats Ruby variable assignment expressions
//!
//! Handles variable assignments including:
//! - Local variable writes: `x = value`
//! - Instance variable writes: `@x = value`
//! - Block value assignments: `x = if true then 1 else 2 end`
//! - Multiline method chain assignments: `x = foo.bar.baz`

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_child, format_leading_comments, format_trailing_comment, line_leading_indent,
    reformat_chain_lines, FormatRule,
};

/// Rule for formatting local variable write expressions.
pub struct LocalVariableWriteRule;

impl FormatRule for LocalVariableWriteRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_variable_write(node, ctx, registry)
    }
}

/// Rule for formatting instance variable write expressions.
pub struct InstanceVariableWriteRule;

impl FormatRule for InstanceVariableWriteRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_variable_write(node, ctx, registry)
    }
}

/// Shared implementation for variable write formatting.
///
/// Handles three cases:
/// 1. Block value (if/unless/case/begin/while/until/for): formats on new line with indent
/// 2. Multiline method chain: reformats with indented chain style
/// 3. Simple value: inline assignment
fn format_variable_write(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(5);

    let start_line = node.location.start_line;
    let end_line = node.location.end_line;

    // 1. Leading comments
    let leading = format_leading_comments(ctx, start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // 2. Get variable name from metadata
    let name = node.metadata.get("name").map(|s| s.as_str()).unwrap_or("_");

    // 3. Get value node (first child)
    let value = match node.children.first() {
        Some(v) => v,
        None => {
            // No value: fallback to source extraction
            if let Some(source_text) = ctx.extract_source(node) {
                docs.push(text(source_text.to_string()));
            }
            let trailing = format_trailing_comment(ctx, end_line);
            if !trailing.is_empty() {
                docs.push(trailing);
            }
            return Ok(concat(docs));
        }
    };

    // 4. Check if value is a block construct
    let is_block_value = matches!(
        value.node_type,
        NodeType::IfNode
            | NodeType::UnlessNode
            | NodeType::CaseNode
            | NodeType::CaseMatchNode
            | NodeType::BeginNode
            | NodeType::WhileNode
            | NodeType::UntilNode
            | NodeType::ForNode
    );

    if is_block_value {
        // Block value: format on new line with indent
        // x =
        //   if true
        //     1
        //   else
        //     2
        //   end
        docs.push(text(format!("{} =", name)));
        docs.push(indent(concat(vec![
            hardline(),
            format_child(value, ctx, registry)?,
        ])));
    } else {
        // Check for multiline method chain
        let is_multiline_call = matches!(value.node_type, NodeType::CallNode)
            && value.location.start_line != value.location.end_line;

        docs.push(text(format!("{} = ", name)));

        if is_multiline_call {
            // Multiline call: reformat chain with indented style
            if let Some(source_text) = ctx.extract_source(value) {
                let base_indent = line_leading_indent(ctx.source(), node.location.start_offset);
                let reformatted = reformat_chain_lines(
                    source_text,
                    base_indent,
                    ctx.config().formatting.indent_width,
                );
                docs.push(text(reformatted.trim_start().to_string()));
            }
        } else {
            // Simple value: extract from source trimmed
            if let Some(source_text) = ctx.extract_source(value) {
                docs.push(text(source_text.trim().to_string()));
            }
        }
    }

    // 5. Trailing comment
    let trailing = format_trailing_comment(ctx, end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location};
    use crate::config::Config;
    use crate::doc::Printer;
    use std::collections::HashMap;

    fn make_local_var_write_node(
        name: &str,
        children: Vec<Node>,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());

        Node {
            node_type: NodeType::LocalVariableWriteNode,
            location: Location::new(start_line, 0, end_line, 10, start_offset, end_offset),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_instance_var_write_node(
        name: &str,
        children: Vec<Node>,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());

        Node {
            node_type: NodeType::InstanceVariableWriteNode,
            location: Location::new(start_line, 0, end_line, 10, start_offset, end_offset),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_integer_node(line: usize, start_offset: usize, end_offset: usize) -> Node {
        Node {
            node_type: NodeType::IntegerNode,
            location: Location::new(line, 0, line, 1, start_offset, end_offset),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_local_var_assignment() {
        let config = Config::default();
        let source = "x = 1";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let value_node = make_integer_node(1, 4, 5);
        let node = make_local_var_write_node("x", vec![value_node], 1, 1, 0, 5);
        ctx.collect_comments(&node);

        let rule = LocalVariableWriteRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "x = 1");
    }

    #[test]
    fn test_simple_instance_var_assignment() {
        let config = Config::default();
        let source = "@value = 42";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let value_node = make_integer_node(1, 9, 11);
        let node = make_instance_var_write_node("@value", vec![value_node], 1, 1, 0, 11);
        ctx.collect_comments(&node);

        let rule = InstanceVariableWriteRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "@value = 42");
    }

    #[test]
    fn test_block_value_assignment() {
        let config = Config::default();
        let source = "x = if true\n  1\nelse\n  2\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let if_node = Node {
            node_type: NodeType::IfNode,
            location: Location::new(1, 4, 5, 3, 4, 28),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_local_var_write_node("x", vec![if_node], 1, 5, 0, 28);
        ctx.collect_comments(&node);

        let rule = LocalVariableWriteRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should format with value on new line
        assert!(result.contains("x ="));
        assert!(result.contains("if"));
    }

    #[test]
    fn test_multiline_chain_assignment() {
        let config = Config::default();
        let source = "x = foo\n  .bar\n  .baz";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let call_node = Node {
            node_type: NodeType::CallNode,
            location: Location::new(1, 4, 3, 6, 4, 21),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_local_var_write_node("x", vec![call_node], 1, 3, 0, 21);
        ctx.collect_comments(&node);

        let rule = LocalVariableWriteRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should reformat chain with indented style
        assert!(result.contains("x = foo"));
        assert!(result.contains(".bar"));
        assert!(result.contains(".baz"));
    }
}

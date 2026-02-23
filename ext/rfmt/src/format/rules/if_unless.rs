//! IfRule and UnlessRule - Formats Ruby if/unless conditionals
//!
//! Handles:
//! - Normal if/unless: `if cond ... end`
//! - Postfix if/unless: `expr if cond`
//! - Ternary operator: `cond ? then_expr : else_expr`
//! - Inline then: `if cond then expr end`
//! - elsif/else chains

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_leading_comments, format_statements, format_trailing_comment, FormatRule,
};

/// Rule for formatting if conditionals.
pub struct IfRule;

/// Rule for formatting unless conditionals.
pub struct UnlessRule;

impl FormatRule for IfRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_if_unless(node, ctx, registry, "if", false)
    }
}

impl FormatRule for UnlessRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_if_unless(node, ctx, registry, "unless", false)
    }
}

/// Formats if/unless/elsif/else constructs.
///
/// # Arguments
/// * `node` - The if/unless node
/// * `ctx` - The formatting context
/// * `registry` - The rule registry for recursive formatting
/// * `keyword` - "if" or "unless"
/// * `is_elsif` - true if this is an elsif clause (don't emit 'end')
fn format_if_unless(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    keyword: &str,
    is_elsif: bool,
) -> Result<Doc> {
    // Check if this is a postfix if (modifier form)
    let is_postfix = if let (Some(predicate), Some(statements)) =
        (node.children.first(), node.children.get(1))
    {
        statements.location.start_offset < predicate.location.start_offset
    } else {
        false
    };

    // Postfix if/unless: "statement if/unless condition"
    if is_postfix && !is_elsif {
        return format_postfix(node, ctx, registry, keyword);
    }

    // Check for ternary operator
    let is_ternary = node
        .metadata
        .get("is_ternary")
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_ternary && !is_elsif {
        return format_ternary(node, ctx);
    }

    // Check for inline then style: "if true then 1 end"
    let is_single_line = node.location.start_line == node.location.end_line;
    let is_inline_then = !is_elsif && is_single_line && node.children.get(2).is_none();

    if is_inline_then {
        return format_inline_then(node, ctx, keyword);
    }

    // Normal if/unless/elsif
    format_normal(node, ctx, registry, keyword, is_elsif)
}

/// Formats postfix if/unless: `statement if/unless condition`
fn format_postfix(
    node: &Node,
    ctx: &mut FormatContext,
    _registry: &RuleRegistry,
    keyword: &str,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(6);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // Emit statement
    if let Some(statements) = node.children.get(1) {
        if let Some(source_text) = ctx.extract_source(statements) {
            docs.push(text(source_text.trim()));
        }
    }

    docs.push(text(" "));
    docs.push(text(keyword));
    docs.push(text(" "));

    // Emit condition
    if let Some(predicate) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(predicate) {
            docs.push(text(source_text));
        }
    }

    // Trailing comment
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats ternary operator: `cond ? then_expr : else_expr`
fn format_ternary(node: &Node, ctx: &mut FormatContext) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // Emit condition
    if let Some(predicate) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(predicate) {
            docs.push(text(source_text));
        }
    }

    docs.push(text(" ? "));

    // Emit then expression
    if let Some(statements) = node.children.get(1) {
        if let Some(source_text) = ctx.extract_source(statements) {
            docs.push(text(source_text.trim()));
        }
    }

    docs.push(text(" : "));

    // Emit else expression
    if let Some(else_node) = node.children.get(2) {
        if let Some(else_statements) = else_node.children.first() {
            if let Some(source_text) = ctx.extract_source(else_statements) {
                docs.push(text(source_text.trim()));
            }
        }
    }

    // Trailing comment
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats inline then style: `if cond then expr end`
fn format_inline_then(node: &Node, ctx: &mut FormatContext, keyword: &str) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text(keyword));
    docs.push(text(" "));

    // Emit condition
    if let Some(predicate) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(predicate) {
            docs.push(text(source_text));
        }
    }

    docs.push(text(" then "));

    // Emit statement
    if let Some(statements) = node.children.get(1) {
        if let Some(source_text) = ctx.extract_source(statements) {
            docs.push(text(source_text.trim()));
        }
    }

    docs.push(text(" end"));

    // Trailing comment
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats normal if/unless/elsif with potential else
fn format_normal(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    keyword: &str,
    is_elsif: bool,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(12);

    // Leading comments (only for outermost if/unless)
    if !is_elsif {
        let leading = format_leading_comments(ctx, node.location.start_line);
        if !leading.is_empty() {
            docs.push(leading);
        }
    }

    // Emit 'if'/'unless' or 'elsif' keyword
    if is_elsif {
        docs.push(text("elsif "));
    } else {
        docs.push(text(keyword));
        docs.push(text(" "));
    }

    // Emit predicate (condition)
    if let Some(predicate) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(predicate) {
            docs.push(text(source_text));
        }
    }

    // Trailing comment on same line as if/unless/elsif
    let trailing = format_trailing_comment(ctx, node.location.start_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    // Emit then clause (second child is StatementsNode)
    if let Some(statements) = node.children.get(1) {
        if matches!(statements.node_type, NodeType::StatementsNode) {
            let body_doc = format_statements(statements, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
        }
    }

    // Check for elsif/else (third child)
    if let Some(consequent) = node.children.get(2) {
        match &consequent.node_type {
            NodeType::IfNode => {
                // This is an elsif clause
                docs.push(hardline());
                let elsif_doc = format_if_unless(consequent, ctx, registry, "if", true)?;
                docs.push(elsif_doc);
            }
            NodeType::ElseNode => {
                // This is an else clause
                docs.push(hardline());
                docs.push(text("else"));

                // Emit else body (first child of ElseNode)
                if let Some(else_statements) = consequent.children.first() {
                    if matches!(else_statements.node_type, NodeType::StatementsNode) {
                        let body_doc = format_statements(else_statements, ctx, registry)?;
                        docs.push(indent(concat(vec![hardline(), body_doc])));
                    }
                }
            }
            _ => {}
        }
    }

    // Only emit 'end' for the outermost if (not for elsif)
    if !is_elsif {
        docs.push(hardline());
        docs.push(text("end"));

        // Trailing comment on end line
        let end_trailing = format_trailing_comment(ctx, node.location.end_line);
        if !end_trailing.is_empty() {
            docs.push(end_trailing);
        }
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

    fn make_if_node(
        children: Vec<Node>,
        metadata: HashMap<String, String>,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Node {
        Node {
            node_type: NodeType::IfNode,
            location: Location::new(start_line, 0, end_line, 0, start_offset, end_offset),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_predicate_node(
        start_offset: usize,
        end_offset: usize,
        start_line: usize,
    ) -> Node {
        Node {
            node_type: NodeType::CallNode,
            location: Location::new(start_line, 0, start_line, 0, start_offset, end_offset),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_statements_node(
        start_offset: usize,
        end_offset: usize,
        start_line: usize,
        end_line: usize,
    ) -> Node {
        Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(start_line, 0, end_line, 0, start_offset, end_offset),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_if() {
        let config = Config::default();
        let source = "if true\n  puts 'yes'\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // predicate: "true" at offset 3-7
        let predicate = make_predicate_node(3, 7, 1);
        // statements: "puts 'yes'" at offset 10-20
        let statements = make_statements_node(10, 20, 2, 2);

        let node = make_if_node(vec![predicate, statements], HashMap::new(), 1, 3, 0, 24);
        ctx.collect_comments(&node);

        let rule = IfRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("if true"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_postfix_if() {
        let config = Config::default();
        let source = "puts 'yes' if true";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // For postfix: statements come before predicate in source
        // predicate: "true" at offset 14-18
        let predicate = make_predicate_node(14, 18, 1);
        // statements: "puts 'yes'" at offset 0-10
        let statements = make_statements_node(0, 10, 1, 1);

        let node = make_if_node(vec![predicate, statements], HashMap::new(), 1, 1, 0, 18);
        ctx.collect_comments(&node);

        let rule = IfRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("puts 'yes' if true"));
    }
}

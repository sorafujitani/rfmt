//! BeginRule - Formats Ruby begin/rescue/ensure blocks
//!
//! Handles:
//! - Explicit begin...end blocks
//! - Implicit begin wrapping method body with rescue/ensure

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_child, format_leading_comments, format_statements, format_trailing_comment, FormatRule,
};

/// Rule for formatting begin/rescue/ensure blocks.
pub struct BeginRule;

/// Rule for formatting rescue clauses.
pub struct RescueRule;

/// Rule for formatting ensure clauses.
pub struct EnsureRule;

impl FormatRule for BeginRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_begin(node, ctx, registry)
    }
}

impl FormatRule for RescueRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_rescue(node, ctx, registry, 0)
    }
}

impl FormatRule for EnsureRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_ensure(node, ctx, registry, 0)
    }
}

/// Formats begin block
fn format_begin(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    // Check if this is an explicit begin block by looking at source
    let is_explicit_begin = ctx
        .extract_source(node)
        .map(|s| s.trim_start().starts_with("begin"))
        .unwrap_or(false);

    if is_explicit_begin {
        format_explicit_begin(node, ctx, registry)
    } else {
        // Implicit begin - emit children directly
        format_implicit_begin(node, ctx, registry)
    }
}

/// Formats explicit begin...end block
fn format_explicit_begin(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text("begin"));
    docs.push(hardline());

    for child in &node.children {
        let child_doc = format_child(child, ctx, registry)?;
        docs.push(child_doc);
        docs.push(hardline());
    }

    docs.push(text("end"));

    // Trailing comment on end line
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats implicit begin (children only, no begin/end keywords)
fn format_implicit_begin(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(node.children.len() * 2);

    for (i, child) in node.children.iter().enumerate() {
        if i > 0 {
            docs.push(hardline());
        }
        let child_doc = format_child(child, ctx, registry)?;
        docs.push(child_doc);
    }

    Ok(concat(docs))
}

/// Formats rescue clause
fn format_rescue(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    dedent_level: usize,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // For rescue within a method, dedent by 1 level
    // But since we're in Doc IR, we handle this at the caller level
    let _ = dedent_level;

    docs.push(text("rescue"));

    // Extract exception classes and variable from source
    if let Some(source_text) = ctx.extract_source(node) {
        // Find the rescue declaration part (first line only, unless trailing comma/backslash)
        let mut rescue_decl = String::new();
        let mut expect_continuation = false;

        for line in source_text.lines() {
            let trimmed = line.trim();

            if rescue_decl.is_empty() {
                // First line - remove "rescue" prefix
                let after_rescue = trimmed.trim_start_matches("rescue").trim();
                if !after_rescue.is_empty() {
                    // Check if line ends with continuation marker
                    expect_continuation =
                        after_rescue.ends_with(',') || after_rescue.ends_with('\\');
                    rescue_decl.push_str(after_rescue.trim_end_matches('\\').trim());
                }
                if !expect_continuation {
                    break;
                }
            } else if expect_continuation {
                // Continuation line after trailing comma or backslash
                if !rescue_decl.ends_with(' ') {
                    rescue_decl.push(' ');
                }
                let content = trimmed.trim_end_matches('\\').trim();
                rescue_decl.push_str(content);
                expect_continuation = trimmed.ends_with(',') || trimmed.ends_with('\\');
                if !expect_continuation {
                    break;
                }
            } else {
                break;
            }
        }

        if !rescue_decl.is_empty() {
            docs.push(text(" "));
            docs.push(text(rescue_decl));
        }
    }

    docs.push(hardline());

    // Emit rescue body and handle subsequent rescue nodes
    for child in &node.children {
        match &child.node_type {
            NodeType::StatementsNode => {
                let body_doc = format_statements(child, ctx, registry)?;
                docs.push(indent(body_doc));
            }
            NodeType::RescueNode => {
                // Emit subsequent rescue clause
                let rescue_doc = format_rescue(child, ctx, registry, dedent_level)?;
                docs.push(rescue_doc);
            }
            _ => {
                // Skip exception classes and variable (already handled above)
            }
        }
    }

    Ok(concat(docs))
}

/// Formats ensure clause
fn format_ensure(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    dedent_level: usize,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(6);
    let _ = dedent_level;

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text("ensure"));
    docs.push(hardline());

    // Emit ensure body statements
    for child in &node.children {
        match &child.node_type {
            NodeType::StatementsNode => {
                let body_doc = format_statements(child, ctx, registry)?;
                docs.push(indent(body_doc));
            }
            _ => {
                let child_doc = format_child(child, ctx, registry)?;
                docs.push(indent(child_doc));
            }
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

    fn make_begin_node(children: Vec<Node>, start_offset: usize, end_offset: usize) -> Node {
        Node {
            node_type: NodeType::BeginNode,
            location: Location::new(1, 0, 5, 3, start_offset, end_offset),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_explicit_begin() {
        let config = Config::default();
        let source = "begin\n  puts 'hello'\nrescue\n  puts 'error'\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let statements = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(2, 2, 2, 14, 8, 20),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let rescue_body = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(4, 2, 4, 14, 30, 42),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let rescue = Node {
            node_type: NodeType::RescueNode,
            location: Location::new(3, 0, 4, 14, 21, 42),
            children: vec![rescue_body],
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_begin_node(vec![statements, rescue], 0, 46);
        ctx.collect_comments(&node);

        let rule = BeginRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("begin"));
        assert!(result.contains("rescue"));
        assert!(result.contains("end"));
    }
}

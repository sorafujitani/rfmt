//! CaseRule, WhenRule, CaseMatchRule, InRule - Formats Ruby case expressions
//!
//! Handles:
//! - case/when: `case x when ... end`
//! - case/in: `case x in ... end` (pattern matching)

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_leading_comments, format_statements, format_trailing_comment, FormatRule,
};

/// Rule for formatting case expressions.
pub struct CaseRule;

/// Rule for formatting when clauses.
pub struct WhenRule;

/// Rule for formatting case match expressions (Ruby 3.0+ pattern matching).
pub struct CaseMatchRule;

/// Rule for formatting in clauses (pattern matching).
pub struct InRule;

impl FormatRule for CaseRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_case(node, ctx, registry)
    }
}

impl FormatRule for WhenRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_when(node, ctx, registry)
    }
}

impl FormatRule for CaseMatchRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_case_match(node, ctx, registry)
    }
}

impl FormatRule for InRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_in(node, ctx, registry)
    }
}

/// Formats case expression
fn format_case(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // "case" keyword
    docs.push(text("case"));

    // Find predicate (first child that isn't WhenNode or ElseNode)
    let mut when_start_idx = 0;
    if let Some(first_child) = node.children.first() {
        if !matches!(
            first_child.node_type,
            NodeType::WhenNode | NodeType::ElseNode
        ) {
            // This is the predicate
            if let Some(source_text) = ctx.extract_source(first_child) {
                docs.push(text(" "));
                docs.push(text(source_text));
            }
            when_start_idx = 1;
        }
    }

    // Emit when clauses and else
    for child in node.children.iter().skip(when_start_idx) {
        match &child.node_type {
            NodeType::WhenNode => {
                docs.push(hardline());
                let when_doc = format_when(child, ctx, registry)?;
                docs.push(when_doc);
            }
            NodeType::ElseNode => {
                docs.push(hardline());
                docs.push(text("else"));

                // Emit else body
                for else_child in &child.children {
                    if matches!(else_child.node_type, NodeType::StatementsNode) {
                        let body_doc = format_statements(else_child, ctx, registry)?;
                        docs.push(indent(concat(vec![hardline(), body_doc])));
                    }
                }
            }
            _ => {}
        }
    }

    // End keyword
    docs.push(hardline());
    docs.push(text("end"));

    // Trailing comment on end line
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats when clause
fn format_when(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(6);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text("when "));

    // Collect conditions (all children except StatementsNode)
    let conditions: Vec<_> = node
        .children
        .iter()
        .filter(|c| !matches!(c.node_type, NodeType::StatementsNode))
        .collect();

    // Emit conditions with comma separator
    for (i, cond) in conditions.iter().enumerate() {
        if let Some(source_text) = ctx.extract_source(cond) {
            docs.push(text(source_text));
            if i < conditions.len() - 1 {
                docs.push(text(", "));
            }
        }
    }

    let statements = node
        .children
        .iter()
        .find(|c| matches!(c.node_type, NodeType::StatementsNode));

    let is_single_line = node.location.start_line == node.location.end_line;

    if is_single_line {
        // Inline style: when X then Y
        if let Some(statements) = statements {
            if let Some(source_text) = ctx.extract_source(statements) {
                docs.push(text(" then "));
                docs.push(text(source_text));
            }
        }
    } else {
        // Multi-line style: when X\n  Y
        if let Some(statements) = statements {
            let body_doc = format_statements(statements, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
        }
    }

    Ok(concat(docs))
}

/// Formats case match expression (Ruby 3.0+ pattern matching)
fn format_case_match(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // "case" keyword
    docs.push(text("case"));

    // Find predicate (first child that isn't InNode or ElseNode)
    let mut in_start_idx = 0;
    if let Some(first_child) = node.children.first() {
        if !matches!(first_child.node_type, NodeType::InNode | NodeType::ElseNode) {
            // This is the predicate
            if let Some(source_text) = ctx.extract_source(first_child) {
                docs.push(text(" "));
                docs.push(text(source_text));
            }
            in_start_idx = 1;
        }
    }

    // Emit in clauses and else
    for child in node.children.iter().skip(in_start_idx) {
        match &child.node_type {
            NodeType::InNode => {
                docs.push(hardline());
                let in_doc = format_in(child, ctx, registry)?;
                docs.push(in_doc);
            }
            NodeType::ElseNode => {
                docs.push(hardline());
                docs.push(text("else"));

                // Emit else body
                for else_child in &child.children {
                    if matches!(else_child.node_type, NodeType::StatementsNode) {
                        let body_doc = format_statements(else_child, ctx, registry)?;
                        docs.push(indent(concat(vec![hardline(), body_doc])));
                    }
                }
            }
            _ => {}
        }
    }

    // End keyword
    docs.push(hardline());
    docs.push(text("end"));

    // Trailing comment on end line
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats in clause (pattern matching)
fn format_in(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(6);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text("in "));

    // First child is the pattern
    if let Some(pattern) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(pattern) {
            docs.push(text(source_text));
        }
    }

    let is_single_line = node.location.start_line == node.location.end_line;

    if is_single_line {
        // Inline style: in X then Y
        if let Some(statements) = node.children.get(1) {
            if let Some(source_text) = ctx.extract_source(statements) {
                docs.push(text(" then "));
                docs.push(text(source_text));
            }
        }
    } else {
        // Multi-line style: in X\n  Y
        if let Some(statements) = node.children.get(1) {
            if matches!(statements.node_type, NodeType::StatementsNode) {
                let body_doc = format_statements(statements, ctx, registry)?;
                docs.push(indent(concat(vec![hardline(), body_doc])));
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

    fn make_case_node(children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        Node {
            node_type: NodeType::CaseNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_when_node(children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        Node {
            node_type: NodeType::WhenNode,
            location: Location::new(start_line, 0, end_line, 0, 0, 30),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_case() {
        let config = Config::default();
        let source = "case x\nwhen 1\n  puts 'one'\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // predicate node
        let predicate = Node {
            node_type: NodeType::CallNode,
            location: Location::new(1, 5, 1, 6, 5, 6),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        // when node
        let when_cond = Node {
            node_type: NodeType::IntegerNode,
            location: Location::new(2, 5, 2, 6, 12, 13),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let when_body = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(3, 2, 3, 13, 16, 27),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let when_node = make_when_node(vec![when_cond, when_body], 2, 3);

        let node = make_case_node(vec![predicate, when_node], 1, 4);
        ctx.collect_comments(&node);

        let rule = CaseRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("case x"));
        assert!(result.contains("when 1"));
        assert!(result.contains("end"));
    }
}

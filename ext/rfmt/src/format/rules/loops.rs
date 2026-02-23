//! Loop rules - Formats Ruby while/until/for loops
//!
//! Handles:
//! - while loops: `while cond ... end`
//! - until loops: `until cond ... end`
//! - for loops: `for x in collection ... end`
//! - Postfix forms: `expr while/until cond`

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_leading_comments, format_statements, format_trailing_comment, FormatRule,
};

/// Rule for formatting while loops.
pub struct WhileRule;

/// Rule for formatting until loops.
pub struct UntilRule;

/// Rule for formatting for loops.
pub struct ForRule;

impl FormatRule for WhileRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_while_until(node, ctx, registry, "while")
    }
}

impl FormatRule for UntilRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_while_until(node, ctx, registry, "until")
    }
}

impl FormatRule for ForRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_for(node, ctx, registry)
    }
}

/// Formats while/until loop
fn format_while_until(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    keyword: &str,
) -> Result<Doc> {
    // Check if this is a postfix while/until (modifier form)
    // In postfix form: "statement while/until condition"
    let is_postfix = if node.children.len() >= 2 {
        let predicate = &node.children[0];
        let body = &node.children[1];
        body.location.start_offset < predicate.location.start_offset
    } else {
        false
    };

    if is_postfix {
        return format_postfix_while_until(node, ctx, keyword);
    }

    // Normal while/until with do...end
    format_normal_while_until(node, ctx, registry, keyword)
}

/// Formats postfix while/until
fn format_postfix_while_until(node: &Node, ctx: &mut FormatContext, keyword: &str) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(6);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // Extract from source for postfix form
    if let Some(source_text) = ctx.extract_source(node) {
        docs.push(text(source_text));
    }

    // Trailing comment
    let trailing = format_trailing_comment(ctx, node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    let _ = keyword; // Used for potential future formatting

    Ok(concat(docs))
}

/// Formats normal while/until with do...end
fn format_normal_while_until(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    keyword: &str,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text(keyword));
    docs.push(text(" "));

    // Emit predicate (condition) - first child
    if let Some(predicate) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(predicate) {
            docs.push(text(source_text));
        }
    }

    // Trailing comment on same line as while/until
    let trailing = format_trailing_comment(ctx, node.location.start_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    // Emit body - second child (StatementsNode)
    if let Some(body) = node.children.get(1) {
        if matches!(body.node_type, NodeType::StatementsNode) {
            let body_doc = format_statements(body, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
        }
    }

    docs.push(hardline());
    docs.push(text("end"));

    // Trailing comment on end line
    let end_trailing = format_trailing_comment(ctx, node.location.end_line);
    if !end_trailing.is_empty() {
        docs.push(end_trailing);
    }

    Ok(concat(docs))
}

/// Formats for loop
fn format_for(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(10);

    // Leading comments
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    docs.push(text("for "));

    // node.children: [index, collection, statements]
    // index: LocalVariableTargetNode or MultiTargetNode
    // collection: expression
    // statements: StatementsNode

    // Emit index variable - first child
    if let Some(index) = node.children.first() {
        if let Some(source_text) = ctx.extract_source(index) {
            docs.push(text(source_text));
        }
    }

    docs.push(text(" in "));

    // Emit collection - second child
    if let Some(collection) = node.children.get(1) {
        if let Some(source_text) = ctx.extract_source(collection) {
            docs.push(text(source_text));
        }
    }

    // Emit body - third child (StatementsNode)
    if let Some(body) = node.children.get(2) {
        if matches!(body.node_type, NodeType::StatementsNode) {
            let body_doc = format_statements(body, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
        }
    }

    docs.push(hardline());
    docs.push(text("end"));

    // Trailing comment on end line
    let trailing = format_trailing_comment(ctx, node.location.end_line);
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

    fn make_while_node(
        children: Vec<Node>,
        start_line: usize,
        end_line: usize,
        start_offset: usize,
        end_offset: usize,
    ) -> Node {
        Node {
            node_type: NodeType::WhileNode,
            location: Location::new(start_line, 0, end_line, 3, start_offset, end_offset),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_for_node(children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        Node {
            node_type: NodeType::ForNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_while() {
        let config = Config::default();
        let source = "while true\n  puts 'loop'\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // predicate: "true" at offset 6-10
        let predicate = Node {
            node_type: NodeType::TrueNode,
            location: Location::new(1, 6, 1, 10, 6, 10),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        // body statements at offset 13-25
        let body = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(2, 2, 2, 14, 13, 25),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_while_node(vec![predicate, body], 1, 3, 0, 29);
        ctx.collect_comments(&node);

        let rule = WhileRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("while true"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_simple_for() {
        let config = Config::default();
        let source = "for x in items\n  puts x\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // index: "x" at offset 4-5
        let index = Node {
            node_type: NodeType::LocalVariableReadNode,
            location: Location::new(1, 4, 1, 5, 4, 5),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        // collection: "items" at offset 9-14
        let collection = Node {
            node_type: NodeType::LocalVariableReadNode,
            location: Location::new(1, 9, 1, 14, 9, 14),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        // body
        let body = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(2, 2, 2, 8, 17, 23),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_for_node(vec![index, collection, body], 1, 3);
        ctx.collect_comments(&node);

        let rule = ForRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("for x in items"));
        assert!(result.contains("end"));
    }
}

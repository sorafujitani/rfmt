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

/// True when this BeginNode represents an implicit begin (the source does not
/// start with the `begin` keyword) AND carries at least one rescue/else/ensure
/// clause. Such bodies need the rescue/else/ensure keywords emitted at the
/// outer (e.g. `def`) indent level rather than at the body indent level.
pub(crate) fn is_implicit_begin_with_clauses(node: &Node, ctx: &FormatContext) -> bool {
    if node.node_type != NodeType::BeginNode {
        return false;
    }
    let has_clause = node.children.iter().any(|c| {
        matches!(
            c.node_type,
            NodeType::RescueNode | NodeType::EnsureNode | NodeType::ElseNode
        )
    });
    if !has_clause {
        return false;
    }
    ctx.extract_source(node)
        .map(|s| !s.trim_start().starts_with("begin"))
        .unwrap_or(false)
}

/// Emits the body of a construct (def/class/module/block) whose body is an
/// implicit BeginNode with rescue/else/ensure clauses.
///
/// The returned Doc is meant to sit between the opening line (e.g. `def foo`)
/// and a trailing `hardline + text("end")` emitted by the caller. Body
/// statements are wrapped in `indent`, while rescue/else/ensure clause
/// keywords are emitted at the caller's current indent level so that they
/// align with the opener instead of the body.
pub(crate) fn format_implicit_begin_body(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut body_children: Vec<&Node> = Vec::new();
    let mut clause_children: Vec<&Node> = Vec::new();

    for child in &node.children {
        match child.node_type {
            NodeType::RescueNode | NodeType::EnsureNode | NodeType::ElseNode => {
                clause_children.push(child);
            }
            _ => {
                body_children.push(child);
            }
        }
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(clause_children.len() * 2 + 2);

    if !body_children.is_empty() {
        let mut body_docs: Vec<Doc> = Vec::with_capacity(body_children.len() * 2 + 1);
        body_docs.push(hardline());
        for (i, child) in body_children.iter().enumerate() {
            if i > 0 {
                body_docs.push(hardline());
            }
            body_docs.push(format_child(child, ctx, registry)?);
        }
        docs.push(indent(concat(body_docs)));
    }

    for clause in clause_children {
        docs.push(hardline());
        docs.push(format_begin_clause(clause, ctx, registry)?);
    }

    Ok(concat(docs))
}

/// Emits a rescue/else/ensure clause the way the enclosing begin expects.
///
/// `format_child` sends ElseNode to `FallbackRule`, which slices the source
/// between `else` and the next keyword — but Prism's `ElseNode.location`
/// stretches into the *following* clause's keyword (`ensure`, or the
/// begin's `end`). Emitting that slice as-is duplicates whichever keyword
/// comes after, producing e.g. `else\n  y\nensure\nensure\n  z\nend` or
/// `else\n  y\nend\nend`. Handle ElseNode explicitly so we emit only
/// `else\n  body` and let the caller (or the subsequent EnsureRule) write
/// the following keyword.
fn format_begin_clause(
    clause: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    if !matches!(clause.node_type, NodeType::ElseNode) {
        return format_child(clause, ctx, registry);
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(3);
    docs.push(text("else"));
    for child in &clause.children {
        if matches!(child.node_type, NodeType::StatementsNode) {
            let body_doc = format_statements(child, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
        }
    }
    Ok(concat(docs))
}

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
fn format_begin(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
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

    let mut body_children: Vec<&Node> = Vec::new();
    let mut clause_children: Vec<&Node> = Vec::new();
    for child in &node.children {
        match child.node_type {
            NodeType::RescueNode | NodeType::EnsureNode | NodeType::ElseNode => {
                clause_children.push(child);
            }
            _ => body_children.push(child),
        }
    }

    if !body_children.is_empty() {
        let mut body_docs: Vec<Doc> = Vec::with_capacity(body_children.len() * 2 + 1);
        body_docs.push(hardline());
        for (i, child) in body_children.iter().enumerate() {
            if i > 0 {
                body_docs.push(hardline());
            }
            body_docs.push(format_child(child, ctx, registry)?);
        }
        docs.push(indent(concat(body_docs)));
    }

    for clause in &clause_children {
        docs.push(hardline());
        docs.push(format_begin_clause(clause, ctx, registry)?);
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

    // Emit any standalone comments that sit immediately before the rescue
    // keyword. Without this they would be picked up as leading comments of
    // the first statement inside the rescue body, which visually moves an
    // annotation like `# retry on flaky errors` *into* the rescue branch.
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

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

    // Emit rescue body (indented under the rescue keyword) and subsequent
    // chained rescue clauses (at the same indent level as this rescue).
    //
    // The hardline lives INSIDE the `indent(...)` wrap so that the body's
    // first statement lands at `caller_indent + indent_width`, matching the
    // indentation applied by hardlines later inside `format_statements`.
    let mut body_stmts: Option<&Node> = None;
    let mut subsequent: Option<&Node> = None;
    for child in &node.children {
        match &child.node_type {
            NodeType::StatementsNode => body_stmts = Some(child),
            NodeType::RescueNode => subsequent = Some(child),
            _ => {}
        }
    }

    if let Some(stmts) = body_stmts {
        let body_doc = format_statements(stmts, ctx, registry)?;
        docs.push(indent(concat(vec![hardline(), body_doc])));
    }

    if let Some(sub) = subsequent {
        docs.push(hardline());
        docs.push(format_rescue(sub, ctx, registry, dedent_level)?);
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

    // Emit ensure body. The hardline lives INSIDE the `indent(...)` wrap so
    // that every body statement — including the first one — lands at
    // `caller_indent + indent_width`.
    for child in &node.children {
        match &child.node_type {
            NodeType::StatementsNode => {
                let body_doc = format_statements(child, ctx, registry)?;
                docs.push(indent(concat(vec![hardline(), body_doc])));
            }
            _ => {
                let child_doc = format_child(child, ctx, registry)?;
                docs.push(indent(concat(vec![hardline(), child_doc])));
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

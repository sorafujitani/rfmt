//! CallRule - Formats Ruby method calls
//!
//! Handles:
//! - Simple calls: `foo.bar`
//! - Calls with blocks: `foo.bar do ... end` or `foo.bar { ... }`
//! - Method chains: `foo.bar.baz`

use std::borrow::Cow;

use crate::ast::{Node, NodeType};
use crate::doc::{align, concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_child, format_comments_before_end, format_leading_comments, format_statements,
    format_trailing_comment, line_leading_indent, mark_comments_in_range_emitted,
    reformat_chain_lines, strip_one_trailing_newline, FormatRule,
};

/// Rule for formatting method calls.
pub struct CallRule;

/// Rule for formatting blocks (do...end and {...}).
pub struct BlockRule;

/// Rule for formatting lambdas.
pub struct LambdaRule;

/// Block style for Ruby blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockStyle {
    DoEnd,  // do ... end
    Braces, // { ... }
}

impl FormatRule for CallRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_call(node, ctx, registry)
    }
}

impl FormatRule for BlockRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        // Detect block style and format accordingly
        let block_style = detect_block_style(node, ctx);
        match block_style {
            BlockStyle::DoEnd => format_do_end_block(node, ctx, registry),
            BlockStyle::Braces => format_brace_block(node, ctx, registry),
        }
    }
}

impl FormatRule for LambdaRule {
    fn format(
        &self,
        node: &Node,
        ctx: &mut FormatContext,
        _registry: &RuleRegistry,
    ) -> Result<Doc> {
        // Lambda syntax is complex (-> vs lambda, {} vs do-end)
        // Use source extraction to preserve original style
        let mut docs: Vec<Doc> = Vec::with_capacity(3);

        // Leading comments
        let leading = format_leading_comments(ctx, node.location.start_line);
        if !leading.is_empty() {
            docs.push(leading);
        }

        // Extract source
        if let Some(source_text) = ctx.extract_source(node) {
            docs.push(text(source_text));
        }

        // Mark internal comments as emitted
        mark_comments_in_range_emitted(ctx, node.location.start_line, node.location.end_line);

        // Trailing comment
        let trailing = format_trailing_comment(ctx, node.location.end_line);
        if !trailing.is_empty() {
            docs.push(trailing);
        }

        Ok(concat(docs))
    }
}

/// Formats method call
fn format_call(node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    // Leading comments
    let mut docs: Vec<Doc> = Vec::with_capacity(4);
    let leading = format_leading_comments(ctx, node.location.start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // Check if this call has a block (last child is BlockNode)
    let has_block = node
        .children
        .last()
        .map(|c| matches!(c.node_type, NodeType::BlockNode))
        .unwrap_or(false);

    if !has_block {
        // Simple call - use source extraction with chain reformatting.
        //
        // A CallNode that carries a heredoc argument (e.g.
        // `query(<<~SQL)\n  …\nSQL`) reports its end_offset past the
        // heredoc terminator's trailing newline. Leaving that newline in
        // the emitted `Doc::Text` combines with the later `hardline + end`
        // or inter-statement hardline to produce a spurious blank line.
        // Strip at most one trailing newline so this doesn't happen; using
        // the full `trim_end` here would instead eat a blank separator line
        // that legitimately belongs between statements.
        if let Some(source_text) = ctx.extract_source(node) {
            let base_indent = line_leading_indent(ctx.source(), node.location.start_offset);
            let reformatted = reformat_chain_lines(
                source_text,
                base_indent,
                ctx.config().formatting.indent_width,
            );
            let trimmed = strip_one_trailing_newline(&reformatted);
            docs.push(text(trimmed.to_string()));
        }

        // Mark comments in this range as emitted (they're in source extraction)
        mark_comments_in_range_emitted(ctx, node.location.start_line, node.location.end_line);

        // Trailing comment
        let trailing = format_trailing_comment(ctx, node.location.end_line);
        if !trailing.is_empty() {
            docs.push(trailing);
        }

        return Ok(concat(docs));
    }

    // Has block - need to handle specially
    let block_node = node.children.last().unwrap();
    let block_style = detect_block_style(block_node, ctx);

    // Emit the call part (receiver.method(args)) from source with chain
    // reformatting. Track whether reformatting actually fired so the block
    // body can be re-aligned to match the chain's new depth.
    let call_end_offset = block_node.location.start_offset;
    let chain_reformatted = if let Some(call_text) = ctx
        .source()
        .get(node.location.start_offset..call_end_offset)
    {
        let base_indent = line_leading_indent(ctx.source(), node.location.start_offset);
        let reformatted = reformat_chain_lines(
            call_text.trim_end(),
            base_indent,
            ctx.config().formatting.indent_width,
        );
        let changed = matches!(reformatted, Cow::Owned(_));
        docs.push(text(reformatted));
        changed
    } else {
        false
    };

    // Mark comments in the call part (before block) as emitted
    // This includes trailing comments that are part of the extracted source
    mark_comments_in_range_emitted(
        ctx,
        node.location.start_line,
        block_node.location.start_line,
    );

    // Format the block. When the receiver's chain was re-indented, the
    // `do`-line ends up one level below `base_indent` instead of at
    // `base_indent` itself, so the default `indent(body)` wrap inside the
    // block formatter now places the body *at* the chain depth rather than
    // one level below it (and the `end` keyword floats up to `base_indent`).
    // Push both down with `Align` so the `do…end` body is indented relative
    // to the chain's last line, matching what a human would write.
    let block_doc = match block_style {
        BlockStyle::DoEnd => format_do_end_block(block_node, ctx, registry)?,
        BlockStyle::Braces => format_brace_block(block_node, ctx, registry)?,
    };
    if chain_reformatted {
        docs.push(align(ctx.config().formatting.indent_width, block_doc));
    } else {
        docs.push(block_doc);
    }

    Ok(concat(docs))
}

/// Detect whether block uses do...end or { } style
fn detect_block_style(block_node: &Node, ctx: &FormatContext) -> BlockStyle {
    if let Some(first_char) = ctx
        .source()
        .get(block_node.location.start_offset..block_node.location.start_offset + 1)
    {
        if first_char == "{" {
            return BlockStyle::Braces;
        }
    }
    BlockStyle::DoEnd
}

/// Formats do...end style block
fn format_do_end_block(
    block_node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    docs.push(text(" do"));

    // Emit block parameters if present (|x, y|)
    if let Some(params) = extract_block_parameters(block_node, ctx) {
        docs.push(text(" "));
        docs.push(text(params));
    }

    // Trailing comment on same line as do |...|
    let trailing = format_trailing_comment(ctx, block_node.location.start_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    // Find and emit the body (StatementsNode or BeginNode among children)
    for child in &block_node.children {
        match &child.node_type {
            NodeType::StatementsNode => {
                let body_doc = format_statements(child, ctx, registry)?;
                docs.push(indent(concat(vec![hardline(), body_doc])));
                break;
            }
            NodeType::BeginNode => {
                // Block with rescue/else/ensure needs the clause keywords at
                // the block opener's indent level, not at the body indent.
                if super::begin::is_implicit_begin_with_clauses(child, ctx) {
                    docs.push(super::begin::format_implicit_begin_body(
                        child, ctx, registry,
                    )?);
                } else {
                    let body_doc = format_child(child, ctx, registry)?;
                    docs.push(indent(concat(vec![hardline(), body_doc])));
                }
                break;
            }
            _ => {
                // Skip parameter nodes
            }
        }
    }

    // Emit any standalone comments between the last body statement and `end`.
    //
    // Without this the orphan comments inside a `do…end` block (e.g. the
    // commented-out config stanzas in a generated `spec_helper.rb`) never
    // get claimed by any `format_leading_comments` call, fall through to
    // `format_remaining_comments` at the end of the file, and get emitted
    // *after* the block's own `end` — producing `end# comment…` with no
    // separator and dropping the body indent.
    let comments_before_end = format_comments_before_end(
        ctx,
        block_node.location.start_line,
        block_node.location.end_line,
    );
    if !comments_before_end.is_empty() {
        docs.push(indent(comments_before_end));
    }

    // Emit 'end'
    docs.push(hardline());
    docs.push(text("end"));

    // Trailing comment on end line
    let end_trailing = format_trailing_comment(ctx, block_node.location.end_line);
    if !end_trailing.is_empty() {
        docs.push(end_trailing);
    }

    Ok(concat(docs))
}

/// Formats { } style block
fn format_brace_block(
    block_node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let is_multiline = block_node.location.start_line != block_node.location.end_line;

    if is_multiline {
        format_multiline_brace_block(block_node, ctx, registry)
    } else {
        format_inline_brace_block(block_node, ctx)
    }
}

/// Formats multiline brace block
fn format_multiline_brace_block(
    block_node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    docs.push(text(" {"));

    // Emit block parameters if present
    if let Some(params) = extract_block_parameters(block_node, ctx) {
        docs.push(text(" "));
        docs.push(text(params));
    }

    // Emit body
    for child in &block_node.children {
        if matches!(child.node_type, NodeType::StatementsNode) {
            let body_doc = format_statements(child, ctx, registry)?;
            docs.push(indent(concat(vec![hardline(), body_doc])));
            break;
        }
    }

    docs.push(hardline());
    docs.push(text("}"));

    // Trailing comment
    let trailing = format_trailing_comment(ctx, block_node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Formats inline brace block
fn format_inline_brace_block(block_node: &Node, ctx: &mut FormatContext) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(3);

    docs.push(text(" "));

    // Extract from source to preserve spacing
    if let Some(source_text) = ctx.extract_source(block_node) {
        docs.push(text(source_text));
    }

    // Mark internal comments as emitted
    mark_comments_in_range_emitted(
        ctx,
        block_node.location.start_line,
        block_node.location.end_line,
    );

    // Trailing comment
    let trailing = format_trailing_comment(ctx, block_node.location.end_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    Ok(concat(docs))
}

/// Extract block parameters (|x, y|) from block node
fn extract_block_parameters(block_node: &Node, ctx: &FormatContext) -> Option<String> {
    let source = ctx.source();
    if source.is_empty() {
        return None;
    }

    let block_source =
        source.get(block_node.location.start_offset..block_node.location.end_offset)?;

    // Only look at the first line of the block for parameters
    let first_line = block_source.lines().next()?;

    // Find |...| pattern in the first line only
    let pipe_start = first_line.find('|')?;
    let rest = &first_line[pipe_start + 1..];
    let pipe_end = rest.find('|')?;

    Some(first_line[pipe_start..=pipe_start + 1 + pipe_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location};
    use crate::config::Config;
    use crate::doc::Printer;
    use std::collections::HashMap;

    fn make_call_node(
        children: Vec<Node>,
        start_offset: usize,
        end_offset: usize,
        start_line: usize,
        end_line: usize,
    ) -> Node {
        Node {
            node_type: NodeType::CallNode,
            location: Location::new(start_line, 0, end_line, 0, start_offset, end_offset),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_call() {
        let config = Config::default();
        let source = "puts 'hello'";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_call_node(Vec::new(), 0, 12, 1, 1);
        ctx.collect_comments(&node);

        let rule = CallRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "puts 'hello'");
    }

    #[test]
    fn test_call_with_do_block() {
        let config = Config::default();
        let source = "items.each do |item|\n  puts item\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let block_body = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(2, 2, 2, 11, 23, 32),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let block = Node {
            node_type: NodeType::BlockNode,
            location: Location::new(1, 11, 3, 3, 11, 36),
            children: vec![block_body],
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_call_node(vec![block], 0, 36, 1, 3);
        ctx.collect_comments(&node);

        let rule = CallRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("items.each do"));
        assert!(result.contains("end"));
    }
}

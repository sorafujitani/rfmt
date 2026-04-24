//! Shared helpers for body-with-end constructs
//!
//! This module provides common formatting logic for Ruby constructs
//! that have a header, optional body, and `end` keyword (class, module, def).

use crate::ast::Node;
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_child, format_comments_before_end, format_leading_comments, format_trailing_comment,
    is_structural_node, mark_comments_in_range_emitted,
};

use super::begin::{format_implicit_begin_body, is_implicit_begin_with_clauses};

/// Configuration for formatting a body-with-end construct.
pub struct BodyEndConfig<'a> {
    /// The keyword (e.g., "class", "module", "def")
    pub keyword: &'static str,
    /// The node being formatted
    pub node: &'a Node,
    /// Function to build the header after the keyword
    pub header_builder: Box<dyn Fn(&'a Node) -> Vec<Doc> + 'a>,
    /// Optional filter for which children are considered structural (skipped in body)
    pub skip_same_line_children: bool,
}

/// Formats a body-with-end construct (class, module, def).
///
/// This handles the common pattern of:
/// 1. Leading comments
/// 2. Header line (keyword + name + optional extras)
/// 3. Trailing comment on header line
/// 4. Indented body (skipping structural nodes)
/// 5. Comments before end
/// 6. End keyword
/// 7. Trailing comment on end line
pub fn format_body_end(
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    config: BodyEndConfig,
) -> Result<Doc> {
    let mut docs: Vec<Doc> = Vec::with_capacity(8);

    let start_line = config.node.location.start_line;
    let end_line = config.node.location.end_line;

    // 1. Leading comments before definition
    let leading = format_leading_comments(ctx, start_line);
    if !leading.is_empty() {
        docs.push(leading);
    }

    // Single-line form: `def foo = expr` (endless), `def foo; body; end`,
    // `class Foo; end`, etc. Emit the source verbatim instead of forcing
    // a multi-line `def ... end` layout. This preserves Ruby 3+ endless
    // methods and the `Error < StandardError; end` exception-hierarchy
    // idiom that is pervasive in Rails code.
    if start_line == end_line {
        if let Some(source_text) = ctx.extract_source(config.node) {
            docs.push(text(source_text.to_string()));
            mark_comments_in_range_emitted(ctx, start_line, end_line);

            let trailing = format_trailing_comment(ctx, end_line);
            if !trailing.is_empty() {
                docs.push(trailing);
            }
            return Ok(concat(docs));
        }
    }

    // Body children (filtered) — reused by both the multi-line header path
    // below and the normal path further down.
    let body_children: Vec<&Node> = config
        .node
        .children
        .iter()
        .filter(|c| {
            if config.skip_same_line_children && c.location.start_line == start_line {
                return false;
            }
            !is_structural_node(c)
        })
        .collect();

    // Multi-line header form: `def foo(a, # c1\n  b, # c2\n  c) # c3`.
    //
    // Rebuilding the header from `parameters_text` + a reconstructed `)`
    // drops the closing line's trailing comment and, worse, lets comments
    // that the params slice already contains be re-picked up by the later
    // `format_trailing_comment` + `format_leading_comments` calls. The net
    // effect is that the first inline comment is duplicated, the rest are
    // moved into the body, and the file stops being idempotent because the
    // relocated comments keep accumulating every format pass. Detect this
    // case early and emit the header verbatim from source.
    let params_text = config.node.metadata.get("parameters_text");
    let header_is_multiline = params_text.is_some_and(|t| t.contains('\n'));
    if header_is_multiline {
        let header_end_line = body_children
            .first()
            .map(|c| c.location.start_line.saturating_sub(1))
            .unwrap_or(start_line);
        if header_end_line >= start_line {
            let header_end_offset = line_end_offset(ctx.source(), header_end_line);
            if let Some(header_src) = ctx
                .source()
                .get(config.node.location.start_offset..header_end_offset)
            {
                docs.push(text(header_src.to_string()));
                mark_comments_in_range_emitted(ctx, start_line, header_end_line + 1);

                // Body + "end" (same as the normal path; no header trailing
                // comment — it's already baked into the source slice above).
                push_body_and_end(
                    &mut docs,
                    ctx,
                    registry,
                    &body_children,
                    start_line,
                    end_line,
                )?;
                return Ok(concat(docs));
            }
        }
    }

    // 2. Build header: "keyword ..."
    let mut header_parts: Vec<Doc> = vec![text(config.keyword), text(" ")];
    header_parts.extend((config.header_builder)(config.node));
    docs.push(concat(header_parts));

    // 3. Trailing comment on definition line
    let trailing = format_trailing_comment(ctx, start_line);
    if !trailing.is_empty() {
        docs.push(trailing);
    }

    // 4. Body (children), skipping structural nodes — already collected above.

    // Special case: body is an implicit BeginNode carrying rescue/else/ensure.
    // In that case the clause keywords must align with the opener, not with
    // the body statements — so we split the body and clause emission instead
    // of wrapping everything in a single `indent(...)`.
    if body_children.len() == 1 && is_implicit_begin_with_clauses(body_children[0], ctx) {
        docs.push(format_implicit_begin_body(body_children[0], ctx, registry)?);
    } else if !body_children.is_empty() {
        let mut body_docs: Vec<Doc> = Vec::with_capacity(body_children.len() * 2);
        for child in &body_children {
            let child_doc = format_child(child, ctx, registry)?;
            body_docs.push(hardline());
            body_docs.push(child_doc);
        }
        docs.push(indent(concat(body_docs)));
    }

    // 5. Comments before end
    let comments_before_end = format_comments_before_end(ctx, start_line, end_line);
    if !comments_before_end.is_empty() {
        docs.push(indent(comments_before_end));
    }

    // 6. Add newline before end
    docs.push(hardline());

    // 7. End keyword
    docs.push(text("end"));

    // 8. Trailing comment on end line
    let end_trailing = format_trailing_comment(ctx, end_line);
    if !end_trailing.is_empty() {
        docs.push(end_trailing);
    }

    Ok(concat(docs))
}

/// Returns the byte offset of the character immediately past the end of the
/// given 1-based line (i.e. the index of the trailing `\n`, or `source.len()`
/// if the line has no trailing newline).
fn line_end_offset(source: &str, line: usize) -> usize {
    let mut current = 1;
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            if current == line {
                return i;
            }
            current += 1;
        }
    }
    source.len()
}

/// Emits the body (possibly an implicit BeginNode with rescue clauses) and the
/// trailing `end` keyword. Extracted so that both the standard
/// header-reconstruction path and the multi-line source-extraction path can
/// share the emission logic.
fn push_body_and_end(
    docs: &mut Vec<Doc>,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
    body_children: &[&Node],
    start_line: usize,
    end_line: usize,
) -> Result<()> {
    if body_children.len() == 1 && is_implicit_begin_with_clauses(body_children[0], ctx) {
        docs.push(format_implicit_begin_body(body_children[0], ctx, registry)?);
    } else if !body_children.is_empty() {
        let mut body_docs: Vec<Doc> = Vec::with_capacity(body_children.len() * 2);
        for child in body_children {
            let child_doc = format_child(child, ctx, registry)?;
            body_docs.push(hardline());
            body_docs.push(child_doc);
        }
        docs.push(indent(concat(body_docs)));
    }

    let comments_before_end = format_comments_before_end(ctx, start_line, end_line);
    if !comments_before_end.is_empty() {
        docs.push(indent(comments_before_end));
    }

    docs.push(hardline());
    docs.push(text("end"));

    let end_trailing = format_trailing_comment(ctx, end_line);
    if !end_trailing.is_empty() {
        docs.push(end_trailing);
    }

    Ok(())
}

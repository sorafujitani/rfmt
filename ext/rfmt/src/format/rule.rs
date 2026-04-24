//! FormatRule trait and helper functions
//!
//! This module defines the core FormatRule trait that all formatting rules implement,
//! along with shared helper functions for common formatting patterns.

use crate::ast::{CommentType, Node};
use crate::doc::{concat, hardline, leading_comment, literalline, text, trailing_comment, Doc};
use crate::error::Result;

use super::context::FormatContext;
use super::registry::RuleRegistry;

/// Trait for formatting rules.
///
/// Each rule handles a specific node type (or set of node types) and produces
/// a Doc IR representation of that node.
///
/// Rules are stateless and can be shared across multiple formatting contexts.
pub trait FormatRule: Send + Sync {
    /// Formats a node and returns the Doc IR.
    ///
    /// # Arguments
    /// * `node` - The AST node to format
    /// * `ctx` - The formatting context with source, config, and comment tracking
    /// * `registry` - The rule registry for recursive formatting
    ///
    /// # Returns
    /// A Doc IR representing the formatted node
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc>;
}

/// Formats a child node by dispatching to the appropriate rule.
///
/// This is the primary way to recursively format child nodes within rules.
pub fn format_child(child: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
    let rule = registry.get_rule(&child.node_type);
    rule.format(child, ctx, registry)
}

/// Boxed rule type for dynamic dispatch.
pub type BoxedRule = Box<dyn FormatRule>;

/// Lightweight comment reference using index instead of cloning.
#[derive(Clone, Copy)]
struct CommentRef {
    idx: usize,
    start_line: usize,
    end_line: usize,
    is_block: bool,
}

/// Emits a single leading comment at the configured indent context.
///
/// `=begin ... =end` (Prism's `EmbDocComment`) must start in column 0;
/// nesting it under an `indent(...)` wrapper would emit `=begin` at the
/// body indent, which is a *syntax error*. Use `literalline` to break out
/// of the current indent so the `=begin` and `=end` markers — and every
/// intermediate line — land at column 0. The following `hardline` then
/// restores the normal indented flow for the code that comes after.
///
/// Prism's location slice for an embdoc includes the trailing newline that
/// sits after `=end`; strip it so that our own `hardline` doesn't produce
/// a double line break.
fn push_leading_comment(docs: &mut Vec<Doc>, comment_text: &str, is_block: bool) {
    if is_block {
        let body = comment_text
            .strip_suffix('\n')
            .and_then(|s| s.strip_suffix('\r').or(Some(s)))
            .unwrap_or(comment_text);
        docs.push(literalline());
        docs.push(text(body.to_string()));
        docs.push(hardline());
    } else {
        docs.push(leading_comment(comment_text, true));
    }
}

/// Formats leading comments before a given line.
///
/// This helper collects all unemitted comments that appear before the given line,
/// formats them with hardlines, and marks them as emitted.
///
/// # Arguments
/// * `ctx` - The formatting context
/// * `line` - The line number to get comments before
///
/// # Returns
/// A Doc containing all leading comments with proper line breaks
pub fn format_leading_comments(ctx: &mut FormatContext, line: usize) -> Doc {
    // Collect lightweight refs while borrowing immutably
    let comment_refs: Vec<CommentRef> = ctx
        .get_comment_indices_before(line)
        .filter_map(|idx| {
            ctx.get_comment(idx).map(|c| CommentRef {
                idx,
                start_line: c.location.start_line,
                end_line: c.location.end_line,
                is_block: matches!(c.comment_type, CommentType::Block),
            })
        })
        .collect();

    if comment_refs.is_empty() {
        return Doc::Empty;
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(comment_refs.len() * 2);
    let mut last_end_line: Option<usize> = None;
    let mut indices_to_mark: Vec<usize> = Vec::with_capacity(comment_refs.len());

    for cref in &comment_refs {
        // Preserve blank lines between comments
        if let Some(prev_end) = last_end_line {
            let gap = cref.start_line.saturating_sub(prev_end);
            for _ in 1..gap {
                docs.push(hardline());
            }
        }

        if let Some(comment) = ctx.get_comment(cref.idx) {
            push_leading_comment(&mut docs, &comment.text, cref.is_block);
        }
        last_end_line = Some(cref.end_line);
        indices_to_mark.push(cref.idx);
    }

    // Mark comments as emitted in batch
    ctx.mark_comments_emitted(indices_to_mark);

    // Add blank line after comments if there's a gap before the node
    if let Some(last_end) = last_end_line {
        if line > last_end + 1 {
            docs.push(hardline());
        }
    }

    concat(docs)
}

/// Formats a trailing comment on the same line.
///
/// # Arguments
/// * `ctx` - The formatting context
/// * `line` - The line number to get trailing comments for
///
/// # Returns
/// A Doc containing the trailing comment, or Empty if none
pub fn format_trailing_comment(ctx: &mut FormatContext, line: usize) -> Doc {
    // Collect indices while borrowing immutably
    let indices: Vec<usize> = ctx.get_trailing_comment_indices(line).collect();

    if indices.is_empty() {
        return Doc::Empty;
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(indices.len());

    for &idx in &indices {
        if let Some(comment) = ctx.get_comment(idx) {
            docs.push(trailing_comment(&comment.text));
        }
    }

    // Mark comments as emitted in batch
    ctx.mark_comments_emitted(indices);

    concat(docs)
}

/// Formats comments that appear before the `end` keyword of a construct.
///
/// This is used for comments inside class/module/def bodies that appear
/// on standalone lines before the closing `end`.
///
/// # Arguments
/// * `ctx` - The formatting context
/// * `start_line` - The start line of the construct
/// * `end_line` - The end line of the construct (where `end` appears)
///
/// # Returns
/// A Doc containing the formatted comments
pub fn format_comments_before_end(
    ctx: &mut FormatContext,
    start_line: usize,
    end_line: usize,
) -> Doc {
    // Collect indices for comments in range
    let indices: Vec<usize> = ctx
        .get_comment_indices_in_range(start_line + 1, end_line)
        .collect();

    if indices.is_empty() {
        return Doc::Empty;
    }

    // Filter to only standalone comments
    let standalone_refs: Vec<CommentRef> = indices
        .iter()
        .filter_map(|&idx| {
            ctx.get_comment(idx).and_then(|c| {
                if ctx.is_standalone_comment(c) && c.location.end_line < end_line {
                    Some(CommentRef {
                        idx,
                        start_line: c.location.start_line,
                        end_line: c.location.end_line,
                        is_block: matches!(c.comment_type, CommentType::Block),
                    })
                } else {
                    None
                }
            })
        })
        .collect();

    if standalone_refs.is_empty() {
        return Doc::Empty;
    }

    let mut docs: Vec<Doc> = vec![hardline()];
    // Preserve the blank line that separated the body's last content from
    // the first trailing comment. Without this, a construct like
    //
    //   def foo
    //     body
    //                          <- blank line
    //     # trailing annotation
    //   end
    //
    // collapses to `body\n# trailing annotation\nend`. Detect the case
    // heuristically: if the source line immediately above the first
    // standalone comment is blank, emit an extra hardline.
    if let Some(first) = standalone_refs.first() {
        if first.start_line > 1 && is_line_blank(ctx.source(), first.start_line - 1) {
            docs.push(hardline());
        }
    }
    let mut last_end_line: Option<usize> = None;
    let mut indices_to_mark: Vec<usize> = Vec::with_capacity(standalone_refs.len());

    let last_idx = standalone_refs.len().saturating_sub(1);
    for (i, cref) in standalone_refs.iter().enumerate() {
        // Preserve blank lines between comments
        if let Some(prev_end) = last_end_line {
            let gap = cref.start_line.saturating_sub(prev_end);
            for _ in 1..gap {
                docs.push(hardline());
            }
        }

        if let Some(comment) = ctx.get_comment(cref.idx) {
            // The caller always emits its own `hardline + "end"` right after
            // us, so the last comment must *not* also emit its own trailing
            // newline — doing so produces a spurious blank line between the
            // comment and the `end` keyword.
            let hard_line_after = i != last_idx;
            if cref.is_block {
                let body = comment
                    .text
                    .strip_suffix('\n')
                    .and_then(|s| s.strip_suffix('\r').or(Some(s)))
                    .unwrap_or(&comment.text)
                    .to_string();
                docs.push(literalline());
                docs.push(text(body));
                if hard_line_after {
                    docs.push(hardline());
                }
            } else {
                docs.push(leading_comment(&comment.text, hard_line_after));
            }
        }
        last_end_line = Some(cref.end_line);
        indices_to_mark.push(cref.idx);
    }

    // Mark comments as emitted in batch
    ctx.mark_comments_emitted(indices_to_mark);

    concat(docs)
}

/// Formats remaining comments at the end of the file.
///
/// This should be called after all nodes have been formatted to emit
/// any comments that weren't attached to specific nodes.
///
/// # Arguments
/// * `ctx` - The formatting context
/// * `last_code_line` - The last line of code in the file
///
/// # Returns
/// A Doc containing all remaining comments
pub fn format_remaining_comments(ctx: &mut FormatContext, last_code_line: usize) -> Doc {
    // Collect remaining comment indices and their line info
    let comment_refs: Vec<CommentRef> = ctx
        .get_remaining_comment_indices()
        .filter_map(|idx| {
            ctx.get_comment(idx).map(|c| CommentRef {
                idx,
                start_line: c.location.start_line,
                end_line: c.location.end_line,
                is_block: matches!(c.comment_type, CommentType::Block),
            })
        })
        .collect();

    if comment_refs.is_empty() {
        return Doc::Empty;
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(comment_refs.len() * 2);
    let mut last_end_line = last_code_line;
    let mut is_first = true;
    let mut indices_to_mark: Vec<usize> = Vec::with_capacity(comment_refs.len());

    for cref in &comment_refs {
        // Preserve blank lines. On the first iteration we must emit *at
        // least one* hardline to separate the first remaining comment from
        // the main document's last token (otherwise an orphan comment whose
        // `start_line <= last_code_line` would concatenate onto whatever
        // ended the output — producing e.g. `end# comment…` when a block's
        // internal comments fall through to this tail handler). Round-tripping
        // the already-formatted output must still be idempotent, so we
        // cap the emission at the number of line breaks visible in the
        // source: 1 for an adjacent comment, N for N-1 blank lines above it.
        let gap = cref.start_line.saturating_sub(last_end_line);

        if is_first {
            let hardlines_to_emit = gap.max(1);
            for _ in 0..hardlines_to_emit {
                docs.push(hardline());
            }
        } else if gap > 0 {
            for _ in 0..gap.max(1) {
                docs.push(hardline());
            }
        }

        if let Some(comment) = ctx.get_comment(cref.idx) {
            if cref.is_block {
                let body = comment
                    .text
                    .strip_suffix('\n')
                    .and_then(|s| s.strip_suffix('\r').or(Some(s)))
                    .unwrap_or(&comment.text)
                    .to_string();
                docs.push(literalline());
                docs.push(text(body));
            } else {
                docs.push(leading_comment(&comment.text, false));
            }
        }
        last_end_line = cref.end_line;
        is_first = false;
        indices_to_mark.push(cref.idx);
    }

    // Mark comments as emitted in batch
    ctx.mark_comments_emitted(indices_to_mark);

    concat(docs)
}

/// Formats a statements node as a sequence of children with proper line spacing.
///
/// This is a shared helper used by multiple formatting rules (if_unless, case,
/// begin, call, loops) to format StatementsNode children consistently.
///
/// # Arguments
/// * `node` - The StatementsNode to format
/// * `ctx` - The formatting context
/// * `registry` - The rule registry for recursive formatting
///
/// # Returns
/// A Doc containing all statements with proper line breaks between them
pub fn format_statements(
    node: &Node,
    ctx: &mut FormatContext,
    registry: &RuleRegistry,
) -> Result<Doc> {
    if node.children.is_empty() {
        return Ok(Doc::Empty);
    }

    let mut docs: Vec<Doc> = Vec::with_capacity(node.children.len() * 2);

    for (i, child) in node.children.iter().enumerate() {
        let child_doc = format_child(child, ctx, registry)?;
        docs.push(child_doc);

        // Add newlines between statements. A pure line-number diff would add
        // a blank-line hardline whenever two consecutive statements sit on
        // lines that are more than 1 apart — but that gap may be occupied
        // by one or more standalone comments, each of which gets emitted as
        // a leading comment of the next statement and already supplies its
        // own line break. Subtract the lines consumed by comments so we
        // only preserve *actually* blank lines between statements.
        if let Some(next_child) = node.children.get(i + 1) {
            let current_end_line = child.location.end_line;
            let next_start_line = next_child.location.start_line;
            let line_diff = next_start_line.saturating_sub(current_end_line);

            docs.push(hardline());

            if line_diff > 1 {
                let (comment_lines_in_gap, gap_has_block): (usize, bool) = ctx
                    .get_comment_indices_in_range(current_end_line + 1, next_start_line)
                    .filter_map(|idx| ctx.get_comment(idx).cloned())
                    .fold((0usize, false), |(lines, had_block), c| {
                        let span = c.location.end_line.saturating_sub(c.location.start_line) + 1;
                        let is_block = matches!(c.comment_type, CommentType::Block);
                        (lines + span, had_block || is_block)
                    });
                // `line_diff - 1` is the count of lines strictly between the
                // two statements. Subtract comment-occupied lines to get the
                // count of truly blank lines.
                let mut blank_lines = line_diff
                    .saturating_sub(1)
                    .saturating_sub(comment_lines_in_gap);
                // A block comment (`=begin/=end`) is emitted via
                // `literalline + text + hardline`. The leading `literalline`
                // already supplies one line break, so the normal
                // blank-line hardline added here would produce one extra
                // blank line above `=begin`. Deduct one.
                if gap_has_block && blank_lines > 0 {
                    blank_lines -= 1;
                }
                if blank_lines >= 1 {
                    docs.push(hardline());
                }
            }
        }
    }

    Ok(concat(docs))
}

/// Returns the number of leading space/tab characters on the line containing `offset`.
///
/// The source text extracted by `FormatContext::extract_source` starts at the node's
/// offset and does not include the whitespace that precedes the first line in the
/// original source. `Doc::Text` is printed verbatim without re-indenting embedded
/// newlines, so any reformatting that emits a multi-line string must include the
/// original leading indent itself.
pub fn line_leading_indent(source: &str, offset: usize) -> usize {
    let offset = offset.min(source.len());
    let line_start = source[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
    source.as_bytes()[line_start..offset]
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

/// Reformats multiline method chain text with indented style.
///
/// Converts aligned method chains to indented style:
/// - First line is kept as-is (trimmed at end)
/// - Subsequent lines starting with `.` or `&.` are re-indented to
///   `base_indent + indent_width` spaces
///
/// `base_indent` is the column at which the first line starts in the original source
/// (obtain via `line_leading_indent`). Because `Doc::Text` is printed verbatim without
/// re-indenting embedded newlines, this indent must be included in the returned string.
///
/// Returns `Cow::Borrowed` when no transformation is needed to avoid allocation.
///
/// # Example
/// ```text
/// Input (base_indent=4, indent_width=2):
///   "foo.bar\n                  .baz"
/// Output:
///   "foo.bar\n      .baz"
/// ```
pub fn reformat_chain_lines(
    source_text: &str,
    base_indent: usize,
    indent_width: usize,
) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;

    let lines: Vec<&str> = source_text.lines().collect();
    if lines.len() <= 1 {
        return Cow::Borrowed(source_text);
    }

    // Skip reformatting when the first line opens a new scope (a `{` brace
    // lambda, a `do` block, or a `do |params|` block). The `.method` lines
    // that follow are chain continuations *inside the block body*, not of
    // the outer call — re-indenting them relative to the outer
    // `base_indent` collapses the nested chain one level to the left and
    // breaks the visual structure of the block body.
    //
    // This deliberately keeps the reformat conservative: for a top-level
    // `User.active.where(...)` chain, line 1 ends with an identifier so
    // we still rewrite aligned → indented as PR #100 intended.
    let first_line = lines[0].trim_end();
    if first_line.ends_with('{') || first_line.ends_with(" do") || first_line.ends_with('|') {
        return Cow::Borrowed(source_text);
    }

    // Check if there are actual chain continuation lines (. or &.)
    let has_chain = lines[1..].iter().any(|l| {
        let t = l.trim_start();
        t.starts_with('.') || t.starts_with("&.")
    });

    if !has_chain {
        return Cow::Borrowed(source_text);
    }

    // Build the indented chain with pre-allocated capacity
    let chain_indent = " ".repeat(base_indent + indent_width);
    let mut result = String::with_capacity(source_text.len());
    result.push_str(lines[0].trim_end());

    for line in &lines[1..] {
        result.push('\n');
        let trimmed = line.trim();
        if trimmed.starts_with('.') || trimmed.starts_with("&.") {
            result.push_str(&chain_indent);
            result.push_str(trimmed);
        } else {
            // Non-chain continuation (e.g., heredoc content): preserve as-is
            result.push_str(line);
        }
    }

    Cow::Owned(result)
}

/// Returns true when the given 1-based `line` in `source` contains only
/// whitespace (or is empty). Returns false for any line that has code or
/// a comment.
fn is_line_blank(source: &str, line: usize) -> bool {
    let mut current = 1usize;
    let mut line_start = 0usize;
    for (i, b) in source.bytes().enumerate() {
        if current == line {
            let end = source[i..].find('\n').map_or(source.len(), |n| i + n);
            return source[line_start..end]
                .bytes()
                .all(|b| b == b' ' || b == b'\t' || b == b'\r');
        }
        if b == b'\n' {
            current += 1;
            line_start = i + 1;
        }
    }
    false
}

/// Removes at most one trailing `\n` (optionally preceded by a single `\r`)
/// from `s`. Spaces, tabs, and any additional preceding newlines are
/// preserved.
///
/// This is used by source-extracting rules (FallbackRule, CallRule,
/// VariableWriteRule) to strip the terminator-line newline that Prism
/// includes in node extents for constructs like
/// `foo(<<~HEREDOC)\n…\nHEREDOC\n`. A full `trim_end` would also eat any
/// blank separator line that happens to fall inside the node's range,
/// collapsing the spacing between consecutive statements.
pub fn strip_one_trailing_newline(s: &str) -> &str {
    if let Some(rest) = s.strip_suffix('\n') {
        rest.strip_suffix('\r').unwrap_or(rest)
    } else {
        s
    }
}

/// Marks comments within a line range as emitted.
///
/// This is used when source text is extracted directly, as any comments
/// within the extracted range are included in the output.
///
/// # Arguments
/// * `ctx` - The formatting context
/// * `start_line` - The start line of the range
/// * `end_line` - The end line of the range
pub fn mark_comments_in_range_emitted(ctx: &mut FormatContext, start_line: usize, end_line: usize) {
    let indices: Vec<usize> = ctx
        .get_comment_indices_in_range(start_line, end_line)
        .collect();
    ctx.mark_comments_emitted(indices);
}

/// Checks if a node is a structural node (part of definition syntax, not body).
///
/// Structural nodes are parts of class/module/method definitions that should
/// not be emitted as body content (e.g., constant names, parameter nodes).
pub fn is_structural_node(node: &Node) -> bool {
    use crate::ast::NodeType;

    matches!(
        node.node_type,
        NodeType::ConstantReadNode
            | NodeType::ConstantWriteNode
            | NodeType::ConstantPathNode
            | NodeType::RequiredParameterNode
            | NodeType::OptionalParameterNode
            | NodeType::RestParameterNode
            | NodeType::KeywordParameterNode
            | NodeType::RequiredKeywordParameterNode
            | NodeType::OptionalKeywordParameterNode
            | NodeType::KeywordRestParameterNode
            | NodeType::BlockParameterNode
            | NodeType::ForwardingParameterNode
            | NodeType::NoKeywordsParameterNode
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        Comment, CommentPosition, CommentType, FormattingInfo, Location, Node, NodeType,
    };
    use crate::config::Config;
    use std::collections::HashMap;

    fn make_comment(text: &str, line: usize, start_offset: usize) -> Comment {
        Comment {
            text: text.to_string(),
            location: Location::new(
                line,
                0,
                line,
                text.len(),
                start_offset,
                start_offset + text.len(),
            ),
            comment_type: CommentType::Line,
            position: CommentPosition::Leading,
        }
    }

    fn make_node_with_comments(comments: Vec<Comment>) -> Node {
        Node {
            node_type: NodeType::ProgramNode,
            location: Location::new(1, 0, 10, 0, 0, 100),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments,
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_format_leading_comments() {
        let config = Config::default();
        let source = "# comment\nclass Foo\nend";
        let mut ctx = FormatContext::new(&config, source);

        let comment = make_comment("# comment", 1, 0);
        let node = make_node_with_comments(vec![comment]);
        ctx.collect_comments(&node);

        let doc = format_leading_comments(&mut ctx, 5);
        assert!(!matches!(doc, Doc::Empty));
    }

    #[test]
    fn test_format_trailing_comment() {
        let config = Config::default();
        let source = "code # trailing";
        let mut ctx = FormatContext::new(&config, source);

        let comment = Comment {
            text: "# trailing".to_string(),
            location: Location::new(1, 5, 1, 15, 5, 15),
            comment_type: CommentType::Line,
            position: CommentPosition::Trailing,
        };
        let node = make_node_with_comments(vec![comment]);
        ctx.collect_comments(&node);

        let doc = format_trailing_comment(&mut ctx, 1);
        assert!(!matches!(doc, Doc::Empty));
    }

    #[test]
    fn test_is_structural_node() {
        let structural_node = Node {
            node_type: NodeType::ConstantReadNode,
            location: Location::new(1, 0, 1, 3, 0, 3),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let non_structural_node = Node {
            node_type: NodeType::CallNode,
            location: Location::new(1, 0, 1, 10, 0, 10),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        assert!(is_structural_node(&structural_node));
        assert!(!is_structural_node(&non_structural_node));
    }

    #[test]
    fn test_format_child() {
        let config = Config::default();
        let source = "puts 'hello'";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = Node {
            node_type: NodeType::CallNode,
            location: Location::new(1, 0, 1, 12, 0, 12),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        ctx.collect_comments(&node);

        let doc = format_child(&node, &mut ctx, &registry).unwrap();
        assert!(!matches!(doc, Doc::Empty));
    }

    #[test]
    fn test_reformat_chain_lines_single_line() {
        let input = "foo.bar.baz";
        let result = reformat_chain_lines(input, 0, 2);
        assert_eq!(result, "foo.bar.baz");
    }

    #[test]
    fn test_reformat_chain_lines_multiline_chain() {
        let input = "foo.bar\n                  .baz\n                  .qux";
        let result = reformat_chain_lines(input, 0, 2);
        assert_eq!(result, "foo.bar\n  .baz\n  .qux");
    }

    #[test]
    fn test_reformat_chain_lines_safe_navigation() {
        let input = "foo&.bar\n                  &.baz";
        let result = reformat_chain_lines(input, 0, 2);
        assert_eq!(result, "foo&.bar\n  &.baz");
    }

    #[test]
    fn test_reformat_chain_lines_no_chain() {
        let input = "foo(\n  arg1,\n  arg2\n)";
        let result = reformat_chain_lines(input, 0, 2);
        assert_eq!(result, input);
    }

    #[test]
    fn test_reformat_chain_lines_preserves_base_indent() {
        // Simulates a chain inside a 4-space-indented method body:
        // the caller must include base_indent so the printed continuation
        // lines up with `base_indent + indent_width` columns.
        let input = "foo.bar\n                  .baz\n                  .qux";
        let result = reformat_chain_lines(input, 4, 2);
        assert_eq!(result, "foo.bar\n      .baz\n      .qux");
    }

    #[test]
    fn test_line_leading_indent_counts_spaces_and_tabs() {
        let source = "def foo\n    bar\n\tbaz\nqux\n";
        let bar = source.find("bar").unwrap();
        let baz = source.find("baz").unwrap();
        let qux = source.find("qux").unwrap();
        assert_eq!(line_leading_indent(source, bar), 4);
        assert_eq!(line_leading_indent(source, baz), 1);
        assert_eq!(line_leading_indent(source, qux), 0);
        // Out-of-range offset is clamped.
        assert_eq!(line_leading_indent(source, usize::MAX), 0);
    }
}

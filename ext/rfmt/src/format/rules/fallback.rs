//! FallbackRule - Handles nodes without specific rules
//!
//! The fallback rule extracts source text directly, similar to the
//! existing Emitter's emit_generic function. This provides a safety
//! net for node types that haven't been implemented yet.

use crate::ast::Node;
use crate::doc::{concat, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{format_leading_comments, format_trailing_comment, FormatRule};

/// Fallback rule that extracts source text directly.
///
/// This rule is used when no specific rule is registered for a node type.
/// It extracts the original source text and preserves it as-is, handling
/// leading and trailing comments.
pub struct FallbackRule;

impl FormatRule for FallbackRule {
    fn format(
        &self,
        node: &Node,
        ctx: &mut FormatContext,
        _registry: &RuleRegistry,
    ) -> Result<Doc> {
        let mut docs: Vec<Doc> = Vec::with_capacity(3);

        // Add leading comments
        let leading = format_leading_comments(ctx, node.location.start_line);
        if !leading.is_empty() {
            docs.push(leading);
        }

        // Extract source text with chain reformatting
        if let Some(source_text) = ctx.extract_source(node) {
            let reformatted =
                reformat_chain_lines(source_text, ctx.config().formatting.indent_width);
            docs.push(text(reformatted));

            // Mark any comments within this node's range as emitted
            // (they are included in the source extraction)
            mark_internal_comments_emitted(ctx, node);
        }

        // Add trailing comment
        let trailing = format_trailing_comment(ctx, node.location.end_line);
        if !trailing.is_empty() {
            docs.push(trailing);
        }

        Ok(concat(docs))
    }
}

/// Reformat multiline method chain text with indented style.
///
/// Converts aligned method chains to indented style:
/// - First line is kept as-is (trimmed at end)
/// - Subsequent lines starting with `.` or `&.` are re-indented with one level of indentation
fn reformat_chain_lines(source_text: &str, indent_width: usize) -> String {
    let lines: Vec<&str> = source_text.lines().collect();
    if lines.len() <= 1 {
        return source_text.to_string();
    }

    // Check if there are actual chain continuation lines (. or &.)
    let has_chain = lines[1..].iter().any(|l| {
        let t = l.trim_start();
        t.starts_with('.') || t.starts_with("&.")
    });

    if !has_chain {
        return source_text.to_string();
    }

    // Build the indented chain
    let chain_indent = " ".repeat(indent_width);
    let mut result = String::from(lines[0].trim_end());

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

    result
}

/// Marks comments within a node's line range as emitted.
///
/// This is used when source text is extracted directly, as any comments
/// within the extracted range are included in the output.
fn mark_internal_comments_emitted(ctx: &mut FormatContext, node: &Node) {
    // Collect indices first to avoid borrow conflict
    let indices: Vec<usize> = ctx
        .get_comment_indices_in_range(node.location.start_line, node.location.end_line)
        .collect();

    // Mark as emitted in batch
    ctx.mark_comments_emitted(indices);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location, NodeType};
    use crate::config::Config;
    use std::collections::HashMap;

    #[test]
    fn test_fallback_extracts_source() {
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

        let rule = FallbackRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        // The doc should contain the source text
        assert!(matches!(doc, Doc::Text(s) if s == "puts 'hello'"));
    }

    #[test]
    fn test_fallback_handles_empty_source() {
        let config = Config::default();
        let source = "";
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

        let rule = FallbackRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        // Should handle gracefully
        assert!(matches!(doc, Doc::Empty));
    }
}

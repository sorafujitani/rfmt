//! SingletonClassRule - Formats Ruby singleton class definitions
//!
//! Handles singleton class definitions:
//! - `class << self ... end`
//! - `class << object ... end`

use crate::ast::{Node, NodeType};
use crate::doc::{concat, hardline, indent, text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{
    format_child, format_comments_before_end, format_leading_comments, format_statements,
    format_trailing_comment, FormatRule,
};

/// Rule for formatting singleton class definitions.
///
/// Handles `class << self` and `class << object` patterns.
pub struct SingletonClassRule;

impl FormatRule for SingletonClassRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        let mut docs: Vec<Doc> = Vec::with_capacity(8);

        let start_line = node.location.start_line;
        let end_line = node.location.end_line;

        // 1. Leading comments before definition
        let leading = format_leading_comments(ctx, start_line);
        if !leading.is_empty() {
            docs.push(leading);
        }

        // 2. Build header: "class << "
        docs.push(text("class << "));

        // 3. First child is the expression (self or an object)
        if let Some(expression) = node.children.first() {
            if let Some(expr_text) = ctx.extract_source(expression) {
                docs.push(text(expr_text.to_string()));
            }
        }

        // 4. Trailing comment on definition line
        let trailing = format_trailing_comment(ctx, start_line);
        if !trailing.is_empty() {
            docs.push(trailing);
        }

        // 5. Body (children), skipping the first child (expression)
        let mut body_docs: Vec<Doc> = Vec::new();
        let mut has_body_content = false;

        for (i, child) in node.children.iter().enumerate() {
            // Skip the first child (expression: self or object)
            if i == 0 {
                continue;
            }

            if matches!(child.node_type, NodeType::StatementsNode) {
                has_body_content = true;
                body_docs.push(hardline());
                body_docs.push(format_statements(child, ctx, registry)?);
            } else if !is_structural_node_for_singleton(child) {
                has_body_content = true;
                body_docs.push(hardline());
                body_docs.push(format_child(child, ctx, registry)?);
            }
        }

        if has_body_content {
            docs.push(indent(concat(body_docs)));
        }

        // 6. Comments before end
        let comments_before_end = format_comments_before_end(ctx, start_line, end_line);
        if !comments_before_end.is_empty() {
            docs.push(indent(comments_before_end));
        }

        // 7. Add newline before end
        docs.push(hardline());

        // 8. End keyword
        docs.push(text("end"));

        // 9. Trailing comment on end line
        let end_trailing = format_trailing_comment(ctx, end_line);
        if !end_trailing.is_empty() {
            docs.push(end_trailing);
        }

        Ok(concat(docs))
    }
}

/// Check if a node is structural (should be skipped in body).
fn is_structural_node_for_singleton(node: &Node) -> bool {
    matches!(
        node.node_type,
        NodeType::ConstantReadNode
            | NodeType::ConstantWriteNode
            | NodeType::ConstantPathNode
            | NodeType::SelfNode
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location};
    use crate::config::Config;
    use crate::doc::Printer;
    use std::collections::HashMap;

    fn make_singleton_class_node(children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        Node {
            node_type: NodeType::SingletonClassNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_self_node(line: usize, start_offset: usize) -> Node {
        Node {
            node_type: NodeType::SelfNode,
            location: Location::new(line, 9, line, 13, start_offset, start_offset + 4),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_singleton_class() {
        let config = Config::default();
        let source = "class << self\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let self_node = make_self_node(1, 9);
        let node = make_singleton_class_node(vec![self_node], 1, 2);
        ctx.collect_comments(&node);

        let rule = SingletonClassRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "class << self\nend");
    }

    #[test]
    fn test_singleton_class_with_body() {
        let config = Config::default();
        let source = "class << self\n  def foo\n  end\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let self_node = make_self_node(1, 9);
        let method_node = Node {
            node_type: NodeType::DefNode,
            location: Location::new(2, 2, 3, 5, 16, 28),
            children: Vec::new(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), "foo".to_string());
                m
            },
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let statements_node = Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(2, 2, 3, 5, 16, 28),
            children: vec![method_node],
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_singleton_class_node(vec![self_node, statements_node], 1, 4);
        ctx.collect_comments(&node);

        let rule = SingletonClassRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("class << self"));
        assert!(result.contains("def foo"));
        assert!(result.contains("end"));
    }
}

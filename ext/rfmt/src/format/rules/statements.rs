//! StatementsRule - Handles StatementsNode (body of class/module/def)
//!
//! StatementsNode contains a sequence of statements that form the body
//! of a construct. This rule formats each statement with proper spacing.

use crate::ast::Node;
use crate::doc::Doc;
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::{format_statements, FormatRule};

/// Rule for formatting StatementsNode.
///
/// This rule delegates to the shared `format_statements` helper function
/// which properly handles the sequence of child statements.
pub struct StatementsRule;

impl FormatRule for StatementsRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_statements(node, ctx, registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location, NodeType};
    use crate::config::Config;
    use crate::doc::Printer;
    use std::collections::HashMap;

    fn make_statements_node(children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        Node {
            node_type: NodeType::StatementsNode,
            location: Location::new(start_line, 0, end_line, 0, 0, 100),
            children,
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    fn make_call_node(source: &str, line: usize, start_offset: usize) -> Node {
        Node {
            node_type: NodeType::CallNode,
            location: Location::new(line, 0, line, source.len(), start_offset, start_offset + source.len()),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_statements_single_statement() {
        let config = Config::default();
        let source = "puts 'hello'";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let call = make_call_node("puts 'hello'", 1, 0);
        let stmt = make_statements_node(vec![call], 1, 1);

        ctx.collect_comments(&stmt);

        let rule = StatementsRule;
        let doc = rule.format(&stmt, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result, "puts 'hello'\n");
    }

    #[test]
    fn test_statements_multiple_statements() {
        let config = Config::default();
        let source = "a = 1\nb = 2";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let call1 = make_call_node("a = 1", 1, 0);
        let call2 = make_call_node("b = 2", 2, 6);
        let stmt = make_statements_node(vec![call1, call2], 1, 2);

        ctx.collect_comments(&stmt);

        let rule = StatementsRule;
        let doc = rule.format(&stmt, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert!(result.contains("a = 1"));
        assert!(result.contains("b = 2"));
    }

    #[test]
    fn test_statements_empty() {
        let config = Config::default();
        let source = "";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let stmt = make_statements_node(vec![], 1, 1);

        ctx.collect_comments(&stmt);

        let rule = StatementsRule;
        let doc = rule.format(&stmt, &mut ctx, &registry).unwrap();

        assert!(matches!(doc, Doc::Empty));
    }
}

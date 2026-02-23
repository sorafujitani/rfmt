//! DefRule - Formats Ruby method definitions
//!
//! Handles method definitions including:
//! - Instance methods: `def foo`
//! - Class methods: `def self.foo`
//! - Methods with parameters: `def foo(x, y)` or `def foo x, y`
//! - Method bodies
//! - Leading and trailing comments

use crate::ast::Node;
use crate::doc::{text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::FormatRule;

use super::body_end::{format_body_end, BodyEndConfig};

/// Rule for formatting method definitions.
pub struct DefRule;

impl FormatRule for DefRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_body_end(
            ctx,
            registry,
            BodyEndConfig {
                keyword: "def",
                node,
                header_builder: Box::new(build_def_header),
                skip_same_line_children: false,
            },
        )
    }
}

/// Builds the header portion for a method definition.
///
/// Returns: `[receiver.]name[(params)]`
fn build_def_header(node: &Node) -> Vec<Doc> {
    let mut parts: Vec<Doc> = Vec::with_capacity(6);

    // Get receiver if class method (def self.method_name)
    if let Some(receiver) = node.metadata.get("receiver") {
        parts.push(text(receiver));
        parts.push(text("."));
    }

    // Get method name from metadata
    if let Some(name) = node.metadata.get("name") {
        parts.push(text(name));
    }

    // Get parameters from metadata
    if let Some(params_text) = node.metadata.get("parameters_text") {
        let has_parens = node
            .metadata
            .get("has_parens")
            .is_some_and(|v| v == "true");

        if has_parens {
            parts.push(text("("));
            parts.push(text(params_text));
            parts.push(text(")"));
        } else {
            parts.push(text(" "));
            parts.push(text(params_text));
        }
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FormattingInfo, Location, NodeType};
    use crate::config::Config;
    use crate::doc::Printer;
    use std::collections::HashMap;

    fn make_def_node(
        name: &str,
        receiver: Option<&str>,
        params: Option<(&str, bool)>,
        children: Vec<Node>,
        start_line: usize,
        end_line: usize,
    ) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());

        if let Some(r) = receiver {
            metadata.insert("receiver".to_string(), r.to_string());
        }

        if let Some((p, has_parens)) = params {
            metadata.insert("parameters_text".to_string(), p.to_string());
            metadata.insert("has_parens".to_string(), has_parens.to_string());
        }

        Node {
            node_type: NodeType::DefNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_def() {
        let config = Config::default();
        let source = "def foo\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_def_node("foo", None, None, Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = DefRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "def foo\nend");
    }

    #[test]
    fn test_def_with_parens_params() {
        let config = Config::default();
        let source = "def foo(x, y)\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_def_node("foo", None, Some(("x, y", true)), Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = DefRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "def foo(x, y)\nend");
    }

    #[test]
    fn test_def_without_parens_params() {
        let config = Config::default();
        let source = "def foo x, y\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_def_node("foo", None, Some(("x, y", false)), Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = DefRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "def foo x, y\nend");
    }

    #[test]
    fn test_class_method() {
        let config = Config::default();
        let source = "def self.foo\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_def_node("foo", Some("self"), None, Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = DefRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "def self.foo\nend");
    }

    #[test]
    fn test_def_with_body() {
        let config = Config::default();
        let source = "def foo\n  puts 'hello'\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // Create a body node
        let body_node = Node {
            node_type: NodeType::CallNode,
            location: Location::new(2, 2, 2, 15, 10, 23),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_def_node("foo", None, None, vec![body_node], 1, 3);
        ctx.collect_comments(&node);

        let rule = DefRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should have the method, body, and end
        assert!(result.contains("def foo"));
        assert!(result.contains("puts 'hello'"));
        assert!(result.contains("end"));
    }
}

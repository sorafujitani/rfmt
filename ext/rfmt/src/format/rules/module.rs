//! ModuleRule - Formats Ruby module definitions
//!
//! Handles module definitions including:
//! - Simple modules: `module Foo`
//! - Nested modules: `module Foo::Bar`
//! - Module bodies with methods and other declarations
//! - Leading and trailing comments

use crate::ast::Node;
use crate::doc::{text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::FormatRule;

use super::body_end::{format_body_end, BodyEndConfig};

/// Rule for formatting module definitions.
pub struct ModuleRule;

impl FormatRule for ModuleRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_body_end(
            ctx,
            registry,
            BodyEndConfig {
                keyword: "module",
                node,
                header_builder: Box::new(build_module_header),
                skip_same_line_children: false,
            },
        )
    }
}

/// Builds the header portion for a module definition.
///
/// Returns: `ModuleName`
fn build_module_header(node: &Node) -> Vec<Doc> {
    let mut parts: Vec<Doc> = Vec::with_capacity(1);

    // Get module name from metadata
    if let Some(name) = node.metadata.get("name") {
        parts.push(text(name));
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

    fn make_module_node(name: &str, children: Vec<Node>, start_line: usize, end_line: usize) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());

        Node {
            node_type: NodeType::ModuleNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_module() {
        let config = Config::default();
        let source = "module Foo\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_module_node("Foo", Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = ModuleRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "module Foo\nend");
    }

    #[test]
    fn test_nested_module_name() {
        let config = Config::default();
        let source = "module Foo::Bar\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_module_node("Foo::Bar", Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = ModuleRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "module Foo::Bar\nend");
    }

    #[test]
    fn test_module_with_body() {
        let config = Config::default();
        let source = "module Foo\n  def bar\n  end\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // Create a method node as child
        let method_node = Node {
            node_type: NodeType::DefNode,
            location: Location::new(2, 2, 3, 5, 13, 25),
            children: Vec::new(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), "bar".to_string());
                m
            },
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_module_node("Foo", vec![method_node], 1, 4);
        ctx.collect_comments(&node);

        let rule = ModuleRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should have the module, body, and end
        assert!(result.contains("module Foo"));
        assert!(result.contains("def bar"));
        assert!(result.contains("end"));
    }
}

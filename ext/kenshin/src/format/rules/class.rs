//! ClassRule - Formats Ruby class definitions
//!
//! Handles class definitions including:
//! - Simple classes: `class Foo`
//! - Classes with inheritance: `class Foo < Bar`
//! - Class bodies with methods and other declarations
//! - Leading and trailing comments

use crate::ast::Node;
use crate::doc::{text, Doc};
use crate::error::Result;
use crate::format::context::FormatContext;
use crate::format::registry::RuleRegistry;
use crate::format::rule::FormatRule;

use super::body_end::{format_body_end, BodyEndConfig};

/// Rule for formatting class definitions.
pub struct ClassRule;

impl FormatRule for ClassRule {
    fn format(&self, node: &Node, ctx: &mut FormatContext, registry: &RuleRegistry) -> Result<Doc> {
        format_body_end(
            ctx,
            registry,
            BodyEndConfig {
                keyword: "class",
                node,
                header_builder: Box::new(build_class_header),
                skip_same_line_children: true,
            },
        )
    }
}

/// Builds the header portion for a class definition.
///
/// Returns: `ClassName` or `ClassName < Superclass`
fn build_class_header(node: &Node) -> Vec<Doc> {
    let mut parts: Vec<Doc> = Vec::with_capacity(4);

    // Get class name from metadata
    if let Some(name) = node.metadata.get("name") {
        parts.push(text(name));
    }

    // Get superclass from metadata if present
    if let Some(superclass) = node.metadata.get("superclass") {
        parts.push(text(" < "));
        parts.push(text(superclass));
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

    fn make_class_node(
        name: &str,
        superclass: Option<&str>,
        children: Vec<Node>,
        start_line: usize,
        end_line: usize,
    ) -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());
        if let Some(sc) = superclass {
            metadata.insert("superclass".to_string(), sc.to_string());
        }

        Node {
            node_type: NodeType::ClassNode,
            location: Location::new(start_line, 0, end_line, 3, 0, 50),
            children,
            metadata,
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_simple_class() {
        let config = Config::default();
        let source = "class Foo\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_class_node("Foo", None, Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = ClassRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "class Foo\nend");
    }

    #[test]
    fn test_class_with_inheritance() {
        let config = Config::default();
        let source = "class Foo < Bar\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        let node = make_class_node("Foo", Some("Bar"), Vec::new(), 1, 2);
        ctx.collect_comments(&node);

        let rule = ClassRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.trim(), "class Foo < Bar\nend");
    }

    #[test]
    fn test_class_with_body() {
        let config = Config::default();
        let source = "class Foo\n  def bar\n  end\nend";
        let mut ctx = FormatContext::new(&config, source);
        let registry = RuleRegistry::default_registry();

        // Create a method node as child
        let method_node = Node {
            node_type: NodeType::DefNode,
            location: Location::new(2, 2, 3, 5, 12, 24),
            children: Vec::new(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), "bar".to_string());
                m
            },
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let node = make_class_node("Foo", None, vec![method_node], 1, 4);
        ctx.collect_comments(&node);

        let rule = ClassRule;
        let doc = rule.format(&node, &mut ctx, &registry).unwrap();

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should have the class, body, and end
        assert!(result.contains("class Foo"));
        assert!(result.contains("def bar"));
        assert!(result.contains("end"));
    }
}

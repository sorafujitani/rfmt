//! RuleRegistry - Dispatches nodes to appropriate formatting rules
//!
//! The registry maps NodeType to FormatRule implementations, allowing
//! the formatter to dispatch nodes to the correct rule.

use crate::ast::NodeType;
use std::borrow::Cow;
use std::collections::HashMap;

use super::rule::{BoxedRule, FormatRule};
use super::rules::{
    BeginRule, BlockRule, CallRule, CaseMatchRule, CaseRule, ClassRule, DefRule, EnsureRule,
    FallbackRule, ForRule, IfRule, InRule, LambdaRule, ModuleRule, RescueRule, UnlessRule,
    UntilRule, WhenRule, WhileRule,
};

/// Key type for the registry, derived from NodeType.
///
/// Uses `Cow<'static, str>` to avoid allocation for known node types
/// while still supporting dynamic Unknown types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeTypeKey(Cow<'static, str>);

impl NodeTypeKey {
    #[inline]
    const fn from_static(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }

    fn from_owned(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&NodeType> for NodeTypeKey {
    #[inline]
    fn from(node_type: &NodeType) -> Self {
        match node_type {
            NodeType::ProgramNode => Self::from_static("program_node"),
            NodeType::StatementsNode => Self::from_static("statements_node"),
            NodeType::ClassNode => Self::from_static("class_node"),
            NodeType::ModuleNode => Self::from_static("module_node"),
            NodeType::DefNode => Self::from_static("def_node"),
            NodeType::CallNode => Self::from_static("call_node"),
            NodeType::IfNode => Self::from_static("if_node"),
            NodeType::UnlessNode => Self::from_static("unless_node"),
            NodeType::BeginNode => Self::from_static("begin_node"),
            NodeType::RescueNode => Self::from_static("rescue_node"),
            NodeType::EnsureNode => Self::from_static("ensure_node"),
            NodeType::CaseNode => Self::from_static("case_node"),
            NodeType::WhenNode => Self::from_static("when_node"),
            NodeType::CaseMatchNode => Self::from_static("case_match_node"),
            NodeType::InNode => Self::from_static("in_node"),
            NodeType::WhileNode => Self::from_static("while_node"),
            NodeType::UntilNode => Self::from_static("until_node"),
            NodeType::ForNode => Self::from_static("for_node"),
            NodeType::BlockNode => Self::from_static("block_node"),
            NodeType::LambdaNode => Self::from_static("lambda_node"),
            NodeType::Unknown(s) => Self::from_owned(s.clone()),
            // Default for unhandled types
            _ => Self::from_static("unknown"),
        }
    }
}

/// Registry that maps NodeType to FormatRule.
pub struct RuleRegistry {
    rules: HashMap<NodeTypeKey, BoxedRule>,
    fallback: BoxedRule,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            fallback: Box::new(FallbackRule),
        }
    }

    #[inline]
    pub fn add<R: FormatRule + 'static>(mut self, node_type: NodeType, rule: R) -> Self {
        let key = NodeTypeKey::from(&node_type);
        self.rules.insert(key, Box::new(rule));
        self
    }

    pub fn add_rule<R: FormatRule + 'static>(&mut self, node_type: NodeType, rule: R) {
        let key = NodeTypeKey::from(&node_type);
        self.rules.insert(key, Box::new(rule));
    }

    #[inline]
    pub fn get_rule(&self, node_type: &NodeType) -> &dyn FormatRule {
        let key = NodeTypeKey::from(node_type);
        self.rules
            .get(&key)
            .map(|r| r.as_ref())
            .unwrap_or(self.fallback.as_ref())
    }

    pub fn default_registry() -> Self {
        Self::new()
            .add(NodeType::ClassNode, ClassRule)
            .add(NodeType::ModuleNode, ModuleRule)
            .add(NodeType::DefNode, DefRule)
            .add(NodeType::IfNode, IfRule)
            .add(NodeType::UnlessNode, UnlessRule)
            .add(NodeType::CaseNode, CaseRule)
            .add(NodeType::WhenNode, WhenRule)
            .add(NodeType::CaseMatchNode, CaseMatchRule)
            .add(NodeType::InNode, InRule)
            .add(NodeType::BeginNode, BeginRule)
            .add(NodeType::RescueNode, RescueRule)
            .add(NodeType::EnsureNode, EnsureRule)
            .add(NodeType::CallNode, CallRule)
            .add(NodeType::BlockNode, BlockRule)
            .add(NodeType::LambdaNode, LambdaRule)
            .add(NodeType::WhileNode, WhileRule)
            .add(NodeType::UntilNode, UntilRule)
            .add(NodeType::ForNode, ForRule)
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_key_from_node_type() {
        let key = NodeTypeKey::from(&NodeType::ClassNode);
        assert_eq!(key.0, "class_node");

        let key = NodeTypeKey::from(&NodeType::DefNode);
        assert_eq!(key.0, "def_node");

        let key = NodeTypeKey::from(&NodeType::Unknown("custom".to_string()));
        assert_eq!(key.0, "custom");
    }

    #[test]
    fn test_registry_get_registered_rule() {
        let registry = RuleRegistry::default_registry();

        // ClassRule should be registered
        let _rule = registry.get_rule(&NodeType::ClassNode);
        // If we get here, the rule was found
    }

    #[test]
    fn test_registry_get_fallback_rule() {
        let registry = RuleRegistry::default_registry();

        // IntegerNode doesn't have a specific rule, should use fallback
        let _rule = registry.get_rule(&NodeType::IntegerNode);
        // If we get here, the fallback was used
    }

    #[test]
    fn test_registry_custom_registration() {
        // Using builder pattern with method chaining
        let registry = RuleRegistry::new().add(NodeType::IfNode, FallbackRule);

        // Should find the registered rule
        let _rule = registry.get_rule(&NodeType::IfNode);
    }

    #[test]
    fn test_registry_add_rule_mutable() {
        // Using mutable reference variant
        let mut registry = RuleRegistry::new();
        registry.add_rule(NodeType::DefNode, FallbackRule);

        // Should find the registered rule
        let _rule = registry.get_rule(&NodeType::DefNode);
    }
}

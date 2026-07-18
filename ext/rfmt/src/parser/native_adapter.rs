//! Native ruby-prism -> internal AST converter (prism migration phases 3-4).
//!
//! Ports the conversion semantics of `lib/rfmt/prism_bridge.rb` exactly so the
//! two paths can be diffed node-by-node (`tests/native_parity.rs`), including
//! the live per-type metadata keys and the flat root comment list.

use crate::ast::{
    Comment, CommentPosition, CommentType, FormattingInfo, Location, Node as AstNode, NodeType,
};
use crate::error::{Result, RfmtError};
use crate::parser::RubyParser;
use ruby_prism::{
    ArgumentsNode, ConstantId, Location as PrismLocation, Node as PrismNode, ParseResult, Visit,
};
use std::collections::HashMap;

/// Parses Ruby source with the ruby-prism crate and converts it to the
/// internal tree, without any round-trip through Ruby/JSON.
pub struct NativeAdapter;

impl NativeAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyParser for NativeAdapter {
    fn parse(&self, source: &str) -> Result<AstNode> {
        let parse_result = ruby_prism::parse(source.as_bytes());
        let index = LineIndex::new(source.as_bytes());

        let errors: Vec<String> = parse_result
            .errors()
            .map(|error| {
                let (line, column) = index.line_column(error.location().start_offset());
                format!("{}:{}: {}", line, column, error.message())
            })
            .collect();
        if !errors.is_empty() {
            // ParseError so lib/rfmt.rb surfaces it as Rfmt::Error
            // "Failed to parse Ruby code: ..." (PrismBridge parity).
            return Err(RfmtError::ParseError(format!(
                "Parse errors:\n{}",
                errors.join("\n")
            )));
        }

        let converter = Converter { index: &index };
        let mut root = converter.convert(&parse_result.node()).node;
        // As in the bridge/PrismAdapter pipeline, all comments live in a flat
        // list on the root node; per-node comments stay empty.
        root.comments = root_comments(&parse_result, &index);
        Ok(root)
    }
}

/// The bridge's serialize_ast_with_comments: every comment with its source
/// slice as text, always Leading (refined later by the formatter).
fn root_comments(result: &ParseResult<'_>, index: &LineIndex) -> Vec<Comment> {
    result
        .comments()
        .map(|comment| {
            let loc = comment.location();
            let (start_line, start_column) = index.line_column(loc.start_offset());
            let (mut end_line, end_column) = index.line_column(loc.end_offset());
            // Prism ends `=begin ... =end` at column 0 of the line AFTER the
            // terminator; snap back so the comment doesn't appear to overlap
            // the next statement. Only the location is snapped: text and
            // offsets keep the raw span (bridge parity).
            if end_column == 0 && end_line > start_line {
                end_line -= 1;
            }
            Comment {
                text: String::from_utf8_lossy(loc.as_slice()).into_owned(),
                location: Location::new(
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                    loc.start_offset(),
                    loc.end_offset(),
                ),
                comment_type: match comment.type_() {
                    ruby_prism::CommentType::InlineComment => CommentType::Line,
                    ruby_prism::CommentType::EmbDocComment => CommentType::Block,
                },
                position: CommentPosition::Leading,
            }
        })
        .collect()
}

/// Line-start byte offsets over the source, for deriving 1-based lines and
/// 0-based byte columns from prism's byte offsets.
struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &[u8]) -> Self {
        let mut line_starts = vec![0];
        for (i, byte) in source.iter().enumerate() {
            if *byte == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    fn line_column(&self, offset: usize) -> (usize, usize) {
        let line = self.line_starts.partition_point(|&start| start <= offset);
        (line, offset - self.line_starts[line - 1])
    }
}

/// A widening candidate derived from some node's `closing_loc`, with the
/// heredoc terminator-line snap already applied to `end_line`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClosingCandidate {
    end_offset: usize,
    end_line: usize,
    end_column: usize,
}

/// Merge a candidate, keeping the earliest-seen candidate on end_offset ties
/// (the bridge only replaces on strictly greater end_offset).
fn fold_candidate(best: &mut Option<ClosingCandidate>, candidate: Option<ClosingCandidate>) {
    if let Some(c) = candidate {
        match best {
            Some(b) if c.end_offset <= b.end_offset => {}
            _ => *best = Some(c),
        }
    }
}

/// How a prism child subtree participates in the conversion of its parent,
/// mirroring the per-type extraction in prism_bridge.rb.
enum Part<'pr> {
    /// Converted into a child of the internal node.
    Convert(PrismNode<'pr>),
    /// Dropped from the children but still scanned for heredoc widening (the
    /// bridge widens over prism's generic child_nodes, not the extracted
    /// children).
    WidenOnly(PrismNode<'pr>),
    /// A flattened intermediate (e.g. BlockParametersNode): its children are
    /// converted separately, but its own closing_loc still counts.
    ClosingOf(PrismNode<'pr>),
    /// A child with no prism counterpart in the Rust crate (RationalNode's
    /// deprecated `numeric`), rebuilt to match the bridge output.
    Synthesized(AstNode),
}

struct Converted {
    node: AstNode,
    /// Max closing candidate of this node and its whole prism subtree, so the
    /// parent widens in O(1) instead of re-walking the subtree per ancestor.
    closing: Option<ClosingCandidate>,
}

struct Converter<'a> {
    index: &'a LineIndex,
}

impl Converter<'_> {
    fn convert(&self, node: &PrismNode<'_>) -> Converted {
        let loc = node.location();
        let (start_line, start_column) = self.index.line_column(loc.start_offset());
        let (mut end_line, mut end_column) = self.index.line_column(loc.end_offset());
        let mut end_offset = loc.end_offset();
        // The bridge computes `multiline` from the original prism location,
        // before heredoc widening.
        let multiline = start_line != end_line;

        let mut best = self.own_closing_candidate(node);
        let mut children = Vec::new();

        match self.parts(node) {
            Some(parts) => {
                for part in parts {
                    match part {
                        Part::Convert(child) => {
                            let converted = self.convert(&child);
                            fold_candidate(&mut best, converted.closing);
                            children.push(converted.node);
                        }
                        Part::WidenOnly(child) => {
                            fold_candidate(&mut best, self.subtree_closing(&child));
                        }
                        Part::ClosingOf(child) => {
                            fold_candidate(&mut best, self.own_closing_candidate(&child));
                        }
                        Part::Synthesized(child) => children.push(child),
                    }
                }
            }
            // Node types the bridge extracts no children from: the converted
            // node is a leaf, but widening still scans the real prism subtree.
            None => best = self.subtree_closing(node),
        }

        // Oracle check: the bottom-up fold must equal a full generic-subtree
        // scan (catches any per-type arm that drops a closing-bearing child).
        debug_assert_eq!(
            best,
            self.subtree_closing(node),
            "widening fold diverged from generic scan for {}",
            node_kind_name(node)
        );

        if let Some(candidate) = best {
            if candidate.end_offset > end_offset {
                end_offset = candidate.end_offset;
                end_line = candidate.end_line;
                end_column = candidate.end_column;
            }
        }

        let node = AstNode {
            node_type: NodeType::from_str(node_kind_name(node)),
            location: Location::new(
                start_line,
                start_column,
                end_line,
                end_column,
                loc.start_offset(),
                end_offset,
            ),
            children,
            metadata: extract_metadata(node),
            comments: Vec::new(),
            formatting: FormattingInfo {
                multiline,
                ..FormattingInfo::default()
            },
        };

        Converted {
            node,
            closing: best,
        }
    }

    fn own_closing_candidate(&self, node: &PrismNode<'_>) -> Option<ClosingCandidate> {
        own_closing_loc(node).map(|loc| self.closing_candidate(&loc))
    }

    fn closing_candidate(&self, loc: &PrismLocation<'_>) -> ClosingCandidate {
        let (start_line, _) = self.index.line_column(loc.start_offset());
        let (end_line, end_column) = self.index.line_column(loc.end_offset());
        // Prism reports a heredoc terminator's end as column 0 of the line
        // AFTER the terminator; snap back so blank-line preservation works
        // (see prism_bridge.rb heredoc_terminator_line).
        let end_line = if end_column == 0 && end_line > start_line {
            start_line
        } else {
            end_line
        };
        ClosingCandidate {
            end_offset: loc.end_offset(),
            end_line,
            end_column,
        }
    }

    /// Max closing candidate over a node and all of its descendants, in the
    /// same pre-order the bridge walks. Intentional divergence: the bridge
    /// stops recursing at depth 10, so for a heredoc buried deeper than that
    /// it fails to widen the far ancestors; the native path widens them too.
    /// Parity fixtures therefore stay shallower than the cap.
    fn subtree_closing(&self, node: &PrismNode<'_>) -> Option<ClosingCandidate> {
        let mut best = self.own_closing_candidate(node);
        for child in direct_children(node) {
            fold_candidate(&mut best, self.subtree_closing(&child));
        }
        best
    }

    /// The bridge's per-type children extraction (prism_bridge.rb 199-432).
    /// Returns None for node types the bridge gives no children (its `else`
    /// branch and explicit empty arms), which all behave identically.
    #[allow(clippy::too_many_lines)]
    fn parts<'pr>(&self, node: &PrismNode<'pr>) -> Option<Vec<Part<'pr>>> {
        let mut parts: Vec<Part<'pr>> = Vec::new();

        macro_rules! convert {
            ($node:expr) => {
                parts.push(Part::Convert($node))
            };
        }
        macro_rules! convert_opt {
            ($node:expr) => {
                if let Some(child) = $node {
                    parts.push(Part::Convert(child));
                }
            };
        }
        macro_rules! convert_list {
            ($list:expr) => {
                for child in $list.iter() {
                    parts.push(Part::Convert(child));
                }
            };
        }
        // `x.child_nodes.compact` on an ArgumentsNode in the bridge.
        macro_rules! convert_args {
            ($args:expr) => {
                if let Some(arguments) = $args {
                    convert_list!(arguments.arguments());
                }
            };
        }

        match node {
            PrismNode::ProgramNode { .. } => {
                let n = node.as_program_node().unwrap();
                convert_list!(n.statements().body());
            }
            PrismNode::StatementsNode { .. } => {
                let n = node.as_statements_node().unwrap();
                convert_list!(n.body());
            }
            PrismNode::ClassNode { .. } => {
                let n = node.as_class_node().unwrap();
                convert!(n.constant_path());
                convert_opt!(n.superclass());
                convert_opt!(n.body());
            }
            PrismNode::ModuleNode { .. } => {
                let n = node.as_module_node().unwrap();
                convert!(n.constant_path());
                convert_opt!(n.body());
            }
            PrismNode::DefNode { .. } => {
                let n = node.as_def_node().unwrap();
                // The bridge drops the receiver from the children, but its
                // widening walk still sees it.
                if let Some(receiver) = n.receiver() {
                    parts.push(Part::WidenOnly(receiver));
                }
                if let Some(parameters) = n.parameters() {
                    self.flatten(&mut parts, parameters.as_node());
                }
                convert_opt!(n.body());
            }
            PrismNode::CallNode { .. } => {
                let n = node.as_call_node().unwrap();
                convert_opt!(n.receiver());
                convert_args!(n.arguments());
                convert_opt!(n.block());
            }
            PrismNode::IfNode { .. } => {
                let n = node.as_if_node().unwrap();
                convert!(n.predicate());
                convert_opt!(n.statements().map(|s| s.as_node()));
                convert_opt!(n.subsequent());
            }
            PrismNode::UnlessNode { .. } => {
                let n = node.as_unless_node().unwrap();
                convert!(n.predicate());
                convert_opt!(n.statements().map(|s| s.as_node()));
                convert_opt!(n.else_clause().map(|e| e.as_node()));
            }
            PrismNode::ElseNode { .. } => {
                let n = node.as_else_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::ArrayNode { .. } => {
                let n = node.as_array_node().unwrap();
                convert_list!(n.elements());
            }
            PrismNode::HashNode { .. } => {
                let n = node.as_hash_node().unwrap();
                convert_list!(n.elements());
            }
            PrismNode::BlockNode { .. } => {
                let n = node.as_block_node().unwrap();
                if let Some(parameters) = n.parameters() {
                    self.flatten(&mut parts, parameters);
                }
                convert_opt!(n.body());
            }
            PrismNode::BeginNode { .. } => {
                let n = node.as_begin_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
                convert_opt!(n.rescue_clause().map(|r| r.as_node()));
                convert_opt!(n.else_clause().map(|e| e.as_node()));
                convert_opt!(n.ensure_clause().map(|e| e.as_node()));
            }
            PrismNode::EnsureNode { .. } => {
                let n = node.as_ensure_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::LambdaNode { .. } => {
                let n = node.as_lambda_node().unwrap();
                if let Some(parameters) = n.parameters() {
                    self.flatten(&mut parts, parameters);
                }
                convert_opt!(n.body());
            }
            PrismNode::RescueNode { .. } => {
                let n = node.as_rescue_node().unwrap();
                convert_list!(n.exceptions());
                convert_opt!(n.reference());
                convert_opt!(n.statements().map(|s| s.as_node()));
                convert_opt!(n.subsequent().map(|s| s.as_node()));
            }
            PrismNode::LocalVariableWriteNode { .. } => {
                convert!(node.as_local_variable_write_node().unwrap().value());
            }
            PrismNode::InstanceVariableWriteNode { .. } => {
                convert!(node.as_instance_variable_write_node().unwrap().value());
            }
            PrismNode::ReturnNode { .. } => {
                convert_args!(node.as_return_node().unwrap().arguments());
            }
            PrismNode::OrNode { .. } => {
                let n = node.as_or_node().unwrap();
                convert!(n.left());
                convert!(n.right());
            }
            PrismNode::AndNode { .. } => {
                let n = node.as_and_node().unwrap();
                convert!(n.left());
                convert!(n.right());
            }
            PrismNode::AssocNode { .. } => {
                let n = node.as_assoc_node().unwrap();
                convert!(n.key());
                convert!(n.value());
            }
            PrismNode::KeywordHashNode { .. } => {
                convert_list!(node.as_keyword_hash_node().unwrap().elements());
            }
            PrismNode::InterpolatedStringNode { .. } => {
                convert_list!(node.as_interpolated_string_node().unwrap().parts());
            }
            PrismNode::EmbeddedStatementsNode { .. } => {
                let n = node.as_embedded_statements_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::CaseNode { .. } => {
                let n = node.as_case_node().unwrap();
                convert_opt!(n.predicate());
                convert_list!(n.conditions());
                convert_opt!(n.else_clause().map(|e| e.as_node()));
            }
            PrismNode::WhenNode { .. } => {
                let n = node.as_when_node().unwrap();
                convert_list!(n.conditions());
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::WhileNode { .. } => {
                let n = node.as_while_node().unwrap();
                convert!(n.predicate());
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::UntilNode { .. } => {
                let n = node.as_until_node().unwrap();
                convert!(n.predicate());
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::ForNode { .. } => {
                let n = node.as_for_node().unwrap();
                convert!(n.index());
                convert!(n.collection());
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::BreakNode { .. } => {
                convert_args!(node.as_break_node().unwrap().arguments());
            }
            PrismNode::NextNode { .. } => {
                convert_args!(node.as_next_node().unwrap().arguments());
            }
            PrismNode::YieldNode { .. } => {
                convert_args!(node.as_yield_node().unwrap().arguments());
            }
            PrismNode::SuperNode { .. } => {
                let n = node.as_super_node().unwrap();
                convert_args!(n.arguments());
                convert_opt!(n.block());
            }
            PrismNode::ForwardingSuperNode { .. } => {
                let n = node.as_forwarding_super_node().unwrap();
                convert_opt!(n.block().map(|b| b.as_node()));
            }
            PrismNode::RescueModifierNode { .. } => {
                let n = node.as_rescue_modifier_node().unwrap();
                convert!(n.expression());
                convert!(n.rescue_expression());
            }
            PrismNode::RangeNode { .. } => {
                let n = node.as_range_node().unwrap();
                convert_opt!(n.left());
                convert_opt!(n.right());
            }
            PrismNode::SplatNode { .. } => {
                convert_opt!(node.as_splat_node().unwrap().expression());
            }
            PrismNode::InterpolatedRegularExpressionNode { .. } => {
                convert_list!(node
                    .as_interpolated_regular_expression_node()
                    .unwrap()
                    .parts());
            }
            PrismNode::InterpolatedSymbolNode { .. } => {
                convert_list!(node.as_interpolated_symbol_node().unwrap().parts());
            }
            PrismNode::InterpolatedXStringNode { .. } => {
                convert_list!(node.as_interpolated_x_string_node().unwrap().parts());
            }
            PrismNode::ClassVariableWriteNode { .. } => {
                convert!(node.as_class_variable_write_node().unwrap().value());
            }
            PrismNode::GlobalVariableWriteNode { .. } => {
                convert!(node.as_global_variable_write_node().unwrap().value());
            }
            PrismNode::ClassVariableOrWriteNode { .. } => {
                convert!(node.as_class_variable_or_write_node().unwrap().value());
            }
            PrismNode::ClassVariableAndWriteNode { .. } => {
                convert!(node.as_class_variable_and_write_node().unwrap().value());
            }
            PrismNode::GlobalVariableOrWriteNode { .. } => {
                convert!(node.as_global_variable_or_write_node().unwrap().value());
            }
            PrismNode::GlobalVariableAndWriteNode { .. } => {
                convert!(node.as_global_variable_and_write_node().unwrap().value());
            }
            PrismNode::LocalVariableOrWriteNode { .. } => {
                convert!(node.as_local_variable_or_write_node().unwrap().value());
            }
            PrismNode::LocalVariableAndWriteNode { .. } => {
                convert!(node.as_local_variable_and_write_node().unwrap().value());
            }
            PrismNode::InstanceVariableOrWriteNode { .. } => {
                convert!(node.as_instance_variable_or_write_node().unwrap().value());
            }
            PrismNode::InstanceVariableAndWriteNode { .. } => {
                convert!(node.as_instance_variable_and_write_node().unwrap().value());
            }
            PrismNode::ConstantOrWriteNode { .. } => {
                convert!(node.as_constant_or_write_node().unwrap().value());
            }
            PrismNode::ConstantAndWriteNode { .. } => {
                convert!(node.as_constant_and_write_node().unwrap().value());
            }
            PrismNode::ClassVariableOperatorWriteNode { .. } => {
                convert!(node
                    .as_class_variable_operator_write_node()
                    .unwrap()
                    .value());
            }
            PrismNode::GlobalVariableOperatorWriteNode { .. } => {
                convert!(node
                    .as_global_variable_operator_write_node()
                    .unwrap()
                    .value());
            }
            PrismNode::LocalVariableOperatorWriteNode { .. } => {
                convert!(node
                    .as_local_variable_operator_write_node()
                    .unwrap()
                    .value());
            }
            PrismNode::InstanceVariableOperatorWriteNode { .. } => {
                convert!(node
                    .as_instance_variable_operator_write_node()
                    .unwrap()
                    .value());
            }
            PrismNode::ConstantOperatorWriteNode { .. } => {
                convert!(node.as_constant_operator_write_node().unwrap().value());
            }
            PrismNode::ConstantPathOrWriteNode { .. } => {
                let n = node.as_constant_path_or_write_node().unwrap();
                convert!(n.target().as_node());
                convert!(n.value());
            }
            PrismNode::ConstantPathAndWriteNode { .. } => {
                let n = node.as_constant_path_and_write_node().unwrap();
                convert!(n.target().as_node());
                convert!(n.value());
            }
            PrismNode::ConstantPathOperatorWriteNode { .. } => {
                let n = node.as_constant_path_operator_write_node().unwrap();
                convert!(n.target().as_node());
                convert!(n.value());
            }
            PrismNode::ConstantPathWriteNode { .. } => {
                let n = node.as_constant_path_write_node().unwrap();
                convert!(n.target().as_node());
                convert!(n.value());
            }
            PrismNode::CaseMatchNode { .. } => {
                let n = node.as_case_match_node().unwrap();
                convert_opt!(n.predicate());
                convert_list!(n.conditions());
                convert_opt!(n.else_clause().map(|e| e.as_node()));
            }
            PrismNode::InNode { .. } => {
                let n = node.as_in_node().unwrap();
                convert!(n.pattern());
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::MatchPredicateNode { .. } => {
                let n = node.as_match_predicate_node().unwrap();
                convert!(n.value());
                convert!(n.pattern());
            }
            PrismNode::MatchRequiredNode { .. } => {
                let n = node.as_match_required_node().unwrap();
                convert!(n.value());
                convert!(n.pattern());
            }
            PrismNode::ParenthesesNode { .. } => {
                convert_opt!(node.as_parentheses_node().unwrap().body());
            }
            PrismNode::DefinedNode { .. } => {
                convert!(node.as_defined_node().unwrap().value());
            }
            PrismNode::SingletonClassNode { .. } => {
                let n = node.as_singleton_class_node().unwrap();
                convert!(n.expression());
                convert_opt!(n.body());
            }
            PrismNode::AliasMethodNode { .. } => {
                let n = node.as_alias_method_node().unwrap();
                convert!(n.new_name());
                convert!(n.old_name());
            }
            PrismNode::AliasGlobalVariableNode { .. } => {
                let n = node.as_alias_global_variable_node().unwrap();
                convert!(n.new_name());
                convert!(n.old_name());
            }
            PrismNode::UndefNode { .. } => {
                convert_list!(node.as_undef_node().unwrap().names());
            }
            PrismNode::AssocSplatNode { .. } => {
                convert_opt!(node.as_assoc_splat_node().unwrap().value());
            }
            PrismNode::BlockArgumentNode { .. } => {
                convert_opt!(node.as_block_argument_node().unwrap().expression());
            }
            PrismNode::MultiWriteNode { .. } => {
                let n = node.as_multi_write_node().unwrap();
                convert_list!(n.lefts());
                convert_opt!(n.rest());
                convert_list!(n.rights());
                convert!(n.value());
            }
            PrismNode::MultiTargetNode { .. } => {
                let n = node.as_multi_target_node().unwrap();
                convert_list!(n.lefts());
                convert_opt!(n.rest());
                convert_list!(n.rights());
            }
            PrismNode::PreExecutionNode { .. } => {
                let n = node.as_pre_execution_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            PrismNode::PostExecutionNode { .. } => {
                let n = node.as_post_execution_node().unwrap();
                convert_opt!(n.statements().map(|s| s.as_node()));
            }
            // The bridge's arm calls the deprecated RationalNode#numeric,
            // which the Rust crate does not expose; rebuild the same child.
            PrismNode::RationalNode { .. } => {
                parts.push(Part::Synthesized(self.rational_numeric(node)));
            }
            PrismNode::ImaginaryNode { .. } => {
                convert!(node.as_imaginary_node().unwrap().numeric());
            }
            PrismNode::EmbeddedVariableNode { .. } => {
                convert!(node.as_embedded_variable_node().unwrap().variable());
            }
            PrismNode::ArrayPatternNode { .. } => {
                let n = node.as_array_pattern_node().unwrap();
                if let Some(constant) = n.constant() {
                    parts.push(Part::WidenOnly(constant));
                }
                convert_list!(n.requireds());
                convert_opt!(n.rest());
                convert_list!(n.posts());
            }
            PrismNode::HashPatternNode { .. } => {
                let n = node.as_hash_pattern_node().unwrap();
                if let Some(constant) = n.constant() {
                    parts.push(Part::WidenOnly(constant));
                }
                convert_list!(n.elements());
                convert_opt!(n.rest());
            }
            PrismNode::FindPatternNode { .. } => {
                let n = node.as_find_pattern_node().unwrap();
                if let Some(constant) = n.constant() {
                    parts.push(Part::WidenOnly(constant));
                }
                convert!(n.left().as_node());
                convert_list!(n.requireds());
                convert!(n.right());
            }
            PrismNode::CapturePatternNode { .. } => {
                let n = node.as_capture_pattern_node().unwrap();
                convert!(n.value());
                convert!(n.target().as_node());
            }
            PrismNode::AlternationPatternNode { .. } => {
                let n = node.as_alternation_pattern_node().unwrap();
                convert!(n.left());
                convert!(n.right());
            }
            PrismNode::PinnedExpressionNode { .. } => {
                convert!(node.as_pinned_expression_node().unwrap().expression());
            }
            PrismNode::PinnedVariableNode { .. } => {
                convert!(node.as_pinned_variable_node().unwrap().variable());
            }
            PrismNode::CallAndWriteNode { .. } => {
                let n = node.as_call_and_write_node().unwrap();
                convert_opt!(n.receiver());
                convert!(n.value());
            }
            PrismNode::CallOrWriteNode { .. } => {
                let n = node.as_call_or_write_node().unwrap();
                convert_opt!(n.receiver());
                convert!(n.value());
            }
            PrismNode::CallOperatorWriteNode { .. } => {
                let n = node.as_call_operator_write_node().unwrap();
                convert_opt!(n.receiver());
                convert!(n.value());
            }
            PrismNode::IndexAndWriteNode { .. } => {
                let n = node.as_index_and_write_node().unwrap();
                convert_opt!(n.receiver());
                convert_opt!(n.arguments().map(|a: ArgumentsNode<'pr>| a.as_node()));
                if let Some(block) = n.block() {
                    parts.push(Part::WidenOnly(block.as_node()));
                }
                convert!(n.value());
            }
            PrismNode::IndexOrWriteNode { .. } => {
                let n = node.as_index_or_write_node().unwrap();
                convert_opt!(n.receiver());
                convert_opt!(n.arguments().map(|a: ArgumentsNode<'pr>| a.as_node()));
                if let Some(block) = n.block() {
                    parts.push(Part::WidenOnly(block.as_node()));
                }
                convert!(n.value());
            }
            PrismNode::IndexOperatorWriteNode { .. } => {
                let n = node.as_index_operator_write_node().unwrap();
                convert_opt!(n.receiver());
                convert_opt!(n.arguments().map(|a: ArgumentsNode<'pr>| a.as_node()));
                if let Some(block) = n.block() {
                    parts.push(Part::WidenOnly(block.as_node()));
                }
                convert!(n.value());
            }
            PrismNode::MatchWriteNode { .. } => {
                let n = node.as_match_write_node().unwrap();
                convert!(n.call().as_node());
                convert_list!(n.targets());
            }
            PrismNode::FlipFlopNode { .. } => {
                let n = node.as_flip_flop_node().unwrap();
                convert_opt!(n.left());
                convert_opt!(n.right());
            }
            PrismNode::ImplicitNode { .. } => {
                convert!(node.as_implicit_node().unwrap().value());
            }
            _ => return None,
        }

        Some(parts)
    }

    /// RationalNode#numeric in the Ruby gem: an IntegerNode or FloatNode
    /// spanning the literal minus the trailing `r`.
    fn rational_numeric(&self, node: &PrismNode<'_>) -> AstNode {
        let loc = node.location();
        let end_offset = loc.end_offset() - 1;
        let (start_line, start_column) = self.index.line_column(loc.start_offset());
        let (end_line, end_column) = self.index.line_column(end_offset);
        // Rational literals allow no exponent, so a dot means a float base.
        let kind = if loc.as_slice().contains(&b'.') {
            "float_node"
        } else {
            "integer_node"
        };
        AstNode {
            node_type: NodeType::from_str(kind),
            location: Location::new(
                start_line,
                start_column,
                end_line,
                end_column,
                loc.start_offset(),
                end_offset,
            ),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo {
                multiline: start_line != end_line,
                ..FormattingInfo::default()
            },
        }
    }

    /// The bridge's `x.child_nodes.compact` flattening of a parameters node.
    /// The flattened node's own closing_loc (BlockParametersNode's `|`) still
    /// participates in widening.
    fn flatten<'pr>(&self, parts: &mut Vec<Part<'pr>>, intermediate: PrismNode<'pr>) {
        for child in direct_children(&intermediate) {
            parts.push(Part::Convert(child));
        }
        parts.push(Part::ClosingOf(intermediate));
    }
}

/// The bridge's per-type metadata (prism_bridge.rb extract_metadata), minus
/// the keys nothing downstream reads (parameters_count, message, content,
/// value). All values stay strings for adapter-port parity.
fn extract_metadata(node: &PrismNode<'_>) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    match node {
        PrismNode::ClassNode { .. } => {
            let n = node.as_class_node().unwrap();
            metadata.insert(
                "name".to_string(),
                class_or_module_name(&n.constant_path(), &n.name()),
            );
            if let Some(superclass) = n.superclass() {
                metadata.insert("superclass".to_string(), superclass_name(&superclass));
            }
        }
        PrismNode::ModuleNode { .. } => {
            let n = node.as_module_node().unwrap();
            metadata.insert(
                "name".to_string(),
                class_or_module_name(&n.constant_path(), &n.name()),
            );
        }
        PrismNode::DefNode { .. } => {
            let n = node.as_def_node().unwrap();
            // name_loc slice rather than the name symbol, so unary operator
            // suffixes survive (prism normalizes `def !@` to :!).
            metadata.insert("name".to_string(), slice_string(&n.name_loc()));
            if let Some(parameters) = n.parameters() {
                metadata.insert(
                    "parameters_text".to_string(),
                    slice_string(&parameters.location()),
                );
                metadata.insert(
                    "has_parens".to_string(),
                    n.lparen_loc().is_some().to_string(),
                );
            }
            if let Some(receiver) = n.receiver() {
                let value = if receiver.as_self_node().is_some() {
                    "self".to_string()
                } else {
                    slice_string(&receiver.location())
                };
                metadata.insert("receiver".to_string(), value);
            }
        }
        PrismNode::CallNode { .. } => {
            let n = node.as_call_node().unwrap();
            metadata.insert("name".to_string(), constant_id_string(&n.name()));
        }
        PrismNode::LocalVariableWriteNode { .. } => {
            let n = node.as_local_variable_write_node().unwrap();
            metadata.insert("name".to_string(), constant_id_string(&n.name()));
        }
        PrismNode::InstanceVariableWriteNode { .. } => {
            let n = node.as_instance_variable_write_node().unwrap();
            metadata.insert("name".to_string(), constant_id_string(&n.name()));
        }
        // UnlessNode gets no is_ternary key: the bridge guards on
        // respond_to?(:if_keyword_loc), which UnlessNode lacks.
        PrismNode::IfNode { .. } => {
            let n = node.as_if_node().unwrap();
            metadata.insert(
                "is_ternary".to_string(),
                n.if_keyword_loc().is_none().to_string(),
            );
        }
        _ => {}
    }

    metadata
}

fn slice_string(loc: &PrismLocation<'_>) -> String {
    String::from_utf8_lossy(loc.as_slice()).into_owned()
}

fn constant_id_string(id: &ConstantId<'_>) -> String {
    String::from_utf8_lossy(id.as_slice()).into_owned()
}

/// prism_node_extractor.rb extract_class_or_module_name: constant_path is
/// always present, `name` is the bridge's fallback for exotic paths.
fn class_or_module_name(constant_path: &PrismNode<'_>, name: &ConstantId<'_>) -> String {
    if let Some(read) = constant_path.as_constant_read_node() {
        constant_id_string(&read.name())
    } else if let Some(path) = constant_path.as_constant_path_node() {
        constant_path_full_name(&path).unwrap_or_else(|| slice_string(&constant_path.location()))
    } else {
        constant_id_string(name)
    }
}

/// prism_node_extractor.rb extract_superclass_name.
fn superclass_name(superclass: &PrismNode<'_>) -> String {
    if let Some(read) = superclass.as_constant_read_node() {
        constant_id_string(&read.name())
    } else if let Some(path) = superclass.as_constant_path_node() {
        constant_path_full_name(&path).unwrap_or_else(|| slice_string(&superclass.location()))
    } else {
        // CallNode superclasses (`ActiveRecord::Migration[8.1]`) and the
        // bridge's generic fallback both take the source slice.
        slice_string(&superclass.location())
    }
}

/// The prism gem's ConstantPathNode#full_name. Divergence: where the gem
/// raises (dynamic parts like `self::Foo`, or missing names) and so aborts
/// the whole bridge parse, this returns None and the callers fall back to
/// the source slice.
fn constant_path_full_name(path: &ruby_prism::ConstantPathNode<'_>) -> Option<String> {
    let mut parts = vec![constant_id_string(&path.name()?)];
    let mut parent = path.parent();
    loop {
        match parent {
            // Root scope (`::Foo`): the gem prepends an empty part.
            None => {
                parts.push(String::new());
                break;
            }
            Some(node) => {
                if let Some(inner) = node.as_constant_path_node() {
                    parts.push(constant_id_string(&inner.name()?));
                    parent = inner.parent();
                } else {
                    let read = node.as_constant_read_node()?;
                    parts.push(constant_id_string(&read.name()));
                    break;
                }
            }
        }
    }
    parts.reverse();
    Some(parts.join("::"))
}

/// Direct children of a node in prism's generic child_nodes order. The crate
/// has no generic child accessor, so every typed visit method is overridden
/// to record the child without descending, and the node dispatches to its
/// generated walk function (all arms generated from the Node enum variants).
fn direct_children<'pr>(node: &PrismNode<'pr>) -> Vec<PrismNode<'pr>> {
    struct Collector<'pr> {
        children: Vec<PrismNode<'pr>>,
    }

    impl<'pr> Visit<'pr> for Collector<'pr> {
        fn visit_alias_global_variable_node(
            &mut self,
            node: &ruby_prism::AliasGlobalVariableNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_alias_method_node(&mut self, node: &ruby_prism::AliasMethodNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_alternation_pattern_node(
            &mut self,
            node: &ruby_prism::AlternationPatternNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_and_node(&mut self, node: &ruby_prism::AndNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_arguments_node(&mut self, node: &ruby_prism::ArgumentsNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_back_reference_read_node(
            &mut self,
            node: &ruby_prism::BackReferenceReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_block_local_variable_node(
            &mut self,
            node: &ruby_prism::BlockLocalVariableNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_break_node(&mut self, node: &ruby_prism::BreakNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_call_and_write_node(&mut self, node: &ruby_prism::CallAndWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_call_operator_write_node(
            &mut self,
            node: &ruby_prism::CallOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_call_or_write_node(&mut self, node: &ruby_prism::CallOrWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_call_target_node(&mut self, node: &ruby_prism::CallTargetNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_capture_pattern_node(&mut self, node: &ruby_prism::CapturePatternNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_and_write_node(
            &mut self,
            node: &ruby_prism::ClassVariableAndWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::ClassVariableOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_or_write_node(
            &mut self,
            node: &ruby_prism::ClassVariableOrWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_read_node(
            &mut self,
            node: &ruby_prism::ClassVariableReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_target_node(
            &mut self,
            node: &ruby_prism::ClassVariableTargetNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_class_variable_write_node(
            &mut self,
            node: &ruby_prism::ClassVariableWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_and_write_node(&mut self, node: &ruby_prism::ConstantAndWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_constant_operator_write_node(
            &mut self,
            node: &ruby_prism::ConstantOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_or_write_node(&mut self, node: &ruby_prism::ConstantOrWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_and_write_node(
            &mut self,
            node: &ruby_prism::ConstantPathAndWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_node(&mut self, node: &ruby_prism::ConstantPathNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_operator_write_node(
            &mut self,
            node: &ruby_prism::ConstantPathOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_or_write_node(
            &mut self,
            node: &ruby_prism::ConstantPathOrWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_target_node(
            &mut self,
            node: &ruby_prism::ConstantPathTargetNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_path_write_node(
            &mut self,
            node: &ruby_prism::ConstantPathWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_constant_read_node(&mut self, node: &ruby_prism::ConstantReadNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_constant_target_node(&mut self, node: &ruby_prism::ConstantTargetNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_defined_node(&mut self, node: &ruby_prism::DefinedNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_else_node(&mut self, node: &ruby_prism::ElseNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_embedded_statements_node(
            &mut self,
            node: &ruby_prism::EmbeddedStatementsNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_false_node(&mut self, node: &ruby_prism::FalseNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_find_pattern_node(&mut self, node: &ruby_prism::FindPatternNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_flip_flop_node(&mut self, node: &ruby_prism::FlipFlopNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_float_node(&mut self, node: &ruby_prism::FloatNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_for_node(&mut self, node: &ruby_prism::ForNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_forwarding_arguments_node(
            &mut self,
            node: &ruby_prism::ForwardingArgumentsNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_forwarding_parameter_node(
            &mut self,
            node: &ruby_prism::ForwardingParameterNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_and_write_node(
            &mut self,
            node: &ruby_prism::GlobalVariableAndWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::GlobalVariableOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_or_write_node(
            &mut self,
            node: &ruby_prism::GlobalVariableOrWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_read_node(
            &mut self,
            node: &ruby_prism::GlobalVariableReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_target_node(
            &mut self,
            node: &ruby_prism::GlobalVariableTargetNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_global_variable_write_node(
            &mut self,
            node: &ruby_prism::GlobalVariableWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_hash_node(&mut self, node: &ruby_prism::HashNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_implicit_node(&mut self, node: &ruby_prism::ImplicitNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_implicit_rest_node(&mut self, node: &ruby_prism::ImplicitRestNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_in_node(&mut self, node: &ruby_prism::InNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_index_and_write_node(&mut self, node: &ruby_prism::IndexAndWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_index_operator_write_node(
            &mut self,
            node: &ruby_prism::IndexOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_index_or_write_node(&mut self, node: &ruby_prism::IndexOrWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_index_target_node(&mut self, node: &ruby_prism::IndexTargetNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_and_write_node(
            &mut self,
            node: &ruby_prism::InstanceVariableAndWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::InstanceVariableOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_or_write_node(
            &mut self,
            node: &ruby_prism::InstanceVariableOrWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_read_node(
            &mut self,
            node: &ruby_prism::InstanceVariableReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_target_node(
            &mut self,
            node: &ruby_prism::InstanceVariableTargetNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_instance_variable_write_node(
            &mut self,
            node: &ruby_prism::InstanceVariableWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_interpolated_match_last_line_node(
            &mut self,
            node: &ruby_prism::InterpolatedMatchLastLineNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_interpolated_regular_expression_node(
            &mut self,
            node: &ruby_prism::InterpolatedRegularExpressionNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_interpolated_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_interpolated_symbol_node(
            &mut self,
            node: &ruby_prism::InterpolatedSymbolNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_interpolated_x_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedXStringNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_it_local_variable_read_node(
            &mut self,
            node: &ruby_prism::ItLocalVariableReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_it_parameters_node(&mut self, node: &ruby_prism::ItParametersNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_keyword_rest_parameter_node(
            &mut self,
            node: &ruby_prism::KeywordRestParameterNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_read_node(
            &mut self,
            node: &ruby_prism::LocalVariableReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_target_node(
            &mut self,
            node: &ruby_prism::LocalVariableTargetNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_missing_node(&mut self, node: &ruby_prism::MissingNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_multi_target_node(&mut self, node: &ruby_prism::MultiTargetNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_next_node(&mut self, node: &ruby_prism::NextNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_nil_node(&mut self, node: &ruby_prism::NilNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_no_keywords_parameter_node(
            &mut self,
            node: &ruby_prism::NoKeywordsParameterNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_numbered_parameters_node(
            &mut self,
            node: &ruby_prism::NumberedParametersNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_numbered_reference_read_node(
            &mut self,
            node: &ruby_prism::NumberedReferenceReadNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_optional_keyword_parameter_node(
            &mut self,
            node: &ruby_prism::OptionalKeywordParameterNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_optional_parameter_node(&mut self, node: &ruby_prism::OptionalParameterNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_or_node(&mut self, node: &ruby_prism::OrNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_parameters_node(&mut self, node: &ruby_prism::ParametersNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_pinned_expression_node(&mut self, node: &ruby_prism::PinnedExpressionNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_pinned_variable_node(&mut self, node: &ruby_prism::PinnedVariableNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_post_execution_node(&mut self, node: &ruby_prism::PostExecutionNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_pre_execution_node(&mut self, node: &ruby_prism::PreExecutionNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_range_node(&mut self, node: &ruby_prism::RangeNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_redo_node(&mut self, node: &ruby_prism::RedoNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_required_keyword_parameter_node(
            &mut self,
            node: &ruby_prism::RequiredKeywordParameterNode<'pr>,
        ) {
            self.children.push(node.as_node());
        }

        fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_retry_node(&mut self, node: &ruby_prism::RetryNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_self_node(&mut self, node: &ruby_prism::SelfNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_shareable_constant_node(&mut self, node: &ruby_prism::ShareableConstantNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_source_encoding_node(&mut self, node: &ruby_prism::SourceEncodingNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_source_file_node(&mut self, node: &ruby_prism::SourceFileNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_source_line_node(&mut self, node: &ruby_prism::SourceLineNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_super_node(&mut self, node: &ruby_prism::SuperNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_true_node(&mut self, node: &ruby_prism::TrueNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_undef_node(&mut self, node: &ruby_prism::UndefNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_until_node(&mut self, node: &ruby_prism::UntilNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_when_node(&mut self, node: &ruby_prism::WhenNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_while_node(&mut self, node: &ruby_prism::WhileNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode<'pr>) {
            self.children.push(node.as_node());
        }

        fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode<'pr>) {
            self.children.push(node.as_node());
        }
    }

    let mut collector = Collector {
        children: Vec::new(),
    };
    match node {
        PrismNode::AliasGlobalVariableNode { .. } => {
            ruby_prism::visit_alias_global_variable_node(
                &mut collector,
                &node.as_alias_global_variable_node().unwrap(),
            );
        }
        PrismNode::AliasMethodNode { .. } => {
            ruby_prism::visit_alias_method_node(
                &mut collector,
                &node.as_alias_method_node().unwrap(),
            );
        }
        PrismNode::AlternationPatternNode { .. } => {
            ruby_prism::visit_alternation_pattern_node(
                &mut collector,
                &node.as_alternation_pattern_node().unwrap(),
            );
        }
        PrismNode::AndNode { .. } => {
            ruby_prism::visit_and_node(&mut collector, &node.as_and_node().unwrap());
        }
        PrismNode::ArgumentsNode { .. } => {
            ruby_prism::visit_arguments_node(&mut collector, &node.as_arguments_node().unwrap());
        }
        PrismNode::ArrayNode { .. } => {
            ruby_prism::visit_array_node(&mut collector, &node.as_array_node().unwrap());
        }
        PrismNode::ArrayPatternNode { .. } => {
            ruby_prism::visit_array_pattern_node(
                &mut collector,
                &node.as_array_pattern_node().unwrap(),
            );
        }
        PrismNode::AssocNode { .. } => {
            ruby_prism::visit_assoc_node(&mut collector, &node.as_assoc_node().unwrap());
        }
        PrismNode::AssocSplatNode { .. } => {
            ruby_prism::visit_assoc_splat_node(
                &mut collector,
                &node.as_assoc_splat_node().unwrap(),
            );
        }
        PrismNode::BackReferenceReadNode { .. } => {
            ruby_prism::visit_back_reference_read_node(
                &mut collector,
                &node.as_back_reference_read_node().unwrap(),
            );
        }
        PrismNode::BeginNode { .. } => {
            ruby_prism::visit_begin_node(&mut collector, &node.as_begin_node().unwrap());
        }
        PrismNode::BlockArgumentNode { .. } => {
            ruby_prism::visit_block_argument_node(
                &mut collector,
                &node.as_block_argument_node().unwrap(),
            );
        }
        PrismNode::BlockLocalVariableNode { .. } => {
            ruby_prism::visit_block_local_variable_node(
                &mut collector,
                &node.as_block_local_variable_node().unwrap(),
            );
        }
        PrismNode::BlockNode { .. } => {
            ruby_prism::visit_block_node(&mut collector, &node.as_block_node().unwrap());
        }
        PrismNode::BlockParameterNode { .. } => {
            ruby_prism::visit_block_parameter_node(
                &mut collector,
                &node.as_block_parameter_node().unwrap(),
            );
        }
        PrismNode::BlockParametersNode { .. } => {
            ruby_prism::visit_block_parameters_node(
                &mut collector,
                &node.as_block_parameters_node().unwrap(),
            );
        }
        PrismNode::BreakNode { .. } => {
            ruby_prism::visit_break_node(&mut collector, &node.as_break_node().unwrap());
        }
        PrismNode::CallAndWriteNode { .. } => {
            ruby_prism::visit_call_and_write_node(
                &mut collector,
                &node.as_call_and_write_node().unwrap(),
            );
        }
        PrismNode::CallNode { .. } => {
            ruby_prism::visit_call_node(&mut collector, &node.as_call_node().unwrap());
        }
        PrismNode::CallOperatorWriteNode { .. } => {
            ruby_prism::visit_call_operator_write_node(
                &mut collector,
                &node.as_call_operator_write_node().unwrap(),
            );
        }
        PrismNode::CallOrWriteNode { .. } => {
            ruby_prism::visit_call_or_write_node(
                &mut collector,
                &node.as_call_or_write_node().unwrap(),
            );
        }
        PrismNode::CallTargetNode { .. } => {
            ruby_prism::visit_call_target_node(
                &mut collector,
                &node.as_call_target_node().unwrap(),
            );
        }
        PrismNode::CapturePatternNode { .. } => {
            ruby_prism::visit_capture_pattern_node(
                &mut collector,
                &node.as_capture_pattern_node().unwrap(),
            );
        }
        PrismNode::CaseMatchNode { .. } => {
            ruby_prism::visit_case_match_node(&mut collector, &node.as_case_match_node().unwrap());
        }
        PrismNode::CaseNode { .. } => {
            ruby_prism::visit_case_node(&mut collector, &node.as_case_node().unwrap());
        }
        PrismNode::ClassNode { .. } => {
            ruby_prism::visit_class_node(&mut collector, &node.as_class_node().unwrap());
        }
        PrismNode::ClassVariableAndWriteNode { .. } => {
            ruby_prism::visit_class_variable_and_write_node(
                &mut collector,
                &node.as_class_variable_and_write_node().unwrap(),
            );
        }
        PrismNode::ClassVariableOperatorWriteNode { .. } => {
            ruby_prism::visit_class_variable_operator_write_node(
                &mut collector,
                &node.as_class_variable_operator_write_node().unwrap(),
            );
        }
        PrismNode::ClassVariableOrWriteNode { .. } => {
            ruby_prism::visit_class_variable_or_write_node(
                &mut collector,
                &node.as_class_variable_or_write_node().unwrap(),
            );
        }
        PrismNode::ClassVariableReadNode { .. } => {
            ruby_prism::visit_class_variable_read_node(
                &mut collector,
                &node.as_class_variable_read_node().unwrap(),
            );
        }
        PrismNode::ClassVariableTargetNode { .. } => {
            ruby_prism::visit_class_variable_target_node(
                &mut collector,
                &node.as_class_variable_target_node().unwrap(),
            );
        }
        PrismNode::ClassVariableWriteNode { .. } => {
            ruby_prism::visit_class_variable_write_node(
                &mut collector,
                &node.as_class_variable_write_node().unwrap(),
            );
        }
        PrismNode::ConstantAndWriteNode { .. } => {
            ruby_prism::visit_constant_and_write_node(
                &mut collector,
                &node.as_constant_and_write_node().unwrap(),
            );
        }
        PrismNode::ConstantOperatorWriteNode { .. } => {
            ruby_prism::visit_constant_operator_write_node(
                &mut collector,
                &node.as_constant_operator_write_node().unwrap(),
            );
        }
        PrismNode::ConstantOrWriteNode { .. } => {
            ruby_prism::visit_constant_or_write_node(
                &mut collector,
                &node.as_constant_or_write_node().unwrap(),
            );
        }
        PrismNode::ConstantPathAndWriteNode { .. } => {
            ruby_prism::visit_constant_path_and_write_node(
                &mut collector,
                &node.as_constant_path_and_write_node().unwrap(),
            );
        }
        PrismNode::ConstantPathNode { .. } => {
            ruby_prism::visit_constant_path_node(
                &mut collector,
                &node.as_constant_path_node().unwrap(),
            );
        }
        PrismNode::ConstantPathOperatorWriteNode { .. } => {
            ruby_prism::visit_constant_path_operator_write_node(
                &mut collector,
                &node.as_constant_path_operator_write_node().unwrap(),
            );
        }
        PrismNode::ConstantPathOrWriteNode { .. } => {
            ruby_prism::visit_constant_path_or_write_node(
                &mut collector,
                &node.as_constant_path_or_write_node().unwrap(),
            );
        }
        PrismNode::ConstantPathTargetNode { .. } => {
            ruby_prism::visit_constant_path_target_node(
                &mut collector,
                &node.as_constant_path_target_node().unwrap(),
            );
        }
        PrismNode::ConstantPathWriteNode { .. } => {
            ruby_prism::visit_constant_path_write_node(
                &mut collector,
                &node.as_constant_path_write_node().unwrap(),
            );
        }
        PrismNode::ConstantReadNode { .. } => {
            ruby_prism::visit_constant_read_node(
                &mut collector,
                &node.as_constant_read_node().unwrap(),
            );
        }
        PrismNode::ConstantTargetNode { .. } => {
            ruby_prism::visit_constant_target_node(
                &mut collector,
                &node.as_constant_target_node().unwrap(),
            );
        }
        PrismNode::ConstantWriteNode { .. } => {
            ruby_prism::visit_constant_write_node(
                &mut collector,
                &node.as_constant_write_node().unwrap(),
            );
        }
        PrismNode::DefNode { .. } => {
            ruby_prism::visit_def_node(&mut collector, &node.as_def_node().unwrap());
        }
        PrismNode::DefinedNode { .. } => {
            ruby_prism::visit_defined_node(&mut collector, &node.as_defined_node().unwrap());
        }
        PrismNode::ElseNode { .. } => {
            ruby_prism::visit_else_node(&mut collector, &node.as_else_node().unwrap());
        }
        PrismNode::EmbeddedStatementsNode { .. } => {
            ruby_prism::visit_embedded_statements_node(
                &mut collector,
                &node.as_embedded_statements_node().unwrap(),
            );
        }
        PrismNode::EmbeddedVariableNode { .. } => {
            ruby_prism::visit_embedded_variable_node(
                &mut collector,
                &node.as_embedded_variable_node().unwrap(),
            );
        }
        PrismNode::EnsureNode { .. } => {
            ruby_prism::visit_ensure_node(&mut collector, &node.as_ensure_node().unwrap());
        }
        PrismNode::FalseNode { .. } => {
            ruby_prism::visit_false_node(&mut collector, &node.as_false_node().unwrap());
        }
        PrismNode::FindPatternNode { .. } => {
            ruby_prism::visit_find_pattern_node(
                &mut collector,
                &node.as_find_pattern_node().unwrap(),
            );
        }
        PrismNode::FlipFlopNode { .. } => {
            ruby_prism::visit_flip_flop_node(&mut collector, &node.as_flip_flop_node().unwrap());
        }
        PrismNode::FloatNode { .. } => {
            ruby_prism::visit_float_node(&mut collector, &node.as_float_node().unwrap());
        }
        PrismNode::ForNode { .. } => {
            ruby_prism::visit_for_node(&mut collector, &node.as_for_node().unwrap());
        }
        PrismNode::ForwardingArgumentsNode { .. } => {
            ruby_prism::visit_forwarding_arguments_node(
                &mut collector,
                &node.as_forwarding_arguments_node().unwrap(),
            );
        }
        PrismNode::ForwardingParameterNode { .. } => {
            ruby_prism::visit_forwarding_parameter_node(
                &mut collector,
                &node.as_forwarding_parameter_node().unwrap(),
            );
        }
        PrismNode::ForwardingSuperNode { .. } => {
            ruby_prism::visit_forwarding_super_node(
                &mut collector,
                &node.as_forwarding_super_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableAndWriteNode { .. } => {
            ruby_prism::visit_global_variable_and_write_node(
                &mut collector,
                &node.as_global_variable_and_write_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableOperatorWriteNode { .. } => {
            ruby_prism::visit_global_variable_operator_write_node(
                &mut collector,
                &node.as_global_variable_operator_write_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableOrWriteNode { .. } => {
            ruby_prism::visit_global_variable_or_write_node(
                &mut collector,
                &node.as_global_variable_or_write_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableReadNode { .. } => {
            ruby_prism::visit_global_variable_read_node(
                &mut collector,
                &node.as_global_variable_read_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableTargetNode { .. } => {
            ruby_prism::visit_global_variable_target_node(
                &mut collector,
                &node.as_global_variable_target_node().unwrap(),
            );
        }
        PrismNode::GlobalVariableWriteNode { .. } => {
            ruby_prism::visit_global_variable_write_node(
                &mut collector,
                &node.as_global_variable_write_node().unwrap(),
            );
        }
        PrismNode::HashNode { .. } => {
            ruby_prism::visit_hash_node(&mut collector, &node.as_hash_node().unwrap());
        }
        PrismNode::HashPatternNode { .. } => {
            ruby_prism::visit_hash_pattern_node(
                &mut collector,
                &node.as_hash_pattern_node().unwrap(),
            );
        }
        PrismNode::IfNode { .. } => {
            ruby_prism::visit_if_node(&mut collector, &node.as_if_node().unwrap());
        }
        PrismNode::ImaginaryNode { .. } => {
            ruby_prism::visit_imaginary_node(&mut collector, &node.as_imaginary_node().unwrap());
        }
        PrismNode::ImplicitNode { .. } => {
            ruby_prism::visit_implicit_node(&mut collector, &node.as_implicit_node().unwrap());
        }
        PrismNode::ImplicitRestNode { .. } => {
            ruby_prism::visit_implicit_rest_node(
                &mut collector,
                &node.as_implicit_rest_node().unwrap(),
            );
        }
        PrismNode::InNode { .. } => {
            ruby_prism::visit_in_node(&mut collector, &node.as_in_node().unwrap());
        }
        PrismNode::IndexAndWriteNode { .. } => {
            ruby_prism::visit_index_and_write_node(
                &mut collector,
                &node.as_index_and_write_node().unwrap(),
            );
        }
        PrismNode::IndexOperatorWriteNode { .. } => {
            ruby_prism::visit_index_operator_write_node(
                &mut collector,
                &node.as_index_operator_write_node().unwrap(),
            );
        }
        PrismNode::IndexOrWriteNode { .. } => {
            ruby_prism::visit_index_or_write_node(
                &mut collector,
                &node.as_index_or_write_node().unwrap(),
            );
        }
        PrismNode::IndexTargetNode { .. } => {
            ruby_prism::visit_index_target_node(
                &mut collector,
                &node.as_index_target_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableAndWriteNode { .. } => {
            ruby_prism::visit_instance_variable_and_write_node(
                &mut collector,
                &node.as_instance_variable_and_write_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableOperatorWriteNode { .. } => {
            ruby_prism::visit_instance_variable_operator_write_node(
                &mut collector,
                &node.as_instance_variable_operator_write_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableOrWriteNode { .. } => {
            ruby_prism::visit_instance_variable_or_write_node(
                &mut collector,
                &node.as_instance_variable_or_write_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableReadNode { .. } => {
            ruby_prism::visit_instance_variable_read_node(
                &mut collector,
                &node.as_instance_variable_read_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableTargetNode { .. } => {
            ruby_prism::visit_instance_variable_target_node(
                &mut collector,
                &node.as_instance_variable_target_node().unwrap(),
            );
        }
        PrismNode::InstanceVariableWriteNode { .. } => {
            ruby_prism::visit_instance_variable_write_node(
                &mut collector,
                &node.as_instance_variable_write_node().unwrap(),
            );
        }
        PrismNode::IntegerNode { .. } => {
            ruby_prism::visit_integer_node(&mut collector, &node.as_integer_node().unwrap());
        }
        PrismNode::InterpolatedMatchLastLineNode { .. } => {
            ruby_prism::visit_interpolated_match_last_line_node(
                &mut collector,
                &node.as_interpolated_match_last_line_node().unwrap(),
            );
        }
        PrismNode::InterpolatedRegularExpressionNode { .. } => {
            ruby_prism::visit_interpolated_regular_expression_node(
                &mut collector,
                &node.as_interpolated_regular_expression_node().unwrap(),
            );
        }
        PrismNode::InterpolatedStringNode { .. } => {
            ruby_prism::visit_interpolated_string_node(
                &mut collector,
                &node.as_interpolated_string_node().unwrap(),
            );
        }
        PrismNode::InterpolatedSymbolNode { .. } => {
            ruby_prism::visit_interpolated_symbol_node(
                &mut collector,
                &node.as_interpolated_symbol_node().unwrap(),
            );
        }
        PrismNode::InterpolatedXStringNode { .. } => {
            ruby_prism::visit_interpolated_x_string_node(
                &mut collector,
                &node.as_interpolated_x_string_node().unwrap(),
            );
        }
        PrismNode::ItLocalVariableReadNode { .. } => {
            ruby_prism::visit_it_local_variable_read_node(
                &mut collector,
                &node.as_it_local_variable_read_node().unwrap(),
            );
        }
        PrismNode::ItParametersNode { .. } => {
            ruby_prism::visit_it_parameters_node(
                &mut collector,
                &node.as_it_parameters_node().unwrap(),
            );
        }
        PrismNode::KeywordHashNode { .. } => {
            ruby_prism::visit_keyword_hash_node(
                &mut collector,
                &node.as_keyword_hash_node().unwrap(),
            );
        }
        PrismNode::KeywordRestParameterNode { .. } => {
            ruby_prism::visit_keyword_rest_parameter_node(
                &mut collector,
                &node.as_keyword_rest_parameter_node().unwrap(),
            );
        }
        PrismNode::LambdaNode { .. } => {
            ruby_prism::visit_lambda_node(&mut collector, &node.as_lambda_node().unwrap());
        }
        PrismNode::LocalVariableAndWriteNode { .. } => {
            ruby_prism::visit_local_variable_and_write_node(
                &mut collector,
                &node.as_local_variable_and_write_node().unwrap(),
            );
        }
        PrismNode::LocalVariableOperatorWriteNode { .. } => {
            ruby_prism::visit_local_variable_operator_write_node(
                &mut collector,
                &node.as_local_variable_operator_write_node().unwrap(),
            );
        }
        PrismNode::LocalVariableOrWriteNode { .. } => {
            ruby_prism::visit_local_variable_or_write_node(
                &mut collector,
                &node.as_local_variable_or_write_node().unwrap(),
            );
        }
        PrismNode::LocalVariableReadNode { .. } => {
            ruby_prism::visit_local_variable_read_node(
                &mut collector,
                &node.as_local_variable_read_node().unwrap(),
            );
        }
        PrismNode::LocalVariableTargetNode { .. } => {
            ruby_prism::visit_local_variable_target_node(
                &mut collector,
                &node.as_local_variable_target_node().unwrap(),
            );
        }
        PrismNode::LocalVariableWriteNode { .. } => {
            ruby_prism::visit_local_variable_write_node(
                &mut collector,
                &node.as_local_variable_write_node().unwrap(),
            );
        }
        PrismNode::MatchLastLineNode { .. } => {
            ruby_prism::visit_match_last_line_node(
                &mut collector,
                &node.as_match_last_line_node().unwrap(),
            );
        }
        PrismNode::MatchPredicateNode { .. } => {
            ruby_prism::visit_match_predicate_node(
                &mut collector,
                &node.as_match_predicate_node().unwrap(),
            );
        }
        PrismNode::MatchRequiredNode { .. } => {
            ruby_prism::visit_match_required_node(
                &mut collector,
                &node.as_match_required_node().unwrap(),
            );
        }
        PrismNode::MatchWriteNode { .. } => {
            ruby_prism::visit_match_write_node(
                &mut collector,
                &node.as_match_write_node().unwrap(),
            );
        }
        PrismNode::MissingNode { .. } => {
            ruby_prism::visit_missing_node(&mut collector, &node.as_missing_node().unwrap());
        }
        PrismNode::ModuleNode { .. } => {
            ruby_prism::visit_module_node(&mut collector, &node.as_module_node().unwrap());
        }
        PrismNode::MultiTargetNode { .. } => {
            ruby_prism::visit_multi_target_node(
                &mut collector,
                &node.as_multi_target_node().unwrap(),
            );
        }
        PrismNode::MultiWriteNode { .. } => {
            ruby_prism::visit_multi_write_node(
                &mut collector,
                &node.as_multi_write_node().unwrap(),
            );
        }
        PrismNode::NextNode { .. } => {
            ruby_prism::visit_next_node(&mut collector, &node.as_next_node().unwrap());
        }
        PrismNode::NilNode { .. } => {
            ruby_prism::visit_nil_node(&mut collector, &node.as_nil_node().unwrap());
        }
        PrismNode::NoKeywordsParameterNode { .. } => {
            ruby_prism::visit_no_keywords_parameter_node(
                &mut collector,
                &node.as_no_keywords_parameter_node().unwrap(),
            );
        }
        PrismNode::NumberedParametersNode { .. } => {
            ruby_prism::visit_numbered_parameters_node(
                &mut collector,
                &node.as_numbered_parameters_node().unwrap(),
            );
        }
        PrismNode::NumberedReferenceReadNode { .. } => {
            ruby_prism::visit_numbered_reference_read_node(
                &mut collector,
                &node.as_numbered_reference_read_node().unwrap(),
            );
        }
        PrismNode::OptionalKeywordParameterNode { .. } => {
            ruby_prism::visit_optional_keyword_parameter_node(
                &mut collector,
                &node.as_optional_keyword_parameter_node().unwrap(),
            );
        }
        PrismNode::OptionalParameterNode { .. } => {
            ruby_prism::visit_optional_parameter_node(
                &mut collector,
                &node.as_optional_parameter_node().unwrap(),
            );
        }
        PrismNode::OrNode { .. } => {
            ruby_prism::visit_or_node(&mut collector, &node.as_or_node().unwrap());
        }
        PrismNode::ParametersNode { .. } => {
            ruby_prism::visit_parameters_node(&mut collector, &node.as_parameters_node().unwrap());
        }
        PrismNode::ParenthesesNode { .. } => {
            ruby_prism::visit_parentheses_node(
                &mut collector,
                &node.as_parentheses_node().unwrap(),
            );
        }
        PrismNode::PinnedExpressionNode { .. } => {
            ruby_prism::visit_pinned_expression_node(
                &mut collector,
                &node.as_pinned_expression_node().unwrap(),
            );
        }
        PrismNode::PinnedVariableNode { .. } => {
            ruby_prism::visit_pinned_variable_node(
                &mut collector,
                &node.as_pinned_variable_node().unwrap(),
            );
        }
        PrismNode::PostExecutionNode { .. } => {
            ruby_prism::visit_post_execution_node(
                &mut collector,
                &node.as_post_execution_node().unwrap(),
            );
        }
        PrismNode::PreExecutionNode { .. } => {
            ruby_prism::visit_pre_execution_node(
                &mut collector,
                &node.as_pre_execution_node().unwrap(),
            );
        }
        PrismNode::ProgramNode { .. } => {
            ruby_prism::visit_program_node(&mut collector, &node.as_program_node().unwrap());
        }
        PrismNode::RangeNode { .. } => {
            ruby_prism::visit_range_node(&mut collector, &node.as_range_node().unwrap());
        }
        PrismNode::RationalNode { .. } => {
            ruby_prism::visit_rational_node(&mut collector, &node.as_rational_node().unwrap());
        }
        PrismNode::RedoNode { .. } => {
            ruby_prism::visit_redo_node(&mut collector, &node.as_redo_node().unwrap());
        }
        PrismNode::RegularExpressionNode { .. } => {
            ruby_prism::visit_regular_expression_node(
                &mut collector,
                &node.as_regular_expression_node().unwrap(),
            );
        }
        PrismNode::RequiredKeywordParameterNode { .. } => {
            ruby_prism::visit_required_keyword_parameter_node(
                &mut collector,
                &node.as_required_keyword_parameter_node().unwrap(),
            );
        }
        PrismNode::RequiredParameterNode { .. } => {
            ruby_prism::visit_required_parameter_node(
                &mut collector,
                &node.as_required_parameter_node().unwrap(),
            );
        }
        PrismNode::RescueModifierNode { .. } => {
            ruby_prism::visit_rescue_modifier_node(
                &mut collector,
                &node.as_rescue_modifier_node().unwrap(),
            );
        }
        PrismNode::RescueNode { .. } => {
            ruby_prism::visit_rescue_node(&mut collector, &node.as_rescue_node().unwrap());
        }
        PrismNode::RestParameterNode { .. } => {
            ruby_prism::visit_rest_parameter_node(
                &mut collector,
                &node.as_rest_parameter_node().unwrap(),
            );
        }
        PrismNode::RetryNode { .. } => {
            ruby_prism::visit_retry_node(&mut collector, &node.as_retry_node().unwrap());
        }
        PrismNode::ReturnNode { .. } => {
            ruby_prism::visit_return_node(&mut collector, &node.as_return_node().unwrap());
        }
        PrismNode::SelfNode { .. } => {
            ruby_prism::visit_self_node(&mut collector, &node.as_self_node().unwrap());
        }
        PrismNode::ShareableConstantNode { .. } => {
            ruby_prism::visit_shareable_constant_node(
                &mut collector,
                &node.as_shareable_constant_node().unwrap(),
            );
        }
        PrismNode::SingletonClassNode { .. } => {
            ruby_prism::visit_singleton_class_node(
                &mut collector,
                &node.as_singleton_class_node().unwrap(),
            );
        }
        PrismNode::SourceEncodingNode { .. } => {
            ruby_prism::visit_source_encoding_node(
                &mut collector,
                &node.as_source_encoding_node().unwrap(),
            );
        }
        PrismNode::SourceFileNode { .. } => {
            ruby_prism::visit_source_file_node(
                &mut collector,
                &node.as_source_file_node().unwrap(),
            );
        }
        PrismNode::SourceLineNode { .. } => {
            ruby_prism::visit_source_line_node(
                &mut collector,
                &node.as_source_line_node().unwrap(),
            );
        }
        PrismNode::SplatNode { .. } => {
            ruby_prism::visit_splat_node(&mut collector, &node.as_splat_node().unwrap());
        }
        PrismNode::StatementsNode { .. } => {
            ruby_prism::visit_statements_node(&mut collector, &node.as_statements_node().unwrap());
        }
        PrismNode::StringNode { .. } => {
            ruby_prism::visit_string_node(&mut collector, &node.as_string_node().unwrap());
        }
        PrismNode::SuperNode { .. } => {
            ruby_prism::visit_super_node(&mut collector, &node.as_super_node().unwrap());
        }
        PrismNode::SymbolNode { .. } => {
            ruby_prism::visit_symbol_node(&mut collector, &node.as_symbol_node().unwrap());
        }
        PrismNode::TrueNode { .. } => {
            ruby_prism::visit_true_node(&mut collector, &node.as_true_node().unwrap());
        }
        PrismNode::UndefNode { .. } => {
            ruby_prism::visit_undef_node(&mut collector, &node.as_undef_node().unwrap());
        }
        PrismNode::UnlessNode { .. } => {
            ruby_prism::visit_unless_node(&mut collector, &node.as_unless_node().unwrap());
        }
        PrismNode::UntilNode { .. } => {
            ruby_prism::visit_until_node(&mut collector, &node.as_until_node().unwrap());
        }
        PrismNode::WhenNode { .. } => {
            ruby_prism::visit_when_node(&mut collector, &node.as_when_node().unwrap());
        }
        PrismNode::WhileNode { .. } => {
            ruby_prism::visit_while_node(&mut collector, &node.as_while_node().unwrap());
        }
        PrismNode::XStringNode { .. } => {
            ruby_prism::visit_x_string_node(&mut collector, &node.as_x_string_node().unwrap());
        }
        PrismNode::YieldNode { .. } => {
            ruby_prism::visit_yield_node(&mut collector, &node.as_yield_node().unwrap());
        }
    }
    collector.children
}

/// `closing_loc` for every node kind that has one, mirroring the bridge's
/// `respond_to?(:closing_loc)` (both lists are generated from prism's
/// config.yml, so the coverage is the complete set for this prism version).
fn own_closing_loc<'pr>(node: &PrismNode<'pr>) -> Option<PrismLocation<'pr>> {
    match node {
        PrismNode::ArrayNode { .. } => node.as_array_node().unwrap().closing_loc(),
        PrismNode::ArrayPatternNode { .. } => node.as_array_pattern_node().unwrap().closing_loc(),
        PrismNode::BlockNode { .. } => Some(node.as_block_node().unwrap().closing_loc()),
        PrismNode::BlockParametersNode { .. } => {
            node.as_block_parameters_node().unwrap().closing_loc()
        }
        PrismNode::CallNode { .. } => node.as_call_node().unwrap().closing_loc(),
        PrismNode::EmbeddedStatementsNode { .. } => {
            Some(node.as_embedded_statements_node().unwrap().closing_loc())
        }
        PrismNode::FindPatternNode { .. } => node.as_find_pattern_node().unwrap().closing_loc(),
        PrismNode::HashNode { .. } => Some(node.as_hash_node().unwrap().closing_loc()),
        PrismNode::HashPatternNode { .. } => node.as_hash_pattern_node().unwrap().closing_loc(),
        PrismNode::IndexAndWriteNode { .. } => {
            Some(node.as_index_and_write_node().unwrap().closing_loc())
        }
        PrismNode::IndexOperatorWriteNode { .. } => {
            Some(node.as_index_operator_write_node().unwrap().closing_loc())
        }
        PrismNode::IndexOrWriteNode { .. } => {
            Some(node.as_index_or_write_node().unwrap().closing_loc())
        }
        PrismNode::IndexTargetNode { .. } => {
            Some(node.as_index_target_node().unwrap().closing_loc())
        }
        PrismNode::InterpolatedMatchLastLineNode { .. } => Some(
            node.as_interpolated_match_last_line_node()
                .unwrap()
                .closing_loc(),
        ),
        PrismNode::InterpolatedRegularExpressionNode { .. } => Some(
            node.as_interpolated_regular_expression_node()
                .unwrap()
                .closing_loc(),
        ),
        PrismNode::InterpolatedStringNode { .. } => {
            node.as_interpolated_string_node().unwrap().closing_loc()
        }
        PrismNode::InterpolatedSymbolNode { .. } => {
            node.as_interpolated_symbol_node().unwrap().closing_loc()
        }
        PrismNode::InterpolatedXStringNode { .. } => {
            Some(node.as_interpolated_x_string_node().unwrap().closing_loc())
        }
        PrismNode::LambdaNode { .. } => Some(node.as_lambda_node().unwrap().closing_loc()),
        PrismNode::MatchLastLineNode { .. } => {
            Some(node.as_match_last_line_node().unwrap().closing_loc())
        }
        PrismNode::ParenthesesNode { .. } => {
            Some(node.as_parentheses_node().unwrap().closing_loc())
        }
        PrismNode::PostExecutionNode { .. } => {
            Some(node.as_post_execution_node().unwrap().closing_loc())
        }
        PrismNode::PreExecutionNode { .. } => {
            Some(node.as_pre_execution_node().unwrap().closing_loc())
        }
        PrismNode::RegularExpressionNode { .. } => {
            Some(node.as_regular_expression_node().unwrap().closing_loc())
        }
        PrismNode::StringNode { .. } => node.as_string_node().unwrap().closing_loc(),
        PrismNode::SymbolNode { .. } => node.as_symbol_node().unwrap().closing_loc(),
        PrismNode::UntilNode { .. } => node.as_until_node().unwrap().closing_loc(),
        PrismNode::WhileNode { .. } => node.as_while_node().unwrap().closing_loc(),
        PrismNode::XStringNode { .. } => Some(node.as_x_string_node().unwrap().closing_loc()),
        _ => None,
    }
}

/// Prism node kind as the snake_cased Ruby class name, matching the bridge's
/// node_type_name. Fed to NodeType::from_str, which falls back to Unknown.
fn node_kind_name(node: &PrismNode<'_>) -> &'static str {
    match node {
        PrismNode::AliasGlobalVariableNode { .. } => "alias_global_variable_node",
        PrismNode::AliasMethodNode { .. } => "alias_method_node",
        PrismNode::AlternationPatternNode { .. } => "alternation_pattern_node",
        PrismNode::AndNode { .. } => "and_node",
        PrismNode::ArgumentsNode { .. } => "arguments_node",
        PrismNode::ArrayNode { .. } => "array_node",
        PrismNode::ArrayPatternNode { .. } => "array_pattern_node",
        PrismNode::AssocNode { .. } => "assoc_node",
        PrismNode::AssocSplatNode { .. } => "assoc_splat_node",
        PrismNode::BackReferenceReadNode { .. } => "back_reference_read_node",
        PrismNode::BeginNode { .. } => "begin_node",
        PrismNode::BlockArgumentNode { .. } => "block_argument_node",
        PrismNode::BlockLocalVariableNode { .. } => "block_local_variable_node",
        PrismNode::BlockNode { .. } => "block_node",
        PrismNode::BlockParameterNode { .. } => "block_parameter_node",
        PrismNode::BlockParametersNode { .. } => "block_parameters_node",
        PrismNode::BreakNode { .. } => "break_node",
        PrismNode::CallAndWriteNode { .. } => "call_and_write_node",
        PrismNode::CallNode { .. } => "call_node",
        PrismNode::CallOperatorWriteNode { .. } => "call_operator_write_node",
        PrismNode::CallOrWriteNode { .. } => "call_or_write_node",
        PrismNode::CallTargetNode { .. } => "call_target_node",
        PrismNode::CapturePatternNode { .. } => "capture_pattern_node",
        PrismNode::CaseMatchNode { .. } => "case_match_node",
        PrismNode::CaseNode { .. } => "case_node",
        PrismNode::ClassNode { .. } => "class_node",
        PrismNode::ClassVariableAndWriteNode { .. } => "class_variable_and_write_node",
        PrismNode::ClassVariableOperatorWriteNode { .. } => "class_variable_operator_write_node",
        PrismNode::ClassVariableOrWriteNode { .. } => "class_variable_or_write_node",
        PrismNode::ClassVariableReadNode { .. } => "class_variable_read_node",
        PrismNode::ClassVariableTargetNode { .. } => "class_variable_target_node",
        PrismNode::ClassVariableWriteNode { .. } => "class_variable_write_node",
        PrismNode::ConstantAndWriteNode { .. } => "constant_and_write_node",
        PrismNode::ConstantOperatorWriteNode { .. } => "constant_operator_write_node",
        PrismNode::ConstantOrWriteNode { .. } => "constant_or_write_node",
        PrismNode::ConstantPathAndWriteNode { .. } => "constant_path_and_write_node",
        PrismNode::ConstantPathNode { .. } => "constant_path_node",
        PrismNode::ConstantPathOperatorWriteNode { .. } => "constant_path_operator_write_node",
        PrismNode::ConstantPathOrWriteNode { .. } => "constant_path_or_write_node",
        PrismNode::ConstantPathTargetNode { .. } => "constant_path_target_node",
        PrismNode::ConstantPathWriteNode { .. } => "constant_path_write_node",
        PrismNode::ConstantReadNode { .. } => "constant_read_node",
        PrismNode::ConstantTargetNode { .. } => "constant_target_node",
        PrismNode::ConstantWriteNode { .. } => "constant_write_node",
        PrismNode::DefNode { .. } => "def_node",
        PrismNode::DefinedNode { .. } => "defined_node",
        PrismNode::ElseNode { .. } => "else_node",
        PrismNode::EmbeddedStatementsNode { .. } => "embedded_statements_node",
        PrismNode::EmbeddedVariableNode { .. } => "embedded_variable_node",
        PrismNode::EnsureNode { .. } => "ensure_node",
        PrismNode::FalseNode { .. } => "false_node",
        PrismNode::FindPatternNode { .. } => "find_pattern_node",
        PrismNode::FlipFlopNode { .. } => "flip_flop_node",
        PrismNode::FloatNode { .. } => "float_node",
        PrismNode::ForNode { .. } => "for_node",
        PrismNode::ForwardingArgumentsNode { .. } => "forwarding_arguments_node",
        PrismNode::ForwardingParameterNode { .. } => "forwarding_parameter_node",
        PrismNode::ForwardingSuperNode { .. } => "forwarding_super_node",
        PrismNode::GlobalVariableAndWriteNode { .. } => "global_variable_and_write_node",
        PrismNode::GlobalVariableOperatorWriteNode { .. } => "global_variable_operator_write_node",
        PrismNode::GlobalVariableOrWriteNode { .. } => "global_variable_or_write_node",
        PrismNode::GlobalVariableReadNode { .. } => "global_variable_read_node",
        PrismNode::GlobalVariableTargetNode { .. } => "global_variable_target_node",
        PrismNode::GlobalVariableWriteNode { .. } => "global_variable_write_node",
        PrismNode::HashNode { .. } => "hash_node",
        PrismNode::HashPatternNode { .. } => "hash_pattern_node",
        PrismNode::IfNode { .. } => "if_node",
        PrismNode::ImaginaryNode { .. } => "imaginary_node",
        PrismNode::ImplicitNode { .. } => "implicit_node",
        PrismNode::ImplicitRestNode { .. } => "implicit_rest_node",
        PrismNode::InNode { .. } => "in_node",
        PrismNode::IndexAndWriteNode { .. } => "index_and_write_node",
        PrismNode::IndexOperatorWriteNode { .. } => "index_operator_write_node",
        PrismNode::IndexOrWriteNode { .. } => "index_or_write_node",
        PrismNode::IndexTargetNode { .. } => "index_target_node",
        PrismNode::InstanceVariableAndWriteNode { .. } => "instance_variable_and_write_node",
        PrismNode::InstanceVariableOperatorWriteNode { .. } => {
            "instance_variable_operator_write_node"
        }
        PrismNode::InstanceVariableOrWriteNode { .. } => "instance_variable_or_write_node",
        PrismNode::InstanceVariableReadNode { .. } => "instance_variable_read_node",
        PrismNode::InstanceVariableTargetNode { .. } => "instance_variable_target_node",
        PrismNode::InstanceVariableWriteNode { .. } => "instance_variable_write_node",
        PrismNode::IntegerNode { .. } => "integer_node",
        PrismNode::InterpolatedMatchLastLineNode { .. } => "interpolated_match_last_line_node",
        PrismNode::InterpolatedRegularExpressionNode { .. } => {
            "interpolated_regular_expression_node"
        }
        PrismNode::InterpolatedStringNode { .. } => "interpolated_string_node",
        PrismNode::InterpolatedSymbolNode { .. } => "interpolated_symbol_node",
        PrismNode::InterpolatedXStringNode { .. } => "interpolated_x_string_node",
        PrismNode::ItLocalVariableReadNode { .. } => "it_local_variable_read_node",
        PrismNode::ItParametersNode { .. } => "it_parameters_node",
        PrismNode::KeywordHashNode { .. } => "keyword_hash_node",
        PrismNode::KeywordRestParameterNode { .. } => "keyword_rest_parameter_node",
        PrismNode::LambdaNode { .. } => "lambda_node",
        PrismNode::LocalVariableAndWriteNode { .. } => "local_variable_and_write_node",
        PrismNode::LocalVariableOperatorWriteNode { .. } => "local_variable_operator_write_node",
        PrismNode::LocalVariableOrWriteNode { .. } => "local_variable_or_write_node",
        PrismNode::LocalVariableReadNode { .. } => "local_variable_read_node",
        PrismNode::LocalVariableTargetNode { .. } => "local_variable_target_node",
        PrismNode::LocalVariableWriteNode { .. } => "local_variable_write_node",
        PrismNode::MatchLastLineNode { .. } => "match_last_line_node",
        PrismNode::MatchPredicateNode { .. } => "match_predicate_node",
        PrismNode::MatchRequiredNode { .. } => "match_required_node",
        PrismNode::MatchWriteNode { .. } => "match_write_node",
        PrismNode::MissingNode { .. } => "missing_node",
        PrismNode::ModuleNode { .. } => "module_node",
        PrismNode::MultiTargetNode { .. } => "multi_target_node",
        PrismNode::MultiWriteNode { .. } => "multi_write_node",
        PrismNode::NextNode { .. } => "next_node",
        PrismNode::NilNode { .. } => "nil_node",
        PrismNode::NoKeywordsParameterNode { .. } => "no_keywords_parameter_node",
        PrismNode::NumberedParametersNode { .. } => "numbered_parameters_node",
        PrismNode::NumberedReferenceReadNode { .. } => "numbered_reference_read_node",
        PrismNode::OptionalKeywordParameterNode { .. } => "optional_keyword_parameter_node",
        PrismNode::OptionalParameterNode { .. } => "optional_parameter_node",
        PrismNode::OrNode { .. } => "or_node",
        PrismNode::ParametersNode { .. } => "parameters_node",
        PrismNode::ParenthesesNode { .. } => "parentheses_node",
        PrismNode::PinnedExpressionNode { .. } => "pinned_expression_node",
        PrismNode::PinnedVariableNode { .. } => "pinned_variable_node",
        PrismNode::PostExecutionNode { .. } => "post_execution_node",
        PrismNode::PreExecutionNode { .. } => "pre_execution_node",
        PrismNode::ProgramNode { .. } => "program_node",
        PrismNode::RangeNode { .. } => "range_node",
        PrismNode::RationalNode { .. } => "rational_node",
        PrismNode::RedoNode { .. } => "redo_node",
        PrismNode::RegularExpressionNode { .. } => "regular_expression_node",
        PrismNode::RequiredKeywordParameterNode { .. } => "required_keyword_parameter_node",
        PrismNode::RequiredParameterNode { .. } => "required_parameter_node",
        PrismNode::RescueModifierNode { .. } => "rescue_modifier_node",
        PrismNode::RescueNode { .. } => "rescue_node",
        PrismNode::RestParameterNode { .. } => "rest_parameter_node",
        PrismNode::RetryNode { .. } => "retry_node",
        PrismNode::ReturnNode { .. } => "return_node",
        PrismNode::SelfNode { .. } => "self_node",
        PrismNode::ShareableConstantNode { .. } => "shareable_constant_node",
        PrismNode::SingletonClassNode { .. } => "singleton_class_node",
        PrismNode::SourceEncodingNode { .. } => "source_encoding_node",
        PrismNode::SourceFileNode { .. } => "source_file_node",
        PrismNode::SourceLineNode { .. } => "source_line_node",
        PrismNode::SplatNode { .. } => "splat_node",
        PrismNode::StatementsNode { .. } => "statements_node",
        PrismNode::StringNode { .. } => "string_node",
        PrismNode::SuperNode { .. } => "super_node",
        PrismNode::SymbolNode { .. } => "symbol_node",
        PrismNode::TrueNode { .. } => "true_node",
        PrismNode::UndefNode { .. } => "undef_node",
        PrismNode::UnlessNode { .. } => "unless_node",
        PrismNode::UntilNode { .. } => "until_node",
        PrismNode::WhenNode { .. } => "when_node",
        PrismNode::WhileNode { .. } => "while_node",
        PrismNode::XStringNode { .. } => "x_string_node",
        PrismNode::YieldNode { .. } => "yield_node",
    }
}

use crate::ast::{Comment, Node, NodeType};
use crate::config::{Config, IndentStyle};
use crate::error::Result;
use std::fmt::Write;

/// Block style for Ruby blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockStyle {
    DoEnd,  // do ... end
    Braces, // { ... }
}

/// Code emitter that converts AST back to Ruby source code
pub struct Emitter {
    config: Config,
    source: String,
    buffer: String,
    all_comments: Vec<Comment>,
    emitted_comment_indices: Vec<usize>,
}

impl Emitter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            source: String::new(),
            buffer: String::new(),
            all_comments: Vec::new(),
            emitted_comment_indices: Vec::new(),
        }
    }

    /// Create emitter with source code for fallback extraction
    pub fn with_source(config: Config, source: String) -> Self {
        Self {
            config,
            source,
            buffer: String::new(),
            all_comments: Vec::new(),
            emitted_comment_indices: Vec::new(),
        }
    }

    /// Emit Ruby source code from an AST
    pub fn emit(&mut self, ast: &Node) -> Result<String> {
        self.buffer.clear();
        self.emitted_comment_indices.clear();

        self.collect_comments(ast);

        self.emit_node(ast, 0)?;

        // Emit any remaining comments that weren't emitted
        self.emit_remaining_comments()?;

        // Ensure file ends with a newline
        if !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }

        Ok(self.buffer.clone())
    }

    /// Emit all comments that haven't been emitted yet
    fn emit_remaining_comments(&mut self) -> Result<()> {
        let mut last_end_line: Option<usize> = None;
        for (idx, comment) in self.all_comments.iter().enumerate() {
            if self.emitted_comment_indices.contains(&idx) {
                continue;
            }
            // Preserve blank lines between comments
            if let Some(prev_line) = last_end_line {
                let gap = comment.location.start_line.saturating_sub(prev_line);
                for _ in 1..gap {
                    self.buffer.push('\n');
                }
            }
            writeln!(self.buffer, "{}", comment.text)?;
            self.emitted_comment_indices.push(idx);
            last_end_line = Some(comment.location.end_line);
        }
        Ok(())
    }

    /// Recursively collect all comments from the AST
    fn collect_comments(&mut self, node: &Node) {
        self.all_comments.extend(node.comments.clone());
        for child in &node.children {
            self.collect_comments(child);
        }
    }

    /// Emit comments that appear before a given line
    fn emit_comments_before(&mut self, line: usize, indent_level: usize) -> Result<()> {
        let indent_str = match self.config.formatting.indent_style {
            IndentStyle::Spaces => " ".repeat(self.config.formatting.indent_width * indent_level),
            IndentStyle::Tabs => "\t".repeat(indent_level),
        };

        let mut comments_to_emit = Vec::new();
        for (idx, comment) in self.all_comments.iter().enumerate() {
            if self.emitted_comment_indices.contains(&idx) {
                continue;
            }

            if comment.location.end_line < line {
                comments_to_emit.push((idx, comment.text.clone(), comment.location.end_line));
            }
        }

        let comments_count = comments_to_emit.len();
        for (i, (idx, text, comment_end_line)) in comments_to_emit.into_iter().enumerate() {
            writeln!(self.buffer, "{}{}", indent_str, text)?;
            self.emitted_comment_indices.push(idx);

            // Only add blank line after the LAST comment if there was a gap in the original
            if i == comments_count - 1 && line > comment_end_line + 1 {
                self.buffer.push('\n');
            }
        }

        Ok(())
    }

    /// Emit comments that appear on the same line (trailing comments)
    fn emit_trailing_comments(&mut self, line: usize) -> Result<()> {
        let mut indices_to_emit = Vec::new();
        for (idx, comment) in self.all_comments.iter().enumerate() {
            if self.emitted_comment_indices.contains(&idx) {
                continue;
            }

            // Collect comments on the same line (trailing)
            if comment.location.start_line == line {
                indices_to_emit.push((idx, comment.text.clone()));
            }
        }

        // Now emit the collected comments
        for (idx, text) in indices_to_emit {
            write!(self.buffer, " {}", text)?;
            self.emitted_comment_indices.push(idx);
        }

        Ok(())
    }

    /// Emit a node with given indentation level
    fn emit_node(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        match &node.node_type {
            NodeType::ProgramNode => self.emit_program(node, indent_level)?,
            NodeType::StatementsNode => self.emit_statements(node, indent_level)?,
            NodeType::ClassNode => self.emit_class(node, indent_level)?,
            NodeType::ModuleNode => self.emit_module(node, indent_level)?,
            NodeType::DefNode => self.emit_method(node, indent_level)?,
            NodeType::IfNode => self.emit_if_unless(node, indent_level, false, "if")?,
            NodeType::UnlessNode => self.emit_if_unless(node, indent_level, false, "unless")?,
            NodeType::CallNode => self.emit_call(node, indent_level)?,
            NodeType::BeginNode => self.emit_begin(node, indent_level)?,
            NodeType::RescueNode => self.emit_rescue(node, indent_level)?,
            NodeType::EnsureNode => self.emit_ensure(node, indent_level)?,
            NodeType::LambdaNode => self.emit_lambda(node, indent_level)?,
            NodeType::CaseNode => self.emit_case(node, indent_level)?,
            NodeType::WhenNode => self.emit_when(node, indent_level)?,
            NodeType::WhileNode => self.emit_while_until(node, indent_level, "while")?,
            NodeType::UntilNode => self.emit_while_until(node, indent_level, "until")?,
            NodeType::ForNode => self.emit_for(node, indent_level)?,
            _ => self.emit_generic(node, indent_level)?,
        }
        Ok(())
    }

    /// Emit program node (root)
    fn emit_program(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        for (i, child) in node.children.iter().enumerate() {
            self.emit_node(child, indent_level)?;

            // Add newlines between top-level statements, normalizing to max 1 blank line
            if i < node.children.len() - 1 {
                let current_end_line = child.location.end_line;
                let next_start_line = node.children[i + 1].location.start_line;
                let line_diff = next_start_line.saturating_sub(current_end_line);

                // Add 1 newline if consecutive, 2 newlines (1 blank line) if there was a gap
                let newlines = if line_diff > 1 { 2 } else { 1 };
                for _ in 0..newlines {
                    self.buffer.push('\n');
                }
            }
        }
        Ok(())
    }

    /// Emit statements node (body of class/module/def)
    fn emit_statements(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        for (i, child) in node.children.iter().enumerate() {
            self.emit_node(child, indent_level)?;

            if i < node.children.len() - 1 {
                let current_end_line = child.location.end_line;
                let next_child = &node.children[i + 1];
                let next_start_line = next_child.location.start_line;

                // Find the first comment between current and next node (if any)
                let first_comment_line = self
                    .all_comments
                    .iter()
                    .filter(|c| {
                        c.location.start_line > current_end_line
                            && c.location.end_line < next_start_line
                    })
                    .map(|c| c.location.start_line)
                    .min();

                // Calculate line diff based on whether there's a comment
                let effective_next_line = first_comment_line.unwrap_or(next_start_line);
                let line_diff = effective_next_line.saturating_sub(current_end_line);

                let newlines = if line_diff > 1 { 2 } else { 1 };

                for _ in 0..newlines {
                    self.buffer.push('\n');
                }
            }
        }
        Ok(())
    }

    /// Emit class definition
    fn emit_class(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Emit any comments before this class
        self.emit_comments_before(node.location.start_line, indent_level)?;

        self.emit_indent(indent_level)?;
        write!(self.buffer, "class ")?;

        if let Some(name) = node.metadata.get("name") {
            write!(self.buffer, "{}", name)?;
        }

        if let Some(superclass) = node.metadata.get("superclass") {
            write!(self.buffer, " < {}", superclass)?;
        }

        // Emit trailing comments on the class definition line (e.g., # rubocop:disable)
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes (class name, superclass)
        // Use start_line check to properly handle CallNode superclasses like ActiveRecord::Migration[8.0]
        let class_start_line = node.location.start_line;
        for child in &node.children {
            // Skip nodes on the same line as class definition (name, superclass)
            if child.location.start_line == class_start_line {
                continue;
            }
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
        }

        // Add newline before end if there was body content
        if node.children.iter().any(|c| {
            c.location.start_line != class_start_line && !self.is_structural_node(&c.node_type)
        }) {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit module definition
    fn emit_module(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Emit any comments before this module
        self.emit_comments_before(node.location.start_line, indent_level)?;

        self.emit_indent(indent_level)?;
        write!(self.buffer, "module ")?;

        if let Some(name) = node.metadata.get("name") {
            write!(self.buffer, "{}", name)?;
        }

        // Emit trailing comments on the module definition line
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
        }

        // Add newline before end if there was body content
        if node
            .children
            .iter()
            .any(|c| !self.is_structural_node(&c.node_type))
        {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit method definition
    fn emit_method(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Emit any comments before this method
        self.emit_comments_before(node.location.start_line, indent_level)?;

        self.emit_indent(indent_level)?;
        write!(self.buffer, "def ")?;

        // Handle class methods (def self.method_name)
        if let Some(receiver) = node.metadata.get("receiver") {
            write!(self.buffer, "{}.", receiver)?;
        }

        if let Some(name) = node.metadata.get("name") {
            write!(self.buffer, "{}", name)?;
        }

        // TODO: Handle parameters properly
        // For now, extract from source if method has parameters
        if node
            .metadata
            .get("parameters_count")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            > 0
        {
            // Extract parameter part from source
            if !self.source.is_empty() && node.location.end_offset <= self.source.len() {
                if let Some(source_text) = self
                    .source
                    .get(node.location.start_offset..node.location.end_offset)
                {
                    // Find parameters in source (between def name and \n or ;)
                    if let Some(def_line) = source_text.lines().next() {
                        if let Some(params_start) = def_line.find('(') {
                            if let Some(params_end) = def_line.find(')') {
                                let params = &def_line[params_start..=params_end];
                                write!(self.buffer, "{}", params)?;
                            }
                        }
                    }
                }
            }
        }

        // Emit trailing comment on same line as def
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes like parameter nodes
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
        }

        // Add newline before end if there was body content
        if node
            .children
            .iter()
            .any(|c| !self.is_structural_node(&c.node_type))
        {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit begin node
    /// BeginNode can be either:
    /// 1. Explicit begin...end block (source starts with "begin")
    /// 2. Implicit begin wrapping method body with rescue/ensure
    fn emit_begin(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Check if this is an explicit begin block by looking at source
        let is_explicit_begin = if !self.source.is_empty() {
            self.source
                .get(node.location.start_offset..)
                .map(|s| s.trim_start().starts_with("begin"))
                .unwrap_or(false)
        } else {
            false
        };

        if is_explicit_begin {
            self.emit_comments_before(node.location.start_line, indent_level)?;
            self.emit_indent(indent_level)?;
            writeln!(self.buffer, "begin")?;

            for child in &node.children {
                self.emit_node(child, indent_level + 1)?;
                self.buffer.push('\n');
            }

            self.emit_indent(indent_level)?;
            write!(self.buffer, "end")?;
        } else {
            // Implicit begin - emit children directly
            for (i, child) in node.children.iter().enumerate() {
                if i > 0 {
                    self.buffer.push('\n');
                }
                self.emit_node(child, indent_level)?;
            }
        }
        Ok(())
    }

    /// Emit rescue node
    fn emit_rescue(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Rescue node structure:
        // - First children are exception class references (ConstantReadNode)
        // - Then exception variable (LocalVariableTargetNode)
        // - Last child is StatementsNode with the rescue body

        // Dedent by 1 level since rescue is at the same level as method body
        let rescue_indent = indent_level.saturating_sub(1);
        self.emit_indent(rescue_indent)?;
        write!(self.buffer, "rescue")?;

        // Extract exception classes and variable from source
        // Handle multi-line rescue clauses (e.g., multiple exception classes spanning lines)
        if !self.source.is_empty() && node.location.end_offset <= self.source.len() {
            if let Some(source_text) = self
                .source
                .get(node.location.start_offset..node.location.end_offset)
            {
                // Find the rescue declaration part (first line only, unless trailing comma/backslash)
                let mut rescue_decl = String::new();
                let mut expect_continuation = false;

                for line in source_text.lines() {
                    let trimmed = line.trim();

                    if rescue_decl.is_empty() {
                        // First line - remove "rescue" prefix
                        let after_rescue = trimmed.trim_start_matches("rescue").trim();
                        if !after_rescue.is_empty() {
                            // Check if line ends with continuation marker
                            expect_continuation =
                                after_rescue.ends_with(',') || after_rescue.ends_with('\\');
                            rescue_decl.push_str(after_rescue.trim_end_matches('\\').trim());
                        }
                        if !expect_continuation {
                            break;
                        }
                    } else if expect_continuation {
                        // Continuation line after trailing comma or backslash
                        if !rescue_decl.ends_with(' ') && !rescue_decl.ends_with(',') {
                            rescue_decl.push(' ');
                        }
                        let content = trimmed.trim_end_matches('\\').trim();
                        rescue_decl.push_str(content);
                        expect_continuation = trimmed.ends_with(',') || trimmed.ends_with('\\');
                        if !expect_continuation {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if !rescue_decl.is_empty() {
                    write!(self.buffer, " {}", rescue_decl)?;
                }
            }
        }

        self.buffer.push('\n');

        // Emit rescue body and handle subsequent rescue nodes
        // Children structure:
        // - ConstantReadNode/ConstantPathNode (exception classes)
        // - LocalVariableTargetNode (optional, exception variable)
        // - StatementsNode (rescue body)
        // - RescueNode (optional, subsequent rescue clause)
        for child in &node.children {
            match &child.node_type {
                NodeType::StatementsNode => {
                    self.emit_node(child, indent_level)?;
                }
                NodeType::RescueNode => {
                    // Emit subsequent rescue clause
                    // Ensure newline before subsequent rescue
                    if !self.buffer.ends_with('\n') {
                        self.buffer.push('\n');
                    }
                    self.emit_rescue(child, indent_level)?;
                }
                _ => {
                    // Skip exception classes and variable (already handled above)
                }
            }
        }

        Ok(())
    }

    /// Emit ensure node
    fn emit_ensure(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // ensure keyword should be at same level as begin/rescue
        let ensure_indent = indent_level.saturating_sub(1);

        self.emit_comments_before(node.location.start_line, ensure_indent)?;
        self.emit_indent(ensure_indent)?;
        writeln!(self.buffer, "ensure")?;

        // Emit ensure body statements
        for child in &node.children {
            match &child.node_type {
                NodeType::StatementsNode => {
                    self.emit_statements(child, indent_level)?;
                }
                _ => {
                    self.emit_node(child, indent_level)?;
                }
            }
        }

        Ok(())
    }

    /// Emit lambda node
    fn emit_lambda(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;

        // Lambda syntax is complex (-> vs lambda, {} vs do-end)
        // Use source extraction to preserve original style
        self.emit_generic_without_comments(node, indent_level)
    }

    /// Emit case node
    fn emit_case(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;

        // Write "case" keyword
        write!(self.buffer, "case")?;

        // Find predicate (first child that isn't WhenNode or ElseNode)
        let mut when_start_idx = 0;
        if let Some(first_child) = node.children.first() {
            if !matches!(
                first_child.node_type,
                NodeType::WhenNode | NodeType::ElseNode
            ) {
                // This is the predicate - extract from source
                let start = first_child.location.start_offset;
                let end = first_child.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, " {}", text)?;
                }
                when_start_idx = 1;
            }
        }

        self.buffer.push('\n');

        // Emit when clauses and else
        for child in node.children.iter().skip(when_start_idx) {
            match &child.node_type {
                NodeType::WhenNode => {
                    self.emit_when(child, indent_level)?;
                    self.buffer.push('\n');
                }
                NodeType::ElseNode => {
                    self.emit_indent(indent_level)?;
                    writeln!(self.buffer, "else")?;
                    // Emit else body
                    for else_child in &child.children {
                        if matches!(else_child.node_type, NodeType::StatementsNode) {
                            self.emit_statements(else_child, indent_level + 1)?;
                        } else {
                            self.emit_node(else_child, indent_level + 1)?;
                        }
                    }
                    self.buffer.push('\n');
                }
                _ => {}
            }
        }

        // Emit "end" keyword
        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit when node
    fn emit_when(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;

        write!(self.buffer, "when ")?;

        // Collect conditions (all children except StatementsNode)
        let conditions: Vec<_> = node
            .children
            .iter()
            .filter(|c| !matches!(c.node_type, NodeType::StatementsNode))
            .collect();

        // Emit conditions with comma separator
        for (i, cond) in conditions.iter().enumerate() {
            let start = cond.location.start_offset;
            let end = cond.location.end_offset;
            if let Some(text) = self.source.get(start..end) {
                write!(self.buffer, "{}", text)?;
            }
            if i < conditions.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        self.buffer.push('\n');

        // Emit statements body
        if let Some(statements) = node
            .children
            .iter()
            .find(|c| matches!(c.node_type, NodeType::StatementsNode))
        {
            self.emit_statements(statements, indent_level + 1)?;
        }

        Ok(())
    }

    /// Emit if/unless/elsif/else node
    /// is_elsif: true if this is an elsif clause (don't emit 'end')
    /// keyword: "if" or "unless"
    fn emit_if_unless(
        &mut self,
        node: &Node,
        indent_level: usize,
        is_elsif: bool,
        keyword: &str,
    ) -> Result<()> {
        // Check if this is a postfix if (modifier form)
        // In postfix if, the statements come before the if keyword in source
        let is_postfix = if let (Some(predicate), Some(statements)) =
            (node.children.first(), node.children.get(1))
        {
            statements.location.start_offset < predicate.location.start_offset
        } else {
            false
        };

        // Postfix if/unless: "statement if/unless condition"
        if is_postfix && !is_elsif {
            self.emit_comments_before(node.location.start_line, indent_level)?;
            self.emit_indent(indent_level)?;

            // Emit statement
            if let Some(statements) = node.children.get(1) {
                if matches!(statements.node_type, NodeType::StatementsNode) {
                    // Extract the statement text (without extra indentation)
                    if !self.source.is_empty() {
                        let start = statements.location.start_offset;
                        let end = statements.location.end_offset;
                        if let Some(text) = self.source.get(start..end) {
                            write!(self.buffer, "{}", text.trim())?;
                        }
                    }
                }
            }

            write!(self.buffer, " {} ", keyword)?;

            // Emit condition
            if let Some(predicate) = node.children.first() {
                if !self.source.is_empty() {
                    let start = predicate.location.start_offset;
                    let end = predicate.location.end_offset;
                    if let Some(text) = self.source.get(start..end) {
                        write!(self.buffer, "{}", text)?;
                    }
                }
            }

            return Ok(());
        }

        // Check for ternary operator
        let is_ternary = node
            .metadata
            .get("is_ternary")
            .map(|v| v == "true")
            .unwrap_or(false);

        if is_ternary && !is_elsif {
            self.emit_comments_before(node.location.start_line, indent_level)?;
            self.emit_indent(indent_level)?;

            // Emit condition
            if let Some(predicate) = node.children.first() {
                if !self.source.is_empty() {
                    let start = predicate.location.start_offset;
                    let end = predicate.location.end_offset;
                    if let Some(text) = self.source.get(start..end) {
                        write!(self.buffer, "{}", text)?;
                    }
                }
            }

            write!(self.buffer, " ? ")?;

            // Emit then expression
            if let Some(statements) = node.children.get(1) {
                if !self.source.is_empty() {
                    let start = statements.location.start_offset;
                    let end = statements.location.end_offset;
                    if let Some(text) = self.source.get(start..end) {
                        write!(self.buffer, "{}", text.trim())?;
                    }
                }
            }

            write!(self.buffer, " : ")?;

            // Emit else expression
            if let Some(else_node) = node.children.get(2) {
                if let Some(else_statements) = else_node.children.first() {
                    if !self.source.is_empty() {
                        let start = else_statements.location.start_offset;
                        let end = else_statements.location.end_offset;
                        if let Some(text) = self.source.get(start..end) {
                            write!(self.buffer, "{}", text.trim())?;
                        }
                    }
                }
            }

            return Ok(());
        }

        // Normal if/unless/elsif
        if !is_elsif {
            self.emit_comments_before(node.location.start_line, indent_level)?;
        }

        // Emit 'if'/'unless' or 'elsif' keyword
        self.emit_indent(indent_level)?;
        if is_elsif {
            write!(self.buffer, "elsif ")?;
        } else {
            write!(self.buffer, "{} ", keyword)?;
        }

        // Emit predicate (condition) - first child
        if let Some(predicate) = node.children.first() {
            // Extract predicate from source
            if !self.source.is_empty() {
                let start = predicate.location.start_offset;
                let end = predicate.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, "{}", text)?;
                }
            }
        }

        // Emit trailing comment on same line as if/unless/elsif
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        // Emit then clause (second child is StatementsNode)
        if let Some(statements) = node.children.get(1) {
            if matches!(statements.node_type, NodeType::StatementsNode) {
                self.emit_statements(statements, indent_level + 1)?;
                self.buffer.push('\n');
            }
        }

        // Check for elsif/else (third child)
        if let Some(consequent) = node.children.get(2) {
            match &consequent.node_type {
                NodeType::IfNode => {
                    // This is an elsif clause (only valid for if, not unless)
                    self.emit_if_unless(consequent, indent_level, true, "if")?;
                }
                NodeType::ElseNode => {
                    // This is an else clause
                    self.emit_indent(indent_level)?;
                    writeln!(self.buffer, "else")?;

                    // Emit else body (first child of ElseNode)
                    if let Some(else_statements) = consequent.children.first() {
                        if matches!(else_statements.node_type, NodeType::StatementsNode) {
                            self.emit_statements(else_statements, indent_level + 1)?;
                            self.buffer.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }

        // Only emit 'end' for the outermost if (not for elsif)
        if !is_elsif {
            self.emit_indent(indent_level)?;
            write!(self.buffer, "end")?;
        }

        Ok(())
    }

    /// Emit method call, handling blocks specially for proper indentation
    fn emit_call(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Emit any comments before this call
        self.emit_comments_before(node.location.start_line, indent_level)?;

        // Check if this call has a block (last child is BlockNode)
        let has_block = node
            .children
            .last()
            .map(|c| matches!(c.node_type, NodeType::BlockNode))
            .unwrap_or(false);

        if !has_block {
            // No block - use generic emission (extracts from source)
            return self.emit_generic_without_comments(node, indent_level);
        }

        // Has block - need to handle specially
        let block_node = node.children.last().unwrap();

        // Determine block style from source (do...end vs { })
        let block_style = self.detect_block_style(block_node);

        // Emit the call part (receiver.method(args)) from source
        self.emit_call_without_block(node, block_node, indent_level)?;

        match block_style {
            BlockStyle::DoEnd => self.emit_do_end_block(block_node, indent_level)?,
            BlockStyle::Braces => self.emit_brace_block(block_node, indent_level)?,
        }

        Ok(())
    }

    /// Detect whether block uses do...end or { } style
    fn detect_block_style(&self, block_node: &Node) -> BlockStyle {
        if self.source.is_empty() {
            return BlockStyle::DoEnd; // Default fallback
        }

        let start = block_node.location.start_offset;
        if let Some(first_char) = self.source.get(start..start + 1) {
            if first_char == "{" {
                return BlockStyle::Braces;
            }
        }

        BlockStyle::DoEnd // Default (includes 'do' keyword)
    }

    /// Emit the method call part without the block
    fn emit_call_without_block(
        &mut self,
        call_node: &Node,
        block_node: &Node,
        indent_level: usize,
    ) -> Result<()> {
        self.emit_indent(indent_level)?;

        if !self.source.is_empty() {
            let start = call_node.location.start_offset;
            let end = block_node.location.start_offset;

            if let Some(text) = self.source.get(start..end) {
                // Trim trailing whitespace but preserve the content
                write!(self.buffer, "{}", text.trim_end())?;
            }
        }

        Ok(())
    }

    /// Emit a do...end style block with proper indentation
    fn emit_do_end_block(&mut self, block_node: &Node, indent_level: usize) -> Result<()> {
        // Add space before 'do' and emit 'do'
        write!(self.buffer, " do")?;

        // Emit block parameters if present (|x, y|)
        self.emit_block_parameters(block_node)?;

        // Emit trailing comment on same line as do |...|
        self.emit_trailing_comments(block_node.location.start_line)?;
        self.buffer.push('\n');

        // Find and emit the body (StatementsNode among children)
        for child in &block_node.children {
            if matches!(child.node_type, NodeType::StatementsNode) {
                self.emit_statements(child, indent_level + 1)?;
                self.buffer.push('\n');
                break;
            }
        }

        // Emit 'end'
        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit a { } style block
    fn emit_brace_block(&mut self, block_node: &Node, indent_level: usize) -> Result<()> {
        // Determine if block should be inline or multiline
        let is_multiline = block_node.location.start_line != block_node.location.end_line;

        if is_multiline {
            // Multiline brace block
            write!(self.buffer, " {{")?;
            self.emit_block_parameters(block_node)?;
            self.buffer.push('\n');

            // Emit body
            for child in &block_node.children {
                if matches!(child.node_type, NodeType::StatementsNode) {
                    self.emit_statements(child, indent_level + 1)?;
                    self.buffer.push('\n');
                    break;
                }
            }

            self.emit_indent(indent_level)?;
            write!(self.buffer, "}}")?;
        } else {
            // Inline brace block - extract from source to preserve spacing
            write!(self.buffer, " ")?;
            if let Some(text) = self
                .source
                .get(block_node.location.start_offset..block_node.location.end_offset)
            {
                write!(self.buffer, "{}", text)?;
            }
        }

        Ok(())
    }

    /// Emit block parameters (|x, y|)
    fn emit_block_parameters(&mut self, block_node: &Node) -> Result<()> {
        if self.source.is_empty() {
            return Ok(());
        }

        let start = block_node.location.start_offset;
        let end = block_node.location.end_offset;

        if let Some(block_source) = self.source.get(start..end) {
            // Only look at the first line of the block for parameters
            let first_line = block_source.lines().next().unwrap_or("");

            // Find |...| pattern in the first line only
            if let Some(pipe_start) = first_line.find('|') {
                // Find matching pipe after first one
                if let Some(pipe_end) = first_line[pipe_start + 1..].find('|') {
                    let params = &first_line[pipe_start..=pipe_start + 1 + pipe_end];
                    write!(self.buffer, " {}", params)?;
                }
            }
        }

        Ok(())
    }

    /// Emit generic node without re-emitting comments (for use when comments already handled)
    fn emit_generic_without_comments(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        if !self.source.is_empty() {
            let start = node.location.start_offset;
            let end = node.location.end_offset;

            let text_owned = self.source.get(start..end).map(|s| s.to_string());

            if let Some(text) = text_owned {
                self.emit_indent(indent_level)?;
                write!(self.buffer, "{}", text)?;
                self.emit_trailing_comments(node.location.end_line)?;
            }
        }
        Ok(())
    }

    /// Emit generic node by extracting from source
    fn emit_generic(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;

        if !self.source.is_empty() {
            let start = node.location.start_offset;
            let end = node.location.end_offset;

            let text_owned = self.source.get(start..end).map(|s| s.to_string());

            if let Some(text) = text_owned {
                self.emit_indent(indent_level)?;
                write!(self.buffer, "{}", text)?;

                // Mark comments within this node's range as emitted
                // (they are included in the source extraction)
                for (idx, comment) in self.all_comments.iter().enumerate() {
                    if !self.emitted_comment_indices.contains(&idx)
                        && comment.location.start_line >= node.location.start_line
                        && comment.location.end_line <= node.location.end_line
                    {
                        self.emitted_comment_indices.push(idx);
                    }
                }

                self.emit_trailing_comments(node.location.end_line)?;
            }
        }
        Ok(())
    }

    /// Emit indentation
    fn emit_indent(&mut self, level: usize) -> Result<()> {
        let indent_str = match self.config.formatting.indent_style {
            IndentStyle::Spaces => " ".repeat(self.config.formatting.indent_width * level),
            IndentStyle::Tabs => "\t".repeat(level),
        };

        write!(self.buffer, "{}", indent_str)?;
        Ok(())
    }

    /// Emit while/until loop
    fn emit_while_until(&mut self, node: &Node, indent_level: usize, keyword: &str) -> Result<()> {
        // Check if this is a postfix while/until (modifier form)
        // In postfix form: "statement while/until condition"
        // Check if body starts before predicate in source
        let is_postfix = if node.children.len() >= 2 {
            let predicate = &node.children[0];
            let body = &node.children[1];
            body.location.start_offset < predicate.location.start_offset
        } else {
            false
        };

        if is_postfix {
            // Postfix form: extract from source as-is
            return self.emit_generic(node, indent_level);
        }

        // Normal while/until with do...end
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;
        write!(self.buffer, "{} ", keyword)?;

        // Emit predicate (condition) - first child
        if let Some(predicate) = node.children.first() {
            if !self.source.is_empty() {
                let start = predicate.location.start_offset;
                let end = predicate.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, "{}", text)?;
                }
            }
        }

        // Emit trailing comment on same line as while/until
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        // Emit body - second child (StatementsNode)
        if let Some(body) = node.children.get(1) {
            if matches!(body.node_type, NodeType::StatementsNode) {
                self.emit_statements(body, indent_level + 1)?;
                self.buffer.push('\n');
            }
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit for loop
    fn emit_for(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;
        write!(self.buffer, "for ")?;

        // node.children: [index, collection, statements]
        // index: LocalVariableTargetNode or MultiTargetNode
        // collection: expression
        // statements: StatementsNode

        // Emit index variable - first child
        if let Some(index) = node.children.first() {
            if !self.source.is_empty() {
                let start = index.location.start_offset;
                let end = index.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, "{}", text)?;
                }
            }
        }

        write!(self.buffer, " in ")?;

        // Emit collection - second child
        if let Some(collection) = node.children.get(1) {
            if !self.source.is_empty() {
                let start = collection.location.start_offset;
                let end = collection.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, "{}", text)?;
                }
            }
        }

        self.buffer.push('\n');

        // Emit body - third child (StatementsNode)
        if let Some(body) = node.children.get(2) {
            if matches!(body.node_type, NodeType::StatementsNode) {
                self.emit_statements(body, indent_level + 1)?;
                self.buffer.push('\n');
            }
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Check if node is structural (part of definition syntax, not body)
    /// These nodes are part of class/module/method definitions and should not be emitted as body
    fn is_structural_node(&self, node_type: &NodeType) -> bool {
        matches!(
            node_type,
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
        )
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

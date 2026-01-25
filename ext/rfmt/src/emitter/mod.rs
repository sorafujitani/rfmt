use crate::ast::{Comment, Node, NodeType};
use crate::config::{Config, IndentStyle};
use crate::error::Result;
use std::collections::{BTreeMap, HashSet};
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
    emitted_comment_indices: HashSet<usize>,
    /// Cached indent strings by level (index = level, value = indent string)
    indent_cache: Vec<String>,
    /// Index of comment indices by start line for O(log n) lookup
    /// Key: start_line, Value: Vec of comment indices that start on that line
    comments_by_line: BTreeMap<usize, Vec<usize>>,
}

impl Emitter {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            source: String::new(),
            buffer: String::new(),
            all_comments: Vec::new(),
            emitted_comment_indices: HashSet::new(),
            indent_cache: Vec::new(),
            comments_by_line: BTreeMap::new(),
        }
    }

    /// Create emitter with source code for fallback extraction
    pub fn with_source(config: Config, source: String) -> Self {
        Self {
            config,
            source,
            buffer: String::new(),
            all_comments: Vec::new(),
            emitted_comment_indices: HashSet::new(),
            indent_cache: Vec::new(),
            comments_by_line: BTreeMap::new(),
        }
    }

    /// Emit Ruby source code from an AST
    pub fn emit(&mut self, ast: &Node) -> Result<String> {
        self.buffer.clear();
        self.emitted_comment_indices.clear();
        self.comments_by_line.clear();

        self.collect_comments(ast);
        self.build_comment_index();

        self.emit_node(ast, 0)?;

        let last_code_line = Self::find_last_code_line(ast);
        self.emit_remaining_comments(last_code_line)?;

        if !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }

        Ok(std::mem::take(&mut self.buffer))
    }

    /// Find the last line of code in the AST (excluding comments)
    fn find_last_code_line(ast: &Node) -> usize {
        let mut max_line = ast.location.end_line;
        let mut stack = vec![ast];

        while let Some(node) = stack.pop() {
            max_line = max_line.max(node.location.end_line);
            stack.extend(node.children.iter());
        }

        max_line
    }

    /// Emit all comments that haven't been emitted yet
    fn emit_remaining_comments(&mut self, last_code_line: usize) -> Result<()> {
        let mut last_end_line: Option<usize> = Some(last_code_line);
        let mut is_first_comment = true;

        for (idx, comment) in self.all_comments.iter().enumerate() {
            if self.emitted_comment_indices.contains(&idx) {
                continue;
            }

            // For the first remaining comment:
            // - If buffer is empty, don't add any leading newline
            // - If buffer has content, ensure we start on a new line
            if is_first_comment && self.buffer.is_empty() {
                // Don't add leading newline for first comment when buffer is empty
            } else if !self.buffer.ends_with('\n') {
                self.buffer.push('\n');
            }

            // Preserve blank lines between code/comments
            // But only if this is not the first comment in an empty buffer
            if !(is_first_comment && self.buffer.is_empty()) {
                if let Some(prev_line) = last_end_line {
                    let gap = comment.location.start_line.saturating_sub(prev_line);
                    for _ in 1..gap {
                        self.buffer.push('\n');
                    }
                }
            }

            writeln!(self.buffer, "{}", comment.text)?;
            self.emitted_comment_indices.insert(idx);
            last_end_line = Some(comment.location.end_line);
            is_first_comment = false;
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

    /// Build the comment index by start line for O(log n) range lookups
    fn build_comment_index(&mut self) {
        for (idx, comment) in self.all_comments.iter().enumerate() {
            self.comments_by_line
                .entry(comment.location.start_line)
                .or_default()
                .push(idx);
        }
    }

    /// Get comment indices in the given line range [start_line, end_line)
    /// Uses BTreeMap range for O(log n) lookup instead of O(n) iteration
    fn get_comment_indices_in_range(&self, start_line: usize, end_line: usize) -> Vec<usize> {
        // Guard against invalid range (e.g., endless methods where start_line >= end_line)
        if start_line >= end_line {
            return Vec::new();
        }
        self.comments_by_line
            .range(start_line..end_line)
            .flat_map(|(_, indices)| indices.iter().copied())
            .filter(|&idx| !self.emitted_comment_indices.contains(&idx))
            .collect()
    }

    /// Get comment indices before a given line (exclusive)
    /// Uses BTreeMap range for O(log n) lookup
    fn get_comment_indices_before(&self, line: usize) -> Vec<usize> {
        self.comments_by_line
            .range(..line)
            .flat_map(|(_, indices)| indices.iter().copied())
            .filter(|&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < line
            })
            .collect()
    }

    /// Get comment indices on a specific line (for trailing comments)
    /// Uses BTreeMap get for O(log n) lookup
    fn get_comment_indices_on_line(&self, line: usize) -> Vec<usize> {
        self.comments_by_line
            .get(&line)
            .map(|indices| {
                indices
                    .iter()
                    .copied()
                    .filter(|&idx| !self.emitted_comment_indices.contains(&idx))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Emit comments that appear before a given line
    /// Uses BTreeMap index for O(log n) lookup instead of O(n) iteration
    fn emit_comments_before(&mut self, line: usize, indent_level: usize) -> Result<()> {
        self.ensure_indent_cache(indent_level);

        let indices = self.get_comment_indices_before(line);

        let mut comments_to_emit: Vec<_> = indices
            .into_iter()
            .map(|idx| {
                let comment = &self.all_comments[idx];
                (idx, comment.location.start_line, comment.location.end_line)
            })
            .collect();

        comments_to_emit.sort_by_key(|(_, start, _)| *start);

        let comments_count = comments_to_emit.len();
        let mut last_comment_end_line: Option<usize> = None;

        for (i, (idx, comment_start_line, comment_end_line)) in
            comments_to_emit.into_iter().enumerate()
        {
            if let Some(prev_end) = last_comment_end_line {
                let gap = comment_start_line.saturating_sub(prev_end);
                for _ in 1..gap {
                    self.buffer.push('\n');
                }
            }

            writeln!(
                self.buffer,
                "{}{}",
                &self.indent_cache[indent_level], &self.all_comments[idx].text
            )?;
            self.emitted_comment_indices.insert(idx);
            last_comment_end_line = Some(comment_end_line);

            if i == comments_count - 1 && line > comment_end_line + 1 {
                self.buffer.push('\n');
            }
        }

        Ok(())
    }

    /// Check if there are any unemitted comments in the given line range
    /// Uses BTreeMap index for O(log n) lookup instead of O(n) iteration
    fn has_comments_in_range(&self, start_line: usize, end_line: usize) -> bool {
        // Guard against invalid range (e.g., endless methods where start_line >= end_line)
        if start_line >= end_line {
            return false;
        }
        self.comments_by_line
            .range(start_line..end_line)
            .flat_map(|(_, indices)| indices.iter())
            .any(|&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < end_line
            })
    }

    /// Emit comments that appear immediately before the end statement while preserving their position
    /// This is crucial for maintaining semantic relationships between comments and the code they precede
    fn emit_comments_before_end(
        &mut self,
        construct_start_line: usize,
        construct_end_line: usize,
        indent_level: usize,
    ) -> Result<()> {
        self.ensure_indent_cache(indent_level);

        // Implement proper comment positioning logic
        // Only emit standalone comments that appear on their own lines
        // This prevents comments from being incorrectly attached to code statements

        // Find comments that are between the construct and the end line
        // Only emit comments that haven't been emitted yet AND are on their own lines
        let indices =
            self.get_comment_indices_in_range(construct_start_line + 1, construct_end_line);

        let mut comments_to_emit: Vec<_> = indices
            .into_iter()
            .filter(|&idx| {
                let comment = &self.all_comments[idx];
                // Only emit if: not already emitted, before end line, and is standalone
                !self.emitted_comment_indices.contains(&idx)
                    && comment.location.end_line < construct_end_line
                    && self.is_standalone_comment(comment)
            })
            .map(|idx| {
                let comment = &self.all_comments[idx];
                (idx, comment.location.start_line, comment.location.end_line)
            })
            .collect();

        if comments_to_emit.is_empty() {
            return Ok(());
        }

        comments_to_emit.sort_by_key(|(_, start, _)| *start);

        // Ensure newline before first comment if buffer doesn't end with one
        if !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }

        let mut last_emitted_line: Option<usize> = None;

        // Emit comments while preserving their exact line positioning
        for (idx, comment_start_line, comment_end_line) in comments_to_emit {
            // Preserve blank lines between comments
            if let Some(prev_line) = last_emitted_line {
                let gap = comment_start_line.saturating_sub(prev_line);
                for _ in 1..gap {
                    self.buffer.push('\n');
                }
            }

            writeln!(
                self.buffer,
                "{}{}",
                &self.indent_cache[indent_level], &self.all_comments[idx].text
            )?;
            self.emitted_comment_indices.insert(idx);
            last_emitted_line = Some(comment_end_line);
        }

        Ok(())
    }

    /// Check if a comment should be treated as standalone
    /// A standalone comment is one that should appear on its own line,
    /// not attached to the end of a code statement
    fn is_standalone_comment(&self, comment: &Comment) -> bool {
        let comment_line = comment.location.start_line;
        let _comment_start_offset = comment.location.start_offset;

        // Get the source lines to analyze the comment's position
        let lines: Vec<&str> = self.source.lines().collect();

        // Check if we have a valid line number (1-indexed to 0-indexed)
        if comment_line == 0 || comment_line > lines.len() {
            return false;
        }

        let line = lines[comment_line - 1]; // Convert to 0-indexed

        // Find where the comment starts within the line
        let comment_text = &comment.text;

        // Look for the comment marker (#) in the line
        if let Some(hash_pos) = line.find('#') {
            // Check if there's only whitespace before the comment
            let before_comment = &line[..hash_pos];
            let is_only_whitespace = before_comment.trim().is_empty();

            // Also verify this is actually our comment by checking the text matches
            let line_comment_text = &line[hash_pos..];
            let is_same_comment = line_comment_text.trim_end() == comment_text.trim_end();

            return is_only_whitespace && is_same_comment;
        }

        // If we can't find the comment marker, assume it's standalone
        // This is a fallback for edge cases
        false
    }

    /// Check if the node spans only a single line
    fn is_single_line(&self, node: &Node) -> bool {
        node.location.start_line == node.location.end_line
    }

    /// Extract and write source text for a node
    fn write_source_text(&mut self, node: &Node) -> Result<()> {
        let start = node.location.start_offset;
        let end = node.location.end_offset;
        if let Some(text) = self.source.get(start..end) {
            write!(self.buffer, "{}", text)?;
        }
        Ok(())
    }

    /// Extract and write trimmed source text for a node
    fn write_source_text_trimmed(&mut self, node: &Node) -> Result<()> {
        let start = node.location.start_offset;
        let end = node.location.end_offset;
        if let Some(text) = self.source.get(start..end) {
            write!(self.buffer, "{}", text.trim())?;
        }
        Ok(())
    }

    /// Emit comments that are within a given line range, preserving blank lines from prev_line
    /// Uses BTreeMap index for O(log n) lookup instead of O(n) iteration
    fn emit_comments_in_range_with_prev_line(
        &mut self,
        start_line: usize,
        end_line: usize,
        indent_level: usize,
        prev_line: usize,
    ) -> Result<()> {
        self.ensure_indent_cache(indent_level);

        let indices = self.get_comment_indices_in_range(start_line, end_line);

        let mut comments_to_emit: Vec<_> = indices
            .into_iter()
            .filter(|&idx| self.all_comments[idx].location.end_line < end_line)
            .map(|idx| {
                let comment = &self.all_comments[idx];
                (idx, comment.location.start_line, comment.location.end_line)
            })
            .collect();

        comments_to_emit.sort_by_key(|(_, start, _)| *start);

        let mut last_end_line: usize = prev_line;

        for (idx, comment_start_line, comment_end_line) in comments_to_emit {
            // Preserve blank lines between previous content and this comment
            let gap = comment_start_line.saturating_sub(last_end_line);
            for _ in 1..gap {
                self.buffer.push('\n');
            }

            writeln!(
                self.buffer,
                "{}{}",
                &self.indent_cache[indent_level], &self.all_comments[idx].text
            )?;
            self.emitted_comment_indices.insert(idx);
            last_end_line = comment_end_line;
        }

        Ok(())
    }

    /// Emit comments that appear on the same line (trailing comments)
    /// Uses BTreeMap index for O(log n) lookup instead of O(n) iteration
    fn emit_trailing_comments(&mut self, line: usize) -> Result<()> {
        // Use indexed lookup for O(log n) access
        let indices = self.get_comment_indices_on_line(line);

        // Collect indices only (no text clone needed)
        let indices_to_emit: Vec<usize> = indices;

        // Now emit the collected comments by accessing text at write time
        for idx in indices_to_emit {
            write!(self.buffer, " {}", &self.all_comments[idx].text)?;
            self.emitted_comment_indices.insert(idx);
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
            NodeType::SingletonClassNode => self.emit_singleton_class(node, indent_level)?,
            NodeType::CaseMatchNode => self.emit_case_match(node, indent_level)?,
            NodeType::InNode => self.emit_in(node, indent_level)?,
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
                // Uses BTreeMap range for O(log n) lookup instead of O(n) iteration
                let first_comment_line = self
                    .comments_by_line
                    .range((current_end_line + 1)..next_start_line)
                    .next()
                    .map(|(line, _)| *line);

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
        let class_end_line = node.location.end_line;
        let mut has_body_content = false;

        for child in &node.children {
            // Skip nodes on the same line as class definition (name, superclass)
            if child.location.start_line == class_start_line {
                continue;
            }
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            has_body_content = true;
            self.emit_node(child, indent_level + 1)?;
        }

        // Emit comments that appear before the end statement while preserving their position
        self.emit_comments_before_end(class_start_line, class_end_line, indent_level + 1)?;

        // Add newline before end if there was body content or internal comments
        if (has_body_content || self.has_comments_in_range(class_start_line + 1, class_end_line))
            && !self.buffer.ends_with('\n')
        {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;
        // Emit trailing comments on end line (e.g., `end # rubocop:disable`)
        self.emit_trailing_comments(node.location.end_line)?;

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

        let module_start_line = node.location.start_line;
        let module_end_line = node.location.end_line;
        let mut has_body_content = false;

        // Emit body (children), but skip structural nodes
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            has_body_content = true;
            self.emit_node(child, indent_level + 1)?;
        }

        // Emit comments that appear before the end statement while preserving their position
        self.emit_comments_before_end(module_start_line, module_end_line, indent_level + 1)?;

        // Add newline before end if there was body content or internal comments
        if (has_body_content || self.has_comments_in_range(module_start_line + 1, module_end_line))
            && !self.buffer.ends_with('\n')
        {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;
        // Emit trailing comments on end line (e.g., `end # rubocop:disable`)
        self.emit_trailing_comments(node.location.end_line)?;

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

        // Emit parameters using metadata from prism_bridge
        if let Some(params_text) = node.metadata.get("parameters_text") {
            let has_parens = node
                .metadata
                .get("has_parens")
                .map(|v| v == "true")
                .unwrap_or(false);
            if has_parens {
                write!(self.buffer, "({})", params_text)?;
            } else {
                write!(self.buffer, " {}", params_text)?;
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

        // Emit comments that appear before the end statement while preserving their position
        self.emit_comments_before_end(
            node.location.start_line,
            node.location.end_line,
            indent_level + 1,
        )?;

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
        // Emit trailing comments on end line (e.g., `end # rubocop:disable`)
        self.emit_trailing_comments(node.location.end_line)?;

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
            // Emit trailing comments on end line
            self.emit_trailing_comments(node.location.end_line)?;
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
                        // Add space after comma or if no trailing space
                        if !rescue_decl.ends_with(' ') {
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
        // Emit trailing comments on end line
        self.emit_trailing_comments(node.location.end_line)?;

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
            self.write_source_text(cond)?;
            if i < conditions.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        let statements = node
            .children
            .iter()
            .find(|c| matches!(c.node_type, NodeType::StatementsNode));

        if self.is_single_line(node) {
            // Inline style: when X then Y
            if let Some(statements) = statements {
                write!(self.buffer, " then ")?;
                self.write_source_text(statements)?;
            }
        } else {
            // Multi-line style: when X\n  Y
            self.buffer.push('\n');

            if let Some(statements) = statements {
                self.emit_statements(statements, indent_level + 1)?;
            }
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
                    self.write_source_text_trimmed(statements)?;
                }
            }

            write!(self.buffer, " {} ", keyword)?;

            // Emit condition
            if let Some(predicate) = node.children.first() {
                self.write_source_text(predicate)?;
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
                self.write_source_text(predicate)?;
            }

            write!(self.buffer, " ? ")?;

            // Emit then expression
            if let Some(statements) = node.children.get(1) {
                self.write_source_text_trimmed(statements)?;
            }

            write!(self.buffer, " : ")?;

            // Emit else expression
            if let Some(else_node) = node.children.get(2) {
                if let Some(else_statements) = else_node.children.first() {
                    self.write_source_text_trimmed(else_statements)?;
                }
            }

            return Ok(());
        }

        // Check for inline then style: "if true then 1 end"
        // Single line, not postfix, not ternary, no else clause
        let is_inline_then =
            !is_elsif && self.is_single_line(node) && node.children.get(2).is_none();

        if is_inline_then {
            self.emit_comments_before(node.location.start_line, indent_level)?;
            self.emit_indent(indent_level)?;
            write!(self.buffer, "{} ", keyword)?;

            // Emit condition
            if let Some(predicate) = node.children.first() {
                self.write_source_text(predicate)?;
            }

            write!(self.buffer, " then ")?;

            // Emit statement
            if let Some(statements) = node.children.get(1) {
                self.write_source_text_trimmed(statements)?;
            }

            write!(self.buffer, " end")?;
            self.emit_trailing_comments(node.location.end_line)?;
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
            self.write_source_text(predicate)?;
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
            // Emit trailing comments on end line
            self.emit_trailing_comments(node.location.end_line)?;
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
        let block_start_line = block_node.location.start_line;
        let block_end_line = block_node.location.end_line;
        let mut last_stmt_end_line = block_start_line;

        for child in &block_node.children {
            match &child.node_type {
                NodeType::StatementsNode => {
                    self.emit_statements(child, indent_level + 1)?;
                    // Track the last statement's end line for blank line preservation
                    if let Some(last_child) = child.children.last() {
                        last_stmt_end_line = last_child.location.end_line;
                    }
                    self.buffer.push('\n');
                    break;
                }
                NodeType::BeginNode => {
                    // Block with rescue/ensure/else - delegate to emit_begin
                    // which handles implicit begin (no "begin" keyword)
                    self.emit_begin(child, indent_level + 1)?;
                    self.buffer.push('\n');
                    last_stmt_end_line = child.location.end_line;
                    break;
                }
                _ => {
                    // Skip parameter nodes
                }
            }
        }

        // Emit comments that are inside the block but not attached to any node
        // (comments between last statement and 'end')
        let had_internal_comments =
            self.has_comments_in_range(block_start_line + 1, block_end_line);
        if had_internal_comments {
            // Preserve blank line between last statement and first comment
            self.emit_comments_in_range_with_prev_line(
                block_start_line + 1,
                block_end_line,
                indent_level + 1,
                last_stmt_end_line,
            )?;
        }

        // Add newline if there were internal comments
        if had_internal_comments && !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }

        // Emit 'end'
        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;
        // Emit trailing comments on end line
        self.emit_trailing_comments(block_end_line)?;

        Ok(())
    }

    /// Emit a { } style block
    fn emit_brace_block(&mut self, block_node: &Node, indent_level: usize) -> Result<()> {
        // Determine if block should be inline or multiline
        let is_multiline = block_node.location.start_line != block_node.location.end_line;
        let block_end_line = block_node.location.end_line;

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
            self.emit_trailing_comments(block_end_line)?;
        } else {
            // Inline brace block - extract from source to preserve spacing
            write!(self.buffer, " ")?;
            if let Some(text) = self
                .source
                .get(block_node.location.start_offset..block_node.location.end_offset)
            {
                write!(self.buffer, "{}", text)?;
            }
            self.emit_trailing_comments(block_end_line)?;
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

                // Mark comments that are strictly inside this node's line range as emitted
                // (they are included in the source extraction)
                // Don't mark trailing comments on the last line (they come after the node ends)
                for (idx, comment) in self.all_comments.iter().enumerate() {
                    if !self.emitted_comment_indices.contains(&idx)
                        && comment.location.start_line >= node.location.start_line
                        && comment.location.end_line < node.location.end_line
                    {
                        self.emitted_comment_indices.insert(idx);
                    }
                }

                // Emit trailing comments on the same line (after the node ends)
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

                // Mark comments that are strictly inside this node's line range as emitted
                // (they are included in the source extraction)
                // Don't mark trailing comments on the last line (they come after the node ends)
                for (idx, comment) in self.all_comments.iter().enumerate() {
                    if !self.emitted_comment_indices.contains(&idx)
                        && comment.location.start_line >= node.location.start_line
                        && comment.location.end_line < node.location.end_line
                    {
                        self.emitted_comment_indices.insert(idx);
                    }
                }

                self.emit_trailing_comments(node.location.end_line)?;
            }
        }
        Ok(())
    }

    /// Ensure indent cache has entries up to and including the given level
    /// This allows pre-building the cache before borrowing self.indent_cache
    fn ensure_indent_cache(&mut self, level: usize) {
        while self.indent_cache.len() <= level {
            let len = self.indent_cache.len();
            let indent = match self.config.formatting.indent_style {
                IndentStyle::Spaces => " ".repeat(self.config.formatting.indent_width * len),
                IndentStyle::Tabs => "\t".repeat(len),
            };
            self.indent_cache.push(indent);
        }
    }

    /// Emit indentation
    fn emit_indent(&mut self, level: usize) -> Result<()> {
        self.ensure_indent_cache(level);
        write!(self.buffer, "{}", &self.indent_cache[level])?;
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
        // Emit trailing comments on end line
        self.emit_trailing_comments(node.location.end_line)?;

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
        // Emit trailing comments on end line
        self.emit_trailing_comments(node.location.end_line)?;

        Ok(())
    }

    /// Emit singleton class definition (class << self / class << object)
    fn emit_singleton_class(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;

        write!(self.buffer, "class << ")?;

        // First child is the expression (self or an object)
        if let Some(expression) = node.children.first() {
            if !self.source.is_empty() {
                let start = expression.location.start_offset;
                let end = expression.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, "{}", text)?;
                }
            }
        }

        // Emit trailing comments on the class << line
        self.emit_trailing_comments(node.location.start_line)?;
        self.buffer.push('\n');

        let class_start_line = node.location.start_line;
        let class_end_line = node.location.end_line;
        let mut has_body_content = false;

        // Emit body (skip the first child which is the expression)
        for (i, child) in node.children.iter().enumerate() {
            if i == 0 {
                // Skip the expression (self or object)
                continue;
            }
            if matches!(child.node_type, NodeType::StatementsNode) {
                has_body_content = true;
                self.emit_statements(child, indent_level + 1)?;
            } else if !self.is_structural_node(&child.node_type) {
                has_body_content = true;
                self.emit_node(child, indent_level + 1)?;
            }
        }

        // Emit comments that appear before the end statement while preserving their position
        self.emit_comments_before_end(class_start_line, class_end_line, indent_level + 1)?;

        // Add newline before end if there was body content
        if (has_body_content || self.has_comments_in_range(class_start_line + 1, class_end_line))
            && !self.buffer.ends_with('\n')
        {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;
        self.emit_trailing_comments(node.location.end_line)?;

        Ok(())
    }

    /// Emit case match (Ruby 3.0+ pattern matching with case...in)
    fn emit_case_match(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;

        // Write "case" keyword
        write!(self.buffer, "case")?;

        // Find predicate (first child that isn't InNode or ElseNode)
        let mut in_start_idx = 0;
        if let Some(first_child) = node.children.first() {
            if !matches!(first_child.node_type, NodeType::InNode | NodeType::ElseNode) {
                // This is the predicate - extract from source
                let start = first_child.location.start_offset;
                let end = first_child.location.end_offset;
                if let Some(text) = self.source.get(start..end) {
                    write!(self.buffer, " {}", text)?;
                }
                in_start_idx = 1;
            }
        }

        self.buffer.push('\n');

        // Emit in clauses and else
        for child in node.children.iter().skip(in_start_idx) {
            match &child.node_type {
                NodeType::InNode => {
                    self.emit_in(child, indent_level)?;
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
        self.emit_trailing_comments(node.location.end_line)?;

        Ok(())
    }

    /// Emit in node (pattern matching clause)
    fn emit_in(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        self.emit_comments_before(node.location.start_line, indent_level)?;
        self.emit_indent(indent_level)?;

        write!(self.buffer, "in ")?;

        // First child is the pattern
        if let Some(pattern) = node.children.first() {
            self.write_source_text(pattern)?;
        }

        if self.is_single_line(node) {
            // Inline style: in X then Y
            if let Some(statements) = node.children.get(1) {
                write!(self.buffer, " then ")?;
                self.write_source_text(statements)?;
            }
            self.buffer.push('\n');
        } else {
            // Multi-line style: in X\n  Y
            self.buffer.push('\n');

            // Second child is the statements body
            if let Some(statements) = node.children.get(1) {
                if matches!(statements.node_type, NodeType::StatementsNode) {
                    self.emit_statements(statements, indent_level + 1)?;
                }
            }
        }

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
                | NodeType::ForwardingParameterNode
                | NodeType::NoKeywordsParameterNode
        )
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

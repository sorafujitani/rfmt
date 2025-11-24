use crate::ast::{Comment, Node, NodeType};
use crate::config::{Config, IndentStyle};
use crate::error::Result;
use std::fmt::Write;

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

        // Collect all comments from the AST
        self.collect_comments(ast);

        self.emit_node(ast, 0)?;
        Ok(self.buffer.clone())
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
            IndentStyle::Spaces => {
                " ".repeat(self.config.formatting.indent_width * indent_level)
            }
            IndentStyle::Tabs => "\t".repeat(indent_level),
        };

        let mut indices_to_emit = Vec::new();
        for (idx, comment) in self.all_comments.iter().enumerate() {
            if self.emitted_comment_indices.contains(&idx) {
                continue;
            }

            // Collect comments that end before this line
            if comment.location.end_line < line {
                indices_to_emit.push((idx, comment.text.clone()));
            }
        }

        // Now emit the collected comments
        for (idx, text) in indices_to_emit {
            write!(self.buffer, "{}{}\n", indent_str, text)?;
            self.emitted_comment_indices.push(idx);
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
            _ => self.emit_generic(node, indent_level)?,
        }
        Ok(())
    }

    /// Emit program node (root)
    fn emit_program(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        for (i, child) in node.children.iter().enumerate() {
            self.emit_node(child, indent_level)?;

            // Add newline between top-level statements
            if i < node.children.len() - 1 {
                self.buffer.push('\n');
            }
        }
        Ok(())
    }

    /// Emit statements node (body of class/module/def)
    fn emit_statements(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        for (i, child) in node.children.iter().enumerate() {
            self.emit_node(child, indent_level)?;

            // Add newline between statements
            if i < node.children.len() - 1 {
                self.buffer.push('\n');
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

        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes like constant_read_node
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
            // Note: don't add newline here, statements node will handle it
        }

        // Add newline before end if there was body content
        if node.children.iter().any(|c| !self.is_structural_node(&c.node_type)) {
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

        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
        }

        // Add newline before end if there was body content
        if node.children.iter().any(|c| !self.is_structural_node(&c.node_type)) {
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

        if let Some(name) = node.metadata.get("name") {
            write!(self.buffer, "{}", name)?;
        }

        // TODO: Handle parameters properly
        // For now, extract from source if method has parameters
        if node.metadata.get("parameters_count").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0) > 0 {
            // Extract parameter part from source
            if !self.source.is_empty() && node.location.end_offset <= self.source.len() {
                if let Some(source_text) = self.source.get(node.location.start_offset..node.location.end_offset) {
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

        self.buffer.push('\n');

        // Emit body (children), but skip structural nodes like parameter nodes
        for child in &node.children {
            if self.is_structural_node(&child.node_type) {
                continue;
            }
            self.emit_node(child, indent_level + 1)?;
        }

        // Add newline before end if there was body content
        if node.children.iter().any(|c| !self.is_structural_node(&c.node_type)) {
            self.buffer.push('\n');
        }

        self.emit_indent(indent_level)?;
        write!(self.buffer, "end")?;

        Ok(())
    }

    /// Emit generic node by extracting from source
    fn emit_generic(&mut self, node: &Node, indent_level: usize) -> Result<()> {
        // Emit any comments before this node
        self.emit_comments_before(node.location.start_line, indent_level)?;

        if !self.source.is_empty() {
            let start = node.location.start_offset;
            let end = node.location.end_offset;

            // Clone text first to avoid borrow conflict
            let text_owned = self.source.get(start..end).map(|s| s.to_string());

            if let Some(text) = text_owned {
                // Add indentation before the extracted text
                self.emit_indent(indent_level)?;
                write!(self.buffer, "{}", text)?;

                // Emit any trailing comments on the same line
                self.emit_trailing_comments(node.location.end_line)?;
            }
        }
        Ok(())
    }

    /// Emit indentation
    fn emit_indent(&mut self, level: usize) -> Result<()> {
        let indent_str = match self.config.formatting.indent_style {
            IndentStyle::Spaces => {
                " ".repeat(self.config.formatting.indent_width * level)
            }
            IndentStyle::Tabs => "\t".repeat(level),
        };

        write!(self.buffer, "{}", indent_str)?;
        Ok(())
    }

    /// Check if node is structural (part of definition syntax, not body)
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
                | NodeType::KeywordRestParameterNode
                | NodeType::BlockParameterNode
        )
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

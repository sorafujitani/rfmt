//! FormatContext - State management for formatting
//!
//! FormatContext encapsulates all state needed during formatting:
//! - Source code reference
//! - Configuration
//! - Comment tracking and emission
//! - Group ID generation for Doc IR

use crate::ast::{Comment, Node};
use crate::config::Config;
use std::collections::{BTreeMap, HashSet};

/// Formatting context that manages state during AST traversal.
///
/// This struct is passed to FormatRules and provides access to:
/// - Source code for extraction
/// - Configuration settings
/// - Comment management (collection, emission tracking)
/// - Group ID generation for Doc IR
pub struct FormatContext<'a> {
    /// Reference to the configuration
    config: &'a Config,

    /// Reference to the source code
    source: &'a str,

    /// Source lines cached for efficient access
    source_lines: Vec<&'a str>,

    /// All comments collected from the AST
    all_comments: Vec<Comment>,

    /// Indices of comments that have been emitted
    emitted_comment_indices: HashSet<usize>,

    /// Index of comment indices by start line for O(log n) lookup
    /// Key: start_line, Value: Vec of comment indices that start on that line
    comments_by_line: BTreeMap<usize, Vec<usize>>,

    /// Counter for generating unique group IDs
    next_group_id: u32,
}

impl<'a> FormatContext<'a> {
    /// Creates a new FormatContext with the given configuration and source code.
    pub fn new(config: &'a Config, source: &'a str) -> Self {
        Self {
            config,
            source,
            source_lines: source.lines().collect(),
            all_comments: Vec::new(),
            emitted_comment_indices: HashSet::new(),
            comments_by_line: BTreeMap::new(),
            next_group_id: 0,
        }
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &Config {
        self.config
    }

    /// Returns a reference to the source code.
    pub fn source(&self) -> &str {
        self.source
    }

    /// Generates a new unique group ID for Doc IR.
    pub fn next_group_id(&mut self) -> u32 {
        let id = self.next_group_id;
        self.next_group_id += 1;
        id
    }

    /// Collects all comments from the AST recursively.
    pub fn collect_comments(&mut self, root: &Node) {
        self.all_comments.clear();
        self.emitted_comment_indices.clear();
        self.comments_by_line.clear();

        // Use iterative approach with stack to avoid deep recursion
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            // Reserve capacity hint based on typical comment count
            if self.all_comments.is_empty() && !node.comments.is_empty() {
                self.all_comments.reserve(node.comments.len() * 4);
            }
            self.all_comments.extend(node.comments.iter().cloned());
            // Process children in reverse order to maintain order when popping
            stack.extend(node.children.iter().rev());
        }

        self.build_comment_index();
    }

    /// Builds the comment index by start line for O(log n) range lookups.
    fn build_comment_index(&mut self) {
        for (idx, comment) in self.all_comments.iter().enumerate() {
            self.comments_by_line
                .entry(comment.location.start_line)
                .or_default()
                .push(idx);
        }
    }

    /// Gets comments that appear before a given line (not emitted yet).
    ///
    /// Returns comments where the entire comment ends before the given line.
    pub fn get_comments_before(&self, line: usize) -> Vec<&Comment> {
        self.comments_by_line
            .range(..line)
            .flat_map(|(_, indices)| indices.iter())
            .filter(|&&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < line
            })
            .map(|&idx| &self.all_comments[idx])
            .collect()
    }

    /// Gets trailing comments on a specific line (not emitted yet).
    ///
    /// Trailing comments are comments on the same line as code.
    pub fn get_trailing_comments(&self, line: usize) -> Vec<&Comment> {
        self.comments_by_line
            .get(&line)
            .map(|indices| {
                indices
                    .iter()
                    .filter(|&&idx| !self.emitted_comment_indices.contains(&idx))
                    .map(|&idx| &self.all_comments[idx])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets comments within a given line range [start_line, end_line).
    ///
    /// Only returns comments that haven't been emitted yet.
    pub fn get_comments_in_range(&self, start_line: usize, end_line: usize) -> Vec<&Comment> {
        if start_line >= end_line {
            return Vec::new();
        }

        self.comments_by_line
            .range(start_line..end_line)
            .flat_map(|(_, indices)| indices.iter())
            .filter(|&&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < end_line
            })
            .map(|&idx| &self.all_comments[idx])
            .collect()
    }

    /// Checks if there are any unemitted comments in the given line range.
    pub fn has_comments_in_range(&self, start_line: usize, end_line: usize) -> bool {
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

    /// Marks a comment as emitted by finding it in the collection.
    ///
    /// Uses the line index for O(log n) lookup instead of linear search.
    pub fn mark_comment_emitted(&mut self, comment: &Comment) {
        if let Some(indices) = self.comments_by_line.get(&comment.location.start_line) {
            for &idx in indices {
                let c = &self.all_comments[idx];
                if c.location == comment.location && c.text == comment.text {
                    self.emitted_comment_indices.insert(idx);
                    return;
                }
            }
        }
    }

    /// Marks a comment at the given index as emitted.
    #[inline]
    pub fn mark_comment_emitted_by_index(&mut self, idx: usize) {
        self.emitted_comment_indices.insert(idx);
    }

    /// Marks multiple comments as emitted by their indices.
    ///
    /// More efficient than calling mark_comment_emitted_by_index repeatedly.
    pub fn mark_comments_emitted(&mut self, indices: impl IntoIterator<Item = usize>) {
        self.emitted_comment_indices.extend(indices);
    }

    /// Extracts source text for a node.
    pub fn extract_source(&self, node: &Node) -> Option<&str> {
        self.source
            .get(node.location.start_offset..node.location.end_offset)
    }

    /// Extracts source text for a range of offsets.
    pub fn extract_source_range(&self, start: usize, end: usize) -> Option<&str> {
        self.source.get(start..end)
    }

    /// Checks if a comment is standalone (on its own line).
    ///
    /// A standalone comment has only whitespace before it on the same line.
    pub fn is_standalone_comment(&self, comment: &Comment) -> bool {
        let comment_line = comment.location.start_line;

        if comment_line == 0 || comment_line > self.source_lines.len() {
            return false;
        }

        let line = self.source_lines[comment_line - 1]; // Convert to 0-indexed

        if let Some(hash_pos) = line.find('#') {
            let before_comment = &line[..hash_pos];
            let is_only_whitespace = before_comment.bytes().all(|b| b == b' ' || b == b'\t');

            let line_comment_text = &line[hash_pos..];
            let is_same_comment = line_comment_text.trim_end() == comment.text.trim_end();

            return is_only_whitespace && is_same_comment;
        }

        false
    }

    /// Gets all remaining unemitted comments.
    ///
    /// Used for emitting comments at the end of the file.
    pub fn get_remaining_comments(&self) -> Vec<&Comment> {
        self.all_comments
            .iter()
            .enumerate()
            .filter(|(idx, _)| !self.emitted_comment_indices.contains(idx))
            .map(|(_, comment)| comment)
            .collect()
    }

    /// Gets comment indices before a given line (not emitted yet).
    ///
    /// Returns indices that can be used with `get_comment` and `mark_comment_emitted_by_index`.
    /// This avoids allocating comment data when only indices are needed.
    pub fn get_comment_indices_before(&self, line: usize) -> impl Iterator<Item = usize> + '_ {
        self.comments_by_line
            .range(..line)
            .flat_map(|(_, indices)| indices.iter().copied())
            .filter(move |&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < line
            })
    }

    /// Gets trailing comment indices on a specific line (not emitted yet).
    pub fn get_trailing_comment_indices(&self, line: usize) -> impl Iterator<Item = usize> + '_ {
        self.comments_by_line
            .get(&line)
            .into_iter()
            .flat_map(|indices| indices.iter().copied())
            .filter(move |&idx| !self.emitted_comment_indices.contains(&idx))
    }

    /// Gets comment indices within a given line range [start_line, end_line).
    pub fn get_comment_indices_in_range(
        &self,
        start_line: usize,
        end_line: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        // Handle invalid range (can happen with single-line nodes)
        let range_start = start_line.min(end_line);
        let range_end = start_line.max(end_line);

        self.comments_by_line
            .range(range_start..range_end)
            .flat_map(|(_, indices)| indices.iter().copied())
            .filter(move |&idx| {
                !self.emitted_comment_indices.contains(&idx)
                    && self.all_comments[idx].location.end_line < end_line
            })
    }

    /// Gets remaining comment indices (not emitted yet).
    pub fn get_remaining_comment_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.all_comments.len()).filter(|idx| !self.emitted_comment_indices.contains(idx))
    }

    /// Gets a comment by index.
    #[inline]
    pub fn get_comment(&self, idx: usize) -> Option<&Comment> {
        self.all_comments.get(idx)
    }

    /// Gets the last line of code in the AST (excluding comments).
    pub fn find_last_code_line(ast: &Node) -> usize {
        let mut max_line = ast.location.end_line;
        let mut stack = vec![ast];

        while let Some(node) = stack.pop() {
            max_line = max_line.max(node.location.end_line);
            stack.extend(node.children.iter());
        }

        max_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{CommentPosition, CommentType, FormattingInfo, Location, NodeType};
    use std::collections::HashMap;

    fn make_comment(text: &str, start_line: usize) -> Comment {
        Comment {
            text: text.to_string(),
            location: Location::new(start_line, 0, start_line, text.len(), 0, text.len()),
            comment_type: CommentType::Line,
            position: CommentPosition::Leading,
        }
    }

    fn make_node_with_comments(comments: Vec<Comment>) -> Node {
        Node {
            node_type: NodeType::ProgramNode,
            location: Location::new(1, 0, 10, 0, 0, 100),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments,
            formatting: FormattingInfo::default(),
        }
    }

    #[test]
    fn test_collect_comments() {
        let config = Config::default();
        let source = "# comment\nclass Foo\nend";
        let mut ctx = FormatContext::new(&config, source);

        let comment = make_comment("# comment", 1);
        let node = make_node_with_comments(vec![comment]);

        ctx.collect_comments(&node);

        let comments = ctx.get_comments_before(10);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "# comment");
    }

    #[test]
    fn test_mark_comment_emitted() {
        let config = Config::default();
        let source = "# comment\ncode";
        let mut ctx = FormatContext::new(&config, source);

        let comment = make_comment("# comment", 1);
        let node = make_node_with_comments(vec![comment.clone()]);

        ctx.collect_comments(&node);

        // Before marking
        assert_eq!(ctx.get_comments_before(10).len(), 1);

        // Mark as emitted
        ctx.mark_comment_emitted(&comment);

        // After marking
        assert_eq!(ctx.get_comments_before(10).len(), 0);
    }

    #[test]
    fn test_get_comments_in_range() {
        let config = Config::default();
        let source = "# comment\ncode";
        let mut ctx = FormatContext::new(&config, source);

        let comment1 = make_comment("# comment 1", 2);
        let comment2 = make_comment("# comment 2", 5);
        let comment3 = make_comment("# comment 3", 8);
        let node = make_node_with_comments(vec![comment1, comment2, comment3]);

        ctx.collect_comments(&node);

        let comments = ctx.get_comments_in_range(3, 7);
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "# comment 2");
    }

    #[test]
    fn test_trailing_comments() {
        let config = Config::default();
        let source = "code # trailing";
        let mut ctx = FormatContext::new(&config, source);

        let comment = Comment {
            text: "# trailing".to_string(),
            location: Location::new(1, 5, 1, 15, 5, 15),
            comment_type: CommentType::Line,
            position: CommentPosition::Trailing,
        };
        let node = make_node_with_comments(vec![comment]);

        ctx.collect_comments(&node);

        let trailing = ctx.get_trailing_comments(1);
        assert_eq!(trailing.len(), 1);
        assert_eq!(trailing[0].text, "# trailing");
    }

    #[test]
    fn test_next_group_id() {
        let config = Config::default();
        let source = "";
        let mut ctx = FormatContext::new(&config, source);

        assert_eq!(ctx.next_group_id(), 0);
        assert_eq!(ctx.next_group_id(), 1);
        assert_eq!(ctx.next_group_id(), 2);
    }

    #[test]
    fn test_extract_source() {
        let config = Config::default();
        let source = "class Foo\nend";
        let ctx = FormatContext::new(&config, source);

        let node = Node {
            node_type: NodeType::ClassNode,
            location: Location::new(1, 0, 2, 3, 0, 13),
            children: Vec::new(),
            metadata: HashMap::new(),
            comments: Vec::new(),
            formatting: FormattingInfo::default(),
        };

        let extracted = ctx.extract_source(&node);
        assert_eq!(extracted, Some("class Foo\nend"));
    }
}

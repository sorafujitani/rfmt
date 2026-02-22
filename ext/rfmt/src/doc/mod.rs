//! Doc IR (Intermediate Representation) for rfmt
//!
//! This module provides a Prettier/Biome-style intermediate representation
//! for code formatting. The Doc IR separates formatting intent from string generation,
//! enabling automatic line breaking and consistent formatting.
//!
//! # Architecture
//!
//! ```text
//! AST → DocBuilder → Doc IR → Printer → String
//! ```
//!
//! # Example
//!
//! ```rust
//! use rfmt::doc::builders::*;
//! use rfmt::doc::Printer;
//!
//! let doc = concat(vec![
//!     text("def foo"),
//!     hardline(),
//!     indent(text("body")),
//!     hardline(),
//!     text("end"),
//! ]);
//!
//! let mut printer = Printer::new(config);
//! let output = printer.print(&doc);
//! // => "def foo\n  body\nend"
//! ```

pub mod builders;
pub mod printer;

pub use builders::*;
pub use printer::Printer;

/// Document intermediate representation for code formatting.
///
/// Doc is a tree structure that describes how code should be formatted
/// without committing to specific line breaks. The Printer converts
/// Doc to a string, making line break decisions based on line width.
#[derive(Debug, Clone, PartialEq)]
pub enum Doc {
    /// A text literal that is printed as-is.
    ///
    /// # Example
    /// ```rust
    /// Doc::Text("hello".to_string())
    /// ```
    Text(String),

    /// Concatenation of multiple documents.
    ///
    /// # Example
    /// ```rust
    /// Doc::Concat(vec![Doc::Text("a".into()), Doc::Text("b".into())])
    /// // Prints: "ab"
    /// ```
    Concat(Vec<Doc>),

    /// A group that tries to fit on one line.
    ///
    /// If the contents fit within the remaining line width, Line docs
    /// inside are replaced with spaces. Otherwise, they become newlines.
    ///
    /// # Fields
    /// - `contents`: The document to group
    /// - `break_parent`: If true, forces parent groups to break
    /// - `id`: Optional identifier for IfBreak references
    Group {
        contents: Box<Doc>,
        break_parent: bool,
        id: Option<GroupId>,
    },

    /// A potential line break.
    ///
    /// # Variants (controlled by fields)
    /// - `soft=false, hard=false`: Space if flat, newline if broken
    /// - `soft=true, hard=false`: Nothing if flat, newline if broken
    /// - `hard=true`: Always a newline
    /// - `literal=true`: Newline without indentation (for heredocs)
    Line {
        soft: bool,
        hard: bool,
        literal: bool,
    },

    /// Increases indentation for the contents.
    ///
    /// # Example
    /// ```rust
    /// concat(vec![
    ///     text("if true"),
    ///     hardline(),
    ///     indent(text("body")),
    ///     hardline(),
    ///     text("end"),
    /// ])
    /// // Prints:
    /// // if true
    /// //   body
    /// // end
    /// ```
    Indent(Box<Doc>),

    /// Conditional content based on group break state.
    ///
    /// If the group is broken (multi-line), prints `break_contents`.
    /// If the group is flat (single line), prints `flat_contents`.
    ///
    /// # Fields
    /// - `break_contents`: Printed when group is broken
    /// - `flat_contents`: Printed when group is flat
    /// - `group_id`: Optional reference to a specific group
    IfBreak {
        break_contents: Box<Doc>,
        flat_contents: Box<Doc>,
        group_id: Option<GroupId>,
    },

    /// An empty document that prints nothing.
    Empty,

    /// A trailing comment that appears at the end of a line.
    ///
    /// # Example
    /// ```rust
    /// concat(vec![text("code"), Doc::TrailingComment("# comment".into())])
    /// // Prints: "code # comment"
    /// ```
    TrailingComment(String),

    /// A leading comment that appears before code.
    ///
    /// # Fields
    /// - `text`: The comment text
    /// - `hard_line_after`: Whether to add a hard line after the comment
    LeadingComment { text: String, hard_line_after: bool },

    /// Aligns content to a specific column offset.
    ///
    /// Used for aligning continuation lines or heredoc content.
    Align { n: usize, contents: Box<Doc> },

    /// Content to be appended at the end of the line.
    ///
    /// Useful for trailing commas that should only appear in multi-line mode.
    LineSuffix(Box<Doc>),

    /// Fill: packs content into lines as tightly as possible.
    ///
    /// Used for array elements, argument lists, etc.
    Fill(Vec<Doc>),
}

/// Identifier for referencing groups in IfBreak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupId(pub u32);

impl Doc {
    /// Returns true if this doc is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, Doc::Empty)
    }

    /// Returns true if this doc contains only text (no line breaks possible).
    pub fn is_flat(&self) -> bool {
        match self {
            Doc::Text(_) => true,
            Doc::Concat(docs) => docs.iter().all(|d| d.is_flat()),
            Doc::Group { contents, .. } => contents.is_flat(),
            Doc::Indent(contents) => contents.is_flat(),
            Doc::IfBreak { flat_contents, .. } => flat_contents.is_flat(),
            Doc::Empty => true,
            Doc::TrailingComment(_) | Doc::LeadingComment { .. } => true,
            Doc::Line { .. } => false,
            Doc::Align { contents, .. } => contents.is_flat(),
            Doc::LineSuffix(_) => true,
            Doc::Fill(docs) => docs.iter().all(|d| d.is_flat()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_is_empty() {
        assert!(Doc::Empty.is_empty());
        assert!(!Doc::Text("hello".into()).is_empty());
    }

    #[test]
    fn test_doc_is_flat() {
        assert!(Doc::Text("hello".into()).is_flat());
        assert!(Doc::Empty.is_flat());
        assert!(Doc::Concat(vec![Doc::Text("a".into()), Doc::Text("b".into())]).is_flat());

        // Line is not flat
        assert!(!Doc::Line {
            soft: false,
            hard: false,
            literal: false
        }
        .is_flat());
    }

    #[test]
    fn test_group_id_equality() {
        let id1 = GroupId(1);
        let id2 = GroupId(1);
        let id3 = GroupId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}

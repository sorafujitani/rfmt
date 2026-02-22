//! Builder functions for constructing Doc IR.
//!
//! These functions provide a convenient API for building Doc trees.
//! They are inspired by Prettier's document builders.
//!
//! # Example
//!
//! ```rust
//! use rfmt::doc::builders::*;
//!
//! // Build a simple Ruby method
//! let doc = concat(vec![
//!     text("def foo"),
//!     hardline(),
//!     indent(concat(vec![
//!         text("puts \"hello\""),
//!     ])),
//!     hardline(),
//!     text("end"),
//! ]);
//! ```

use super::{Doc, GroupId};

/// Creates a text document.
///
/// # Example
/// ```rust
/// let doc = text("hello");
/// ```
pub fn text<S: Into<String>>(s: S) -> Doc {
    Doc::Text(s.into())
}

/// Concatenates multiple documents.
///
/// # Example
/// ```rust
/// let doc = concat(vec![text("a"), text("b"), text("c")]);
/// // Prints: "abc"
/// ```
pub fn concat(docs: Vec<Doc>) -> Doc {
    // Flatten nested Concats and remove Empty docs
    let flattened: Vec<Doc> = docs
        .into_iter()
        .flat_map(|doc| match doc {
            Doc::Concat(inner) => inner,
            Doc::Empty => vec![],
            other => vec![other],
        })
        .collect();

    match flattened.len() {
        0 => Doc::Empty,
        1 => flattened.into_iter().next().unwrap(),
        _ => Doc::Concat(flattened),
    }
}

/// Creates a group that tries to fit on one line.
///
/// If the contents fit within the line width, Line docs become spaces.
/// Otherwise, they become newlines.
///
/// # Example
/// ```rust
/// let doc = group(concat(vec![
///     text("["),
///     softline(),
///     text("1, 2, 3"),
///     softline(),
///     text("]"),
/// ]));
/// // Short version: "[1, 2, 3]"
/// // Long version:
/// // [
/// //   1, 2, 3
/// // ]
/// ```
pub fn group(contents: Doc) -> Doc {
    Doc::Group {
        contents: Box::new(contents),
        break_parent: false,
        id: None,
    }
}

/// Creates a group with a specific ID for IfBreak references.
pub fn group_with_id(contents: Doc, id: GroupId) -> Doc {
    Doc::Group {
        contents: Box::new(contents),
        break_parent: false,
        id: Some(id),
    }
}

/// Creates a group that forces parent groups to break.
pub fn group_break(contents: Doc) -> Doc {
    Doc::Group {
        contents: Box::new(contents),
        break_parent: true,
        id: None,
    }
}

/// Increases indentation for the contents.
///
/// # Example
/// ```rust
/// let doc = concat(vec![
///     text("class Foo"),
///     hardline(),
///     indent(text("def bar; end")),
///     hardline(),
///     text("end"),
/// ]);
/// // Prints:
/// // class Foo
/// //   def bar; end
/// // end
/// ```
pub fn indent(contents: Doc) -> Doc {
    Doc::Indent(Box::new(contents))
}

/// A line break that becomes a space in flat mode.
///
/// - In flat mode (group fits on one line): prints a space
/// - In break mode (group doesn't fit): prints a newline + indentation
///
/// # Example
/// ```rust
/// group(concat(vec![text("a"), line(), text("b")]))
/// // Flat: "a b"
/// // Break: "a\n  b"
/// ```
pub fn line() -> Doc {
    Doc::Line {
        soft: false,
        hard: false,
        literal: false,
    }
}

/// A line break that disappears in flat mode.
///
/// - In flat mode: prints nothing
/// - In break mode: prints a newline + indentation
///
/// # Example
/// ```rust
/// group(concat(vec![text("["), softline(), text("1"), softline(), text("]")]))
/// // Flat: "[1]"
/// // Break: "[\n  1\n]"
/// ```
pub fn softline() -> Doc {
    Doc::Line {
        soft: true,
        hard: false,
        literal: false,
    }
}

/// A line break that always prints a newline.
///
/// This forces a line break regardless of whether the content fits.
///
/// # Example
/// ```rust
/// concat(vec![text("line1"), hardline(), text("line2")])
/// // Always prints:
/// // line1
/// // line2
/// ```
pub fn hardline() -> Doc {
    Doc::Line {
        soft: false,
        hard: true,
        literal: false,
    }
}

/// A literal line break without indentation.
///
/// Used for heredocs where the content should not be indented.
///
/// # Example
/// ```rust
/// concat(vec![text("<<~HEREDOC"), literalline(), text("content"), literalline(), text("HEREDOC")])
/// ```
pub fn literalline() -> Doc {
    Doc::Line {
        soft: false,
        hard: true,
        literal: true,
    }
}

/// Conditional content based on group break state.
///
/// # Arguments
/// - `break_contents`: Printed when the group is broken (multi-line)
/// - `flat_contents`: Printed when the group is flat (single line)
///
/// # Example
/// ```rust
/// group(concat(vec![
///     text("["),
///     if_break(
///         concat(vec![hardline(), indent(text("1, 2, 3")), hardline()]),
///         text("1, 2, 3"),
///     ),
///     text("]"),
/// ]))
/// // Flat: "[1, 2, 3]"
/// // Break:
/// // [
/// //   1, 2, 3
/// // ]
/// ```
pub fn if_break(break_contents: Doc, flat_contents: Doc) -> Doc {
    Doc::IfBreak {
        break_contents: Box::new(break_contents),
        flat_contents: Box::new(flat_contents),
        group_id: None,
    }
}

/// Conditional content referencing a specific group.
pub fn if_break_with_group(break_contents: Doc, flat_contents: Doc, group_id: GroupId) -> Doc {
    Doc::IfBreak {
        break_contents: Box::new(break_contents),
        flat_contents: Box::new(flat_contents),
        group_id: Some(group_id),
    }
}

/// Joins documents with a separator.
///
/// # Example
/// ```rust
/// join(text(", "), vec![text("a"), text("b"), text("c")])
/// // Prints: "a, b, c"
/// ```
pub fn join(separator: Doc, docs: Vec<Doc>) -> Doc {
    if docs.is_empty() {
        return Doc::Empty;
    }

    let mut result = Vec::with_capacity(docs.len() * 2 - 1);
    let mut first = true;

    for doc in docs {
        if !first {
            result.push(separator.clone());
        }
        result.push(doc);
        first = false;
    }

    concat(result)
}

/// Joins documents with a line separator.
///
/// In flat mode, uses spaces. In break mode, uses newlines.
///
/// # Example
/// ```rust
/// group(join_line(vec![text("a"), text("b"), text("c")]))
/// // Flat: "a b c"
/// // Break:
/// // a
/// // b
/// // c
/// ```
pub fn join_line(docs: Vec<Doc>) -> Doc {
    join(line(), docs)
}

/// Joins documents with a softline separator.
pub fn join_softline(docs: Vec<Doc>) -> Doc {
    join(softline(), docs)
}

/// Joins documents with a hardline separator.
pub fn join_hardline(docs: Vec<Doc>) -> Doc {
    join(hardline(), docs)
}

/// Creates an empty document.
pub fn empty() -> Doc {
    Doc::Empty
}

/// Creates a trailing comment.
///
/// # Example
/// ```rust
/// concat(vec![text("code"), trailing_comment("# comment")])
/// // Prints: "code # comment"
/// ```
pub fn trailing_comment<S: Into<String>>(text: S) -> Doc {
    Doc::TrailingComment(text.into())
}

/// Creates a leading comment.
///
/// # Example
/// ```rust
/// concat(vec![leading_comment("# comment", true), text("code")])
/// // Prints:
/// // # comment
/// // code
/// ```
pub fn leading_comment<S: Into<String>>(text: S, hard_line_after: bool) -> Doc {
    Doc::LeadingComment {
        text: text.into(),
        hard_line_after,
    }
}

/// Aligns content to a specific column offset.
pub fn align(n: usize, contents: Doc) -> Doc {
    Doc::Align {
        n,
        contents: Box::new(contents),
    }
}

/// Content to be appended at the end of the line.
pub fn line_suffix(contents: Doc) -> Doc {
    Doc::LineSuffix(Box::new(contents))
}

/// Fill: packs content into lines as tightly as possible.
pub fn fill(docs: Vec<Doc>) -> Doc {
    Doc::Fill(docs)
}

/// Creates multiple hardlines (blank lines).
///
/// # Example
/// ```rust
/// blank_lines(2)
/// // Prints two newlines (one blank line between)
/// ```
pub fn blank_lines(count: usize) -> Doc {
    if count == 0 {
        return Doc::Empty;
    }

    let lines: Vec<Doc> = (0..count).map(|_| hardline()).collect();
    concat(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text() {
        let doc = text("hello");
        assert_eq!(doc, Doc::Text("hello".to_string()));
    }

    #[test]
    fn test_concat_flattens() {
        let doc = concat(vec![
            text("a"),
            concat(vec![text("b"), text("c")]),
            text("d"),
        ]);

        // Should flatten to a single Concat with 4 elements
        if let Doc::Concat(docs) = doc {
            assert_eq!(docs.len(), 4);
        } else {
            panic!("Expected Concat");
        }
    }

    #[test]
    fn test_concat_removes_empty() {
        let doc = concat(vec![text("a"), empty(), text("b")]);

        if let Doc::Concat(docs) = doc {
            assert_eq!(docs.len(), 2);
        } else {
            panic!("Expected Concat");
        }
    }

    #[test]
    fn test_concat_single_element() {
        let doc = concat(vec![text("a")]);
        assert_eq!(doc, Doc::Text("a".to_string()));
    }

    #[test]
    fn test_concat_empty() {
        let doc = concat(vec![]);
        assert_eq!(doc, Doc::Empty);
    }

    #[test]
    fn test_group() {
        let doc = group(text("hello"));
        if let Doc::Group {
            contents,
            break_parent,
            id,
        } = doc
        {
            assert_eq!(*contents, Doc::Text("hello".to_string()));
            assert!(!break_parent);
            assert!(id.is_none());
        } else {
            panic!("Expected Group");
        }
    }

    #[test]
    fn test_indent() {
        let doc = indent(text("body"));
        if let Doc::Indent(contents) = doc {
            assert_eq!(*contents, Doc::Text("body".to_string()));
        } else {
            panic!("Expected Indent");
        }
    }

    #[test]
    fn test_line_variants() {
        let l = line();
        assert_eq!(
            l,
            Doc::Line {
                soft: false,
                hard: false,
                literal: false
            }
        );

        let sl = softline();
        assert_eq!(
            sl,
            Doc::Line {
                soft: true,
                hard: false,
                literal: false
            }
        );

        let hl = hardline();
        assert_eq!(
            hl,
            Doc::Line {
                soft: false,
                hard: true,
                literal: false
            }
        );

        let ll = literalline();
        assert_eq!(
            ll,
            Doc::Line {
                soft: false,
                hard: true,
                literal: true
            }
        );
    }

    #[test]
    fn test_join() {
        let doc = join(text(", "), vec![text("a"), text("b"), text("c")]);

        if let Doc::Concat(docs) = doc {
            assert_eq!(docs.len(), 5); // a, ", ", b, ", ", c
        } else {
            panic!("Expected Concat");
        }
    }

    #[test]
    fn test_join_empty() {
        let doc = join(text(", "), vec![]);
        assert_eq!(doc, Doc::Empty);
    }

    #[test]
    fn test_join_single() {
        let doc = join(text(", "), vec![text("a")]);
        assert_eq!(doc, Doc::Text("a".to_string()));
    }

    #[test]
    fn test_if_break() {
        let doc = if_break(text("broken"), text("flat"));
        if let Doc::IfBreak {
            break_contents,
            flat_contents,
            group_id,
        } = doc
        {
            assert_eq!(*break_contents, Doc::Text("broken".to_string()));
            assert_eq!(*flat_contents, Doc::Text("flat".to_string()));
            assert!(group_id.is_none());
        } else {
            panic!("Expected IfBreak");
        }
    }

    #[test]
    fn test_blank_lines() {
        let doc = blank_lines(0);
        assert_eq!(doc, Doc::Empty);

        let doc = blank_lines(2);
        if let Doc::Concat(docs) = doc {
            assert_eq!(docs.len(), 2);
        } else {
            panic!("Expected Concat");
        }
    }
}

//! Printer converts Doc IR to a formatted string.
//!
//! The printer implements a Prettier-style algorithm that:
//! 1. Tries to fit groups on a single line
//! 2. Breaks groups into multiple lines when they don't fit
//! 3. Manages indentation automatically
//!
//! # Algorithm
//!
//! The printer uses a stack-based approach to process Doc nodes.
//! For each Group, it measures whether the contents fit in the remaining
//! line width. If they fit, Line docs become spaces (flat mode).
//! If they don't fit, Line docs become newlines (break mode).

use super::Doc;
use crate::config::{Config, IndentStyle};

/// Print mode for Line docs within a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Group fits on one line. Line docs become spaces.
    Flat,
    /// Group doesn't fit. Line docs become newlines.
    Break,
}

/// A command in the print stack.
#[derive(Debug)]
struct PrintCommand<'a> {
    /// Current indentation level (in spaces)
    indent: usize,
    /// Print mode (Flat or Break)
    mode: Mode,
    /// The Doc to print
    doc: &'a Doc,
}

/// Converts Doc IR to a formatted string.
pub struct Printer<'a> {
    /// Configuration for formatting
    config: &'a Config,
    /// Output buffer
    output: String,
    /// Current column position (0-indexed)
    pos: usize,
    /// Pre-computed indent strings by width (avoids allocation during print)
    indent_cache: Vec<String>,
}

const MAX_PRECACHED_INDENT: usize = 32;

impl<'a> Printer<'a> {
    /// Creates a new printer with the given configuration.
    pub fn new(config: &'a Config) -> Self {
        let indent_width = config.formatting.indent_width;
        let indent_style = &config.formatting.indent_style;

        let indent_cache = (0..=MAX_PRECACHED_INDENT)
            .map(|width| Self::build_indent_string(width, indent_width, indent_style))
            .collect();

        Self {
            config,
            output: String::new(),
            pos: 0,
            indent_cache,
        }
    }

    #[inline]
    fn build_indent_string(width: usize, indent_width: usize, style: &IndentStyle) -> String {
        match style {
            IndentStyle::Spaces => " ".repeat(width),
            IndentStyle::Tabs if indent_width == 0 => String::new(),
            IndentStyle::Tabs => {
                let tabs = width / indent_width;
                let spaces = width % indent_width;
                if spaces == 0 {
                    "\t".repeat(tabs)
                } else {
                    let mut result = String::with_capacity(tabs + spaces);
                    result.extend(std::iter::repeat_n('\t', tabs));
                    result.extend(std::iter::repeat_n(' ', spaces));
                    result
                }
            }
        }
    }

    /// Prints a Doc to a string.
    #[inline]
    pub fn print(&mut self, doc: &Doc) -> String {
        self.output.clear();
        self.pos = 0;

        let mut commands: Vec<PrintCommand> = vec![PrintCommand {
            indent: 0,
            mode: Mode::Break, // Start in break mode
            doc,
        }];

        while let Some(cmd) = commands.pop() {
            self.process_command(cmd, &mut commands);
        }

        // Ensure trailing newline
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        std::mem::take(&mut self.output)
    }

    /// Processes a single print command.
    #[inline]
    fn process_command<'b>(&mut self, cmd: PrintCommand<'b>, commands: &mut Vec<PrintCommand<'b>>) {
        match cmd.doc {
            Doc::Text(s) => {
                self.output.push_str(s);
                self.pos += s.chars().count();
            }

            Doc::Concat(docs) => {
                // Push in reverse order so first doc is processed first
                for doc in docs.iter().rev() {
                    commands.push(PrintCommand {
                        indent: cmd.indent,
                        mode: cmd.mode,
                        doc,
                    });
                }
            }

            Doc::Group {
                contents,
                break_parent,
                ..
            } => {
                // Determine if the group fits on the remaining line
                let remaining = self.config.formatting.line_length.saturating_sub(self.pos);
                let fits = !*break_parent && self.fits(contents, cmd.indent, remaining, cmd.mode);

                let mode = if fits { Mode::Flat } else { Mode::Break };

                commands.push(PrintCommand {
                    indent: cmd.indent,
                    mode,
                    doc: contents,
                });
            }

            Doc::Line {
                soft,
                hard,
                literal,
            } => match (cmd.mode, *hard) {
                (_, true) => {
                    self.output.push('\n');
                    if !*literal {
                        let indent_str = self.get_indent(cmd.indent);
                        self.output.push_str(&indent_str);
                        self.pos = cmd.indent;
                    } else {
                        self.pos = 0;
                    }
                }
                (Mode::Flat, false) if *soft => {}
                (Mode::Flat, false) => {
                    self.output.push(' ');
                    self.pos += 1;
                }
                (Mode::Break, false) => {
                    self.output.push('\n');
                    let indent_str = self.get_indent(cmd.indent);
                    self.output.push_str(&indent_str);
                    self.pos = cmd.indent;
                }
            },

            Doc::Indent(contents) => {
                let new_indent = cmd.indent + self.config.formatting.indent_width;
                commands.push(PrintCommand {
                    indent: new_indent,
                    mode: cmd.mode,
                    doc: contents,
                });
            }

            Doc::IfBreak {
                break_contents,
                flat_contents,
                ..
            } => {
                let doc = match cmd.mode {
                    Mode::Break => break_contents,
                    Mode::Flat => flat_contents,
                };
                commands.push(PrintCommand {
                    indent: cmd.indent,
                    mode: cmd.mode,
                    doc,
                });
            }

            Doc::Empty => {}

            Doc::TrailingComment(text) => {
                self.output.push(' ');
                self.output.push_str(text);
                self.pos += 1 + text.chars().count();
            }

            Doc::LeadingComment {
                text,
                hard_line_after,
            } => {
                self.output.push_str(text);
                self.pos += text.chars().count();
                if *hard_line_after {
                    self.output.push('\n');
                    let indent_str = self.get_indent(cmd.indent);
                    self.output.push_str(&indent_str);
                    self.pos = cmd.indent;
                }
            }

            Doc::Align { n, contents } => {
                let new_indent = cmd.indent + n;
                commands.push(PrintCommand {
                    indent: new_indent,
                    mode: cmd.mode,
                    doc: contents,
                });
            }

            Doc::LineSuffix(contents) => {
                // For now, just print inline. Proper line suffix handling
                // would buffer these until end of line.
                commands.push(PrintCommand {
                    indent: cmd.indent,
                    mode: cmd.mode,
                    doc: contents,
                });
            }

            Doc::Fill(docs) => {
                // Fill tries to fit as many items on each line as possible.
                // For now, simple implementation: join with softline behavior.
                for (i, doc) in docs.iter().rev().enumerate() {
                    if i > 0 {
                        commands.push(PrintCommand {
                            indent: cmd.indent,
                            mode: cmd.mode,
                            doc: &Doc::Line {
                                soft: true,
                                hard: false,
                                literal: false,
                            },
                        });
                    }
                    commands.push(PrintCommand {
                        indent: cmd.indent,
                        mode: cmd.mode,
                        doc,
                    });
                }
            }
        }
    }

    /// Determines if a Doc fits within the remaining width.
    ///
    /// Uses flat mode for inner groups and returns early when width is exceeded.
    /// This is a hot path, so we pre-allocate a reasonable stack size.
    #[inline]
    fn fits(&self, doc: &Doc, indent: usize, remaining: usize, mode: Mode) -> bool {
        let mut width = 0usize;
        // Pre-allocate stack with reasonable capacity for typical nesting depth
        let mut stack: Vec<(&Doc, usize, Mode)> = Vec::with_capacity(16);
        stack.push((doc, indent, mode));

        while let Some((doc, indent, mode)) = stack.pop() {
            // Early exit: use >= for slightly earlier termination
            if width >= remaining {
                return false;
            }

            match doc {
                Doc::Text(s) => {
                    // Use len() for ASCII strings (common case), chars().count() for accuracy
                    width += if s.is_ascii() {
                        s.len()
                    } else {
                        s.chars().count()
                    };
                }

                Doc::Concat(docs) => {
                    for d in docs.iter().rev() {
                        stack.push((d, indent, mode));
                    }
                }

                Doc::Group { contents, .. } => {
                    // Assume nested groups stay flat during fit check
                    stack.push((contents, indent, Mode::Flat));
                }

                Doc::Line { soft, hard, .. } => {
                    if *hard {
                        // Hard line forces a break - content after fits on new line
                        return true;
                    }
                    match mode {
                        Mode::Flat if *soft => {}   // soft line: nothing
                        Mode::Flat => width += 1,   // regular line: space
                        Mode::Break => return true, // break mode: newline ends measurement
                    }
                }

                Doc::Indent(contents) => {
                    let new_indent = indent + self.config.formatting.indent_width;
                    stack.push((contents, new_indent, mode));
                }

                Doc::IfBreak {
                    flat_contents,
                    break_contents,
                    ..
                } => {
                    let contents = match mode {
                        Mode::Flat => flat_contents,
                        Mode::Break => break_contents,
                    };
                    stack.push((contents, indent, mode));
                }

                Doc::Empty => {}

                Doc::TrailingComment(s) => {
                    width += 1 + if s.is_ascii() {
                        s.len()
                    } else {
                        s.chars().count()
                    };
                }

                Doc::LeadingComment { text, .. } => {
                    width += if text.is_ascii() {
                        text.len()
                    } else {
                        text.chars().count()
                    };
                }

                Doc::Align { n, contents } => {
                    stack.push((contents, indent + n, mode));
                }

                Doc::LineSuffix(contents) => {
                    stack.push((contents, indent, mode));
                }

                Doc::Fill(docs) => {
                    for d in docs.iter().rev() {
                        stack.push((d, indent, mode));
                    }
                }
            }
        }

        width <= remaining
    }

    fn get_indent(&mut self, width: usize) -> String {
        if width < self.indent_cache.len() {
            return self.indent_cache[width].clone();
        }

        let indent_width = self.config.formatting.indent_width;
        let indent_style = &self.config.formatting.indent_style;
        while self.indent_cache.len() <= width {
            let w = self.indent_cache.len();
            self.indent_cache
                .push(Self::build_indent_string(w, indent_width, indent_style));
        }
        self.indent_cache[width].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::builders::*;

    /// Helper to print a doc with default config.
    fn print_doc(doc: &Doc) -> String {
        let config = Config::default();
        let mut printer = Printer::new(&config);
        printer.print(doc)
    }

    #[test]
    fn test_print_text() {
        let doc = text("hello");
        let result = print_doc(&doc);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_print_concat() {
        let doc = concat(vec![text("a"), text("b"), text("c")]);
        let result = print_doc(&doc);
        assert_eq!(result, "abc\n");
    }

    #[test]
    fn test_print_hardline() {
        let doc = concat(vec![text("line1"), hardline(), text("line2")]);
        let result = print_doc(&doc);
        assert_eq!(result, "line1\nline2\n");
    }

    #[test]
    fn test_print_indent() {
        // Correct Doc structure: hardline inside indent
        let doc = concat(vec![
            text("def foo"),
            indent(concat(vec![hardline(), text("body")])),
            hardline(),
            text("end"),
        ]);
        let result = print_doc(&doc);
        assert_eq!(result, "def foo\n  body\nend\n");
    }

    #[test]
    fn test_print_nested_indent() {
        // Correct Doc structure: hardline inside indent
        let doc = concat(vec![
            text("class Foo"),
            indent(concat(vec![
                hardline(),
                text("def bar"),
                indent(concat(vec![hardline(), text("puts 'hello'")])),
                hardline(),
                text("end"),
            ])),
            hardline(),
            text("end"),
        ]);
        let result = print_doc(&doc);
        assert_eq!(
            result,
            "class Foo\n  def bar\n    puts 'hello'\n  end\nend\n"
        );
    }

    #[test]
    fn test_print_group_fits() {
        // Short content should stay on one line
        let doc = group(concat(vec![text("short"), line(), text("text")]));
        let result = print_doc(&doc);
        assert_eq!(result, "short text\n");
    }

    #[test]
    fn test_print_group_breaks() {
        // Create content that doesn't fit on one line
        let long = "a".repeat(80);
        let doc = group(concat(vec![text(&long), line(), text("more")]));
        let result = print_doc(&doc);
        assert!(result.contains('\n'));
        // Should break: long\nmore
        assert!(result.starts_with(&long));
    }

    #[test]
    fn test_print_softline_flat() {
        // Softline disappears in flat mode
        let doc = group(concat(vec![
            text("["),
            softline(),
            text("1"),
            softline(),
            text("]"),
        ]));
        let result = print_doc(&doc);
        assert_eq!(result, "[1]\n");
    }

    #[test]
    fn test_print_if_break_flat() {
        let doc = group(concat(vec![
            text("["),
            if_break(text("BROKEN"), text("FLAT")),
            text("]"),
        ]));
        let result = print_doc(&doc);
        assert_eq!(result, "[FLAT]\n");
    }

    #[test]
    fn test_print_if_break_broken() {
        // Force break with group_break (break_parent = true)
        let doc = group_break(concat(vec![
            text("["),
            line(),
            if_break(text("BROKEN"), text("FLAT")),
            text("]"),
        ]));
        let result = print_doc(&doc);
        assert!(
            result.contains("BROKEN"),
            "Expected BROKEN but got: {}",
            result
        );
    }

    #[test]
    fn test_print_trailing_comment() {
        let doc = concat(vec![text("code"), trailing_comment("# comment")]);
        let result = print_doc(&doc);
        assert_eq!(result, "code # comment\n");
    }

    #[test]
    fn test_print_leading_comment() {
        let doc = concat(vec![leading_comment("# comment", true), text("code")]);
        let result = print_doc(&doc);
        assert_eq!(result, "# comment\ncode\n");
    }

    #[test]
    fn test_print_empty() {
        let doc = empty();
        let result = print_doc(&doc);
        assert_eq!(result, "");
    }

    #[test]
    fn test_print_join() {
        let doc = join(text(", "), vec![text("a"), text("b"), text("c")]);
        let result = print_doc(&doc);
        assert_eq!(result, "a, b, c\n");
    }

    #[test]
    fn test_print_ruby_class() {
        // Correct Doc structure: hardline inside indent
        let doc = concat(vec![
            text("class Foo"),
            indent(concat(vec![
                hardline(),
                text("def initialize"),
                indent(concat(vec![hardline(), text("@value = 1")])),
                hardline(),
                text("end"),
            ])),
            hardline(),
            text("end"),
        ]);
        let result = print_doc(&doc);

        let expected = "\
class Foo
  def initialize
    @value = 1
  end
end
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_print_ruby_array_fits() {
        // Array that fits on one line
        let doc = group(concat(vec![
            text("["),
            softline(),
            join(
                concat(vec![text(","), line()]),
                vec![text("1"), text("2"), text("3")],
            ),
            softline(),
            text("]"),
        ]));
        let result = print_doc(&doc);
        assert_eq!(result, "[1, 2, 3]\n");
    }

    #[test]
    fn test_print_literal_line() {
        // Literal line should not add indentation
        let doc = concat(vec![
            text("<<~HEREDOC"),
            indent(concat(vec![literalline(), text("content"), literalline()])),
            text("HEREDOC"),
        ]);
        let result = print_doc(&doc);
        // literalline should not add indent despite being inside indent()
        assert!(result.contains("\ncontent\n"));
    }

    // Performance regression tests
    // Run with: cargo test --release perf_

    #[test]
    fn perf_deep_nesting() {
        let config = Config::default();

        // Create 20-level deep nesting
        let mut doc = text("deepest");
        for _ in 0..20 {
            doc = indent(concat(vec![hardline(), doc]));
        }

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Verify it produces valid output
        assert!(result.lines().count() >= 20);
    }

    #[test]
    fn perf_many_hardlines() {
        let config = Config::default();

        // Create 500 lines
        let mut lines: Vec<Doc> = Vec::new();
        for i in 0..500 {
            if i > 0 {
                lines.push(hardline());
            }
            lines.push(text("line"));
        }
        let doc = indent(concat(lines));

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        assert_eq!(result.lines().count(), 500);
    }

    #[test]
    fn perf_nested_groups() {
        let config = Config::default();

        // Create nested group structure
        let inner = group(concat(vec![
            text("["),
            softline(),
            join(
                concat(vec![text(","), line()]),
                vec![text("1"), text("2"), text("3")],
            ),
            softline(),
            text("]"),
        ]));

        let doc = group(concat(vec![
            text("{"),
            softline(),
            text("a: "),
            inner.clone(),
            text(","),
            line(),
            text("b: "),
            inner.clone(),
            text(","),
            line(),
            text("c: "),
            inner,
            softline(),
            text("}"),
        ]));

        let mut printer = Printer::new(&config);
        let result = printer.print(&doc);

        // Should produce valid output
        assert!(result.contains("[1, 2, 3]") || result.contains("[\n"));
    }
}

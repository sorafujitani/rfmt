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
pub struct Printer {
    /// Configuration for formatting
    config: Config,
    /// Output buffer
    output: String,
    /// Current column position (0-indexed)
    pos: usize,
    /// Cached indent strings by width
    indent_cache: Vec<String>,
}

impl Printer {
    /// Creates a new printer with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            output: String::new(),
            pos: 0,
            indent_cache: Vec::new(),
        }
    }

    /// Prints a Doc to a string.
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
    fn process_command<'a>(&mut self, cmd: PrintCommand<'a>, commands: &mut Vec<PrintCommand<'a>>) {
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
                let remaining = self
                    .config
                    .formatting
                    .line_length
                    .saturating_sub(self.pos);
                let fits = !*break_parent && self.fits(contents, cmd.indent, remaining, cmd.mode);

                let mode = if fits { Mode::Flat } else { Mode::Break };

                commands.push(PrintCommand {
                    indent: cmd.indent,
                    mode,
                    doc: contents,
                });
            }

            Doc::Line { soft, hard, literal } => {
                match (cmd.mode, *hard) {
                    // Hard line: always break
                    (_, true) => {
                        self.output.push('\n');
                        if !*literal {
                            let indent_str = self.make_indent(cmd.indent);
                            self.output.push_str(&indent_str);
                            self.pos = cmd.indent;
                        } else {
                            self.pos = 0;
                        }
                    }
                    // Flat mode + soft line: nothing
                    (Mode::Flat, false) if *soft => {}
                    // Flat mode + regular line: space
                    (Mode::Flat, false) => {
                        self.output.push(' ');
                        self.pos += 1;
                    }
                    // Break mode: newline + indent
                    (Mode::Break, false) => {
                        self.output.push('\n');
                        let indent_str = self.make_indent(cmd.indent);
                        self.output.push_str(&indent_str);
                        self.pos = cmd.indent;
                    }
                }
            }

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
                    let indent_str = self.make_indent(cmd.indent);
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
    /// This is a simplified fit check. It measures the width assuming
    /// flat mode and checks if it exceeds the remaining space.
    fn fits(&self, doc: &Doc, indent: usize, remaining: usize, mode: Mode) -> bool {
        let mut width = 0;
        let mut stack: Vec<(&Doc, usize, Mode)> = vec![(doc, indent, mode)];

        while let Some((doc, indent, mode)) = stack.pop() {
            // Early exit if we've exceeded
            if width > remaining {
                return false;
            }

            match doc {
                Doc::Text(s) => {
                    width += s.chars().count();
                }

                Doc::Concat(docs) => {
                    for d in docs.iter().rev() {
                        stack.push((d, indent, mode));
                    }
                }

                Doc::Group { contents, .. } => {
                    // In fit check, assume group stays flat
                    stack.push((contents, indent, Mode::Flat));
                }

                Doc::Line { soft, hard, .. } => {
                    if *hard {
                        // Hard line always breaks, doesn't fit on same line
                        return true; // Actually fits but forces break
                    }
                    match mode {
                        Mode::Flat if *soft => {} // soft line in flat: nothing
                        Mode::Flat => width += 1, // regular line in flat: space
                        Mode::Break => return true, // break mode: newline, we're done
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
                    width += 1 + s.chars().count();
                }

                Doc::LeadingComment { text, .. } => {
                    width += text.chars().count();
                }

                Doc::Align { n, contents } => {
                    stack.push((contents, indent + n, mode));
                }

                Doc::LineSuffix(contents) => {
                    stack.push((contents, indent, mode));
                }

                Doc::Fill(docs) => {
                    for (i, d) in docs.iter().rev().enumerate() {
                        if i > 0 {
                            // Add separator width (softline = 0 in flat)
                        }
                        stack.push((d, indent, mode));
                    }
                }
            }
        }

        width <= remaining
    }

    /// Creates an indent string for the given width.
    fn make_indent(&mut self, width: usize) -> String {
        // Check cache first
        if width < self.indent_cache.len() {
            if let Some(cached) = self.indent_cache.get(width) {
                if !cached.is_empty() {
                    return cached.clone();
                }
            }
        }

        let indent_str = match self.config.formatting.indent_style {
            IndentStyle::Spaces => " ".repeat(width),
            IndentStyle::Tabs => {
                let indent_width = self.config.formatting.indent_width;
                if indent_width == 0 {
                    String::new()
                } else {
                    let tabs = width / indent_width;
                    let spaces = width % indent_width;
                    format!("{}{}", "\t".repeat(tabs), " ".repeat(spaces))
                }
            }
        };

        // Cache the result
        while self.indent_cache.len() <= width {
            self.indent_cache.push(String::new());
        }
        self.indent_cache[width] = indent_str.clone();

        indent_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::builders::*;

    fn default_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_print_text() {
        let doc = text("hello");
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_print_concat() {
        let doc = concat(vec![text("a"), text("b"), text("c")]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "abc\n");
    }

    #[test]
    fn test_print_hardline() {
        let doc = concat(vec![text("line1"), hardline(), text("line2")]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
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
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
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
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "class Foo\n  def bar\n    puts 'hello'\n  end\nend\n");
    }

    #[test]
    fn test_print_group_fits() {
        // Short content should stay on one line
        let doc = group(concat(vec![text("short"), line(), text("text")]));
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "short text\n");
    }

    #[test]
    fn test_print_group_breaks() {
        // Create content that doesn't fit on one line
        let long = "a".repeat(80);
        let doc = group(concat(vec![text(&long), line(), text("more")]));
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert!(result.contains('\n'));
        // Should break: long\nmore
        assert!(result.starts_with(&long));
    }

    #[test]
    fn test_print_softline_flat() {
        // Softline disappears in flat mode
        let doc = group(concat(vec![text("["), softline(), text("1"), softline(), text("]")]));
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "[1]\n");
    }

    #[test]
    fn test_print_if_break_flat() {
        let doc = group(concat(vec![
            text("["),
            if_break(text("BROKEN"), text("FLAT")),
            text("]"),
        ]));
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
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
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert!(result.contains("BROKEN"), "Expected BROKEN but got: {}", result);
    }

    #[test]
    fn test_print_trailing_comment() {
        let doc = concat(vec![text("code"), trailing_comment("# comment")]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "code # comment\n");
    }

    #[test]
    fn test_print_leading_comment() {
        let doc = concat(vec![
            leading_comment("# comment", true),
            text("code"),
        ]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "# comment\ncode\n");
    }

    #[test]
    fn test_print_empty() {
        let doc = empty();
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "");
    }

    #[test]
    fn test_print_join() {
        let doc = join(text(", "), vec![text("a"), text("b"), text("c")]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
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
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);

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
            join(concat(vec![text(","), line()]), vec![text("1"), text("2"), text("3")]),
            softline(),
            text("]"),
        ]));
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        assert_eq!(result, "[1, 2, 3]\n");
    }

    #[test]
    fn test_print_literal_line() {
        // Literal line should not add indentation
        let doc = concat(vec![
            text("<<~HEREDOC"),
            indent(concat(vec![
                literalline(),
                text("content"),
                literalline(),
            ])),
            text("HEREDOC"),
        ]);
        let mut printer = Printer::new(default_config());
        let result = printer.print(&doc);
        // literalline should not add indent despite being inside indent()
        assert!(result.contains("\ncontent\n"));
    }
}

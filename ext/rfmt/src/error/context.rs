use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file: Option<PathBuf>,
    pub source: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file: None,
            source: None,
            line: None,
            column: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// エラー発生位置のコードスニペットを生成
    pub fn generate_snippet(&self, context_lines: usize) -> Option<String> {
        let source = self.source.as_ref()?;
        let error_line = self.line?;
        let column = self.column?;

        let lines: Vec<&str> = source.lines().collect();
        let start = error_line.saturating_sub(context_lines + 1);
        let end = (error_line + context_lines).min(lines.len());

        let mut snippet = String::new();

        for line_num in start..end {
            let line = lines.get(line_num)?;
            let is_error_line = line_num == error_line - 1;

            snippet.push_str(&format!("{:4} | {}\n", line_num + 1, line));

            if is_error_line {
                snippet.push_str(&format!(
                    "     | {}^\n",
                    " ".repeat(column.saturating_sub(1))
                ));
            }
        }

        Some(snippet)
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new()
            .with_file(PathBuf::from("test.rb"))
            .with_location(10, 5);

        assert_eq!(context.file, Some(PathBuf::from("test.rb")));
        assert_eq!(context.line, Some(10));
        assert_eq!(context.column, Some(5));
    }

    #[test]
    fn test_generate_snippet() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        let context = ErrorContext::new()
            .with_source(source.to_string())
            .with_location(3, 6);

        let snippet = context.generate_snippet(1).unwrap();

        assert!(snippet.contains("   2 | line 2"));
        assert!(snippet.contains("   3 | line 3"));
        assert!(snippet.contains("   4 | line 4"));
        assert!(snippet.contains("     |      ^")); // Column 6
    }

    #[test]
    fn test_generate_snippet_at_file_start() {
        let source = "line 1\nline 2\nline 3\n";
        let context = ErrorContext::new()
            .with_source(source.to_string())
            .with_location(1, 3);

        let snippet = context.generate_snippet(2).unwrap();

        assert!(snippet.contains("   1 | line 1"));
        assert!(snippet.contains("     |   ^")); // Column 3
    }

    #[test]
    fn test_metadata() {
        let mut context = ErrorContext::new();
        context.add_metadata("phase", "parsing");
        context.add_metadata("node_type", "class");

        assert_eq!(context.metadata.get("phase"), Some(&"parsing".to_string()));
        assert_eq!(context.metadata.get("node_type"), Some(&"class".to_string()));
    }
}

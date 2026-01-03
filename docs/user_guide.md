# rfmt User Guide

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Configuration](#configuration)
- [Command Line Interface](#command-line-interface)
- [Ruby API](#ruby-api)
- [Editor Integration](#editor-integration)
- [Error Handling](#error-handling)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

## Installation

### Requirements

- Ruby 3.0 or higher
- Rust 1.70 or higher (for building from source)

### Install from RubyGems

```bash
gem install rfmt
```

### Install from Source

```bash
git clone https://github.com/fs0414/rfmt.git
cd rfmt
bundle install
bundle exec rake compile
```

### Verify Installation

```bash
rfmt version
```

## Basic Usage

### Format a Single File

```bash
rfmt lib/my_file.rb
```

This will format the file in place. To preview changes without modifying the file:

```bash
rfmt --check lib/my_file.rb
```

### Format Multiple Files

```bash
rfmt lib/**/*.rb
```

### Format Entire Project

```bash
rfmt .
```

### Check if Files Need Formatting

Use the `--check` flag to verify formatting without making changes:

```bash
rfmt check lib/**/*.rb
```

This is useful in CI/CD pipelines. It exits with a non-zero status if any files need formatting.

### Format from Standard Input

```bash
echo "class Foo;def bar;42;end;end" | rfmt -
```

## Configuration

Create a `.rfmt.yml` file in your project root to customize formatting behavior:

```yaml
version: "1.0"

formatting:
  # Maximum line length
  line_length: 100

  # Indentation width (spaces)
  indent_width: 2

  # Indentation style: "spaces" or "tabs"
  indent_style: "spaces"

  # Quote style: "double" or "single"
  quote_style: "double"

# Files to include (glob patterns)
include:
  - "**/*.rb"
  - "**/*.rake"
  - "**/Rakefile"
  - "**/Gemfile"

# Files to exclude (glob patterns)
exclude:
  - "vendor/**/*"
  - "tmp/**/*"
  - "node_modules/**/*"
  - "db/schema.rb"
```

### Configuration Precedence

rfmt looks for configuration in the following order:

1. `.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, or `rfmt.yaml` in the current directory
2. Same files in parent directories (walking up the tree)
3. Same files in home directory (user-level configuration)
4. Default configuration

### Configuration Options

#### `formatting.line_length`

**Type:** Integer
**Default:** 100
**Description:** Maximum line length before wrapping

```yaml
formatting:
  line_length: 120  # Longer lines allowed
```

#### `formatting.indent_width`

**Type:** Integer
**Default:** 2
**Description:** Number of spaces (or tabs) per indentation level

```yaml
formatting:
  indent_width: 4  # 4 spaces per level
```

#### `formatting.indent_style`

**Type:** String (`"spaces"` or `"tabs"`)
**Default:** `"spaces"`
**Description:** Use spaces or tabs for indentation

```yaml
formatting:
  indent_style: "tabs"
```

#### `formatting.quote_style`

**Type:** String (`"double"` or `"single"`)
**Default:** `"double"`
**Description:** Preferred quote style for strings

```yaml
formatting:
  quote_style: "single"  # Use 'single quotes'
```

## Command Line Interface

### Global Options

These options are available for all commands:

- `--config PATH`: Path to custom configuration file
- `--verbose` or `-v`: Enable verbose output and debug logging

### Commands

#### `rfmt [FILES...]` (default)

Format Ruby files. This is the default command.

**Options:**
- `--check`: Check if files need formatting without modifying them
- `--config PATH`: Path to configuration file
- `--diff`: Show diff of changes
- `--verbose`: Enable verbose output

**Examples:**

```bash
# Format and modify files
rfmt lib/user.rb lib/post.rb

# Check formatting (CI/CD)
rfmt --check lib/**/*.rb

# Show diff without modifying
rfmt --diff lib/user.rb
```

#### `rfmt check [FILES...]`

Check if files need formatting (alias for `rfmt --check`).

```bash
rfmt check .
```

#### `rfmt version`

Display version information.

```bash
rfmt version
```

### Exit Codes

- `0`: Success (all files formatted or no changes needed)
- `1`: Error occurred
- `2`: Files need formatting (when using `--check`)

## Ruby API

### Basic Formatting

```ruby
require 'rfmt'

# Input (unformatted code)
source = <<~RUBY
  class User
  def initialize(name)
  @name=name
  end
  end
RUBY

formatted = Rfmt.format(source)
puts formatted

# Output (formatted code):
# class User
#   def initialize(name)
#     @name=name
#   end
# end
```

### Format with Configuration

```ruby
require 'rfmt'

config = {
  formatting: {
    indent_width: 4,
    quote_style: 'single'
  }
}

formatted = Rfmt.format(source, config: config)
```

### Parse and Inspect AST

```ruby
require 'rfmt'

source = "class Foo\nend"

# Parse to JSON AST
ast_json = Rfmt::PrismBridge.parse(source)
ast = JSON.parse(ast_json)

# Inspect structure
puts ast['ast']['node_type']  # => "program_node"
puts ast['comments'].length   # => 0
```

### Error Handling

```ruby
require 'rfmt'

begin
  result = Rfmt.format(invalid_source)
rescue Rfmt::ParseError => e
  puts "Parse error: #{e.message}"
  # Error includes:
  # - Error code (e.g., E001)
  # - Line and column numbers
  # - Code snippet showing the error
rescue Rfmt::Error => e
  puts "Formatting error: #{e.message}"
end
```

## Error Handling

rfmt provides detailed error messages to help you fix issues quickly.

### Error Codes

All errors include an error code and help URL:

| Code | Type | Description |
|------|------|-------------|
| E001 | ParseError | Ruby syntax error in source code |
| E002 | ConfigError | Invalid configuration file |
| E003 | IoError | File read/write error |
| E004 | FormattingError | Error during formatting process |
| E005 | RuleError | Formatting rule application failed |
| E006 | UnsupportedFeature | Feature not yet supported |
| E007 | PrismError | Prism parser integration error |
| E008 | FormatError | General formatting error |
| E999 | InternalError | Internal bug (please report) |

### Error Format

```
[Rfmt::ParseError] Parse error in example.rb:5:10
Expected closing 'end' for class definition

Code:
   3 | class User
   4 |   def initialize
   5 |     @name = name
     |          ^
   6 | end

Help: https://rfmt.dev/errors/E001
```

### Common Errors

#### E001: Parse Error

**Cause:** Ruby syntax error in your code

**Solution:** Fix the syntax error before formatting

```ruby
# Bad
class User
  def initialize
    @name = name
  # Missing 'end' for method
end

# Good
class User
  def initialize
    @name = name
  end
end
```

#### E002: Configuration Error

**Cause:** Invalid `.rfmt.yml` configuration file

**Solution:** Check your configuration against the schema

```yaml
# Bad
formatting:
  line_length: "100"  # Should be integer, not string

# Good
formatting:
  line_length: 100
```

#### E006: Unsupported Feature

**Cause:** Code uses a Ruby feature not yet supported by rfmt

**Solution:** Check the [roadmap](https://github.com/fs0414/rfmt/blob/main/ROADMAP.md) or file an issue

## Troubleshooting

### rfmt is slow on large files

**Solution:** rfmt is designed to be fast, but very large files (>10,000 lines) may take longer. Consider:

1. Breaking up large files into smaller modules
2. Using `--config` to disable expensive checks
3. Running rfmt in parallel on multiple files

### Comments are being moved

**Issue:** rfmt preserves all comments but may reposition them for consistency

**Solution:** This is expected behavior. rfmt maintains comment positions relative to code structure.

### rfmt changed my code's behavior

**Issue:** Formatting should never change behavior

**Solution:** This is a bug! Please file an issue with:
- Original code
- Formatted code
- Expected behavior
- Actual behavior

### CI/CD pipeline failing with rfmt

**Common causes:**

1. **Different rfmt versions:** Pin the version in your Gemfile
   ```ruby
   gem 'rfmt', '~> 0.1.0'
   ```

2. **Configuration not found:** Ensure `.rfmt.yml` is committed to git

3. **Files need formatting:** Run `rfmt .` locally first

### Getting Debug Information

If you encounter issues, you can enable debug logging to see detailed information:

**Using the --verbose flag:**
```bash
rfmt file.rb --verbose
# or
rfmt file.rb -v
```

**Using environment variables:**
```bash
# Enable debug logging with DEBUG
DEBUG=1 rfmt file.rb

# Enable rfmt-specific debug logging
RFMT_DEBUG=1 rfmt file.rb

# Control log level directly
RFMT_LOG=debug rfmt file.rb
```

Debug logging will show:
- Initialization messages
- Configuration file discovery
- File processing details
- Internal Rust extension operations

## FAQ

### Does rfmt preserve comments?

**Yes!** rfmt preserves all comments in their original positions. Line comments, block comments, and documentation comments are all maintained.

### Is formatting idempotent?

**Yes!** Running rfmt multiple times on the same file produces identical results. This is guaranteed by our test suite.

### Can I disable specific rules?

Not yet. rfmt follows a consistent style without configuration. This is by design to reduce bikeshedding. If you have a strong use case, please file an issue.

### How does rfmt compare to RuboCop?

rfmt is a **formatter**, while RuboCop is a **linter**:

| Feature | rfmt | RuboCop |
|---------|------|---------|
| Format code | ✅ | ✅ (with autocorrect) |
| Style enforcement | ✅ | ✅ |
| Code smell detection | ❌ | ✅ |
| Bug detection | ❌ | ✅ |
| Performance | Very fast | Moderate |
| Configuration | Minimal | Extensive |

**Recommendation:** Use both! Run rfmt for consistent formatting, and RuboCop for code quality checks.

### Does rfmt work with Rails?

**Yes!** rfmt works with any Ruby code, including Rails applications. It correctly handles:

- Models, Controllers, Views
- Migrations
- Routes (`config/routes.rb`)
- Rake tasks
- Initializers

### Can I use rfmt with pre-commit hooks?

**Yes!** Example `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: rfmt
        name: rfmt
        entry: bundle exec rfmt
        language: system
        files: \.rb$
```

Or with [Lefthook](https://github.com/evilmartians/lefthook):

```yaml
# lefthook.yml
pre-commit:
  commands:
    rfmt:
      glob: "*.rb"
      run: bundle exec rfmt {staged_files}
```

### What Ruby versions are supported?

rfmt supports **Ruby 3.0 and higher**. We test against:

- Ruby 3.0
- Ruby 3.1
- Ruby 3.2
- Ruby 3.3

### How can I contribute?

See our [Contributing Guide](../CONTRIBUTING.md) for details on:

- Setting up development environment
- Running tests
- Submitting pull requests
- Code style guidelines

### Where can I get help?

- 📖 Documentation: https://rfmt.dev/docs
- 🐛 Issues: https://github.com/fs0414/rfmt/issues
- 💬 Discussions: https://github.com/fs0414/rfmt/discussions
- 📧 Email: fujitanisora0414@gmail.com

## Next Steps

- Read the [API Documentation](api_documentation.md)
- Learn about [Contributing](../CONTRIBUTING.md)
- Check the [Roadmap](../ROADMAP.md)
- Review [Error Reference](error_reference.md)

---

**Version:** 0.2.4
**Last Updated:** 2025-11-25
**License:** MIT

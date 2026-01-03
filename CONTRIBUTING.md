# Contributing to rfmt

Thank you for your interest in contributing to rfmt! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)

## Code of Conduct

This project adheres to the Contributor Covenant [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to fujitanisora0414@gmail.com.

## Getting Started

### Prerequisites

- Ruby 3.0 or higher
- Rust 1.70 or higher
- Git
- Bundler

### Finding an Issue to Work On

1. Browse the [issue tracker](https://github.com/fs0414/rfmt/issues)
2. Look for issues labeled `good first issue` or `help wanted`
3. Comment on the issue to let others know you're working on it
4. For larger changes, discuss your approach in the issue before starting

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/rfmt.git
cd rfmt

# Add the upstream repository
git remote add upstream https://github.com/fs0414/rfmt.git
```

### 2. Install Dependencies

```bash
# Install Ruby dependencies
bundle install

# Compile the Rust extension
bundle exec rake compile
```

### 3. Verify Setup

```bash
# Run tests
bundle exec rspec

# All tests should pass
```

## Project Structure

```
rfmt/
├── lib/                    # Ruby code
│   ├── rfmt.rb            # Main entry point
│   └── rfmt/
│       └── prism_bridge.rb # Ruby-Rust bridge
├── ext/rfmt/              # Rust extension
│   ├── src/
│   │   ├── lib.rs         # FFI interface
│   │   ├── error/         # Error handling
│   │   ├── logging/       # Logging system
│   │   ├── debug/         # Debug utilities
│   │   ├── parser/        # AST parsing
│   │   ├── formatter/     # Formatting engine
│   │   └── emitter/       # Code emission
│   └── Cargo.toml         # Rust dependencies
├── spec/                  # RSpec tests
│   ├── fixtures/          # Test data
│   └── *_spec.rb         # Test files
├── docs/                  # Documentation
└── docspriv/             # Private documentation
```

### Key Components

#### Ruby Side (`lib/`)
- **Rfmt module**: Main interface for users
- **PrismBridge**: Bridges Ruby's Prism parser with Rust

#### Rust Side (`ext/rfmt/src/`)
- **error/**: Error types and handling (E001-E999)
- **logging/**: Structured logging system
- **debug/**: Debug context and macros
- **parser/**: AST parsing from Prism JSON
- **formatter/**: Formatting rules engine
- **emitter/**: Formatted code output

## Development Workflow

### 1. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create a feature branch
git checkout -b feature/my-feature
```

### 2. Make Changes

Follow the [code style guidelines](#code-style) when making changes.

### 3. Test Your Changes

```bash
# Run all tests
bundle exec rspec

# Run specific test file
bundle exec rspec spec/my_spec.rb

# Run Rust tests
cd ext/rfmt && cargo test
```

### 4. Commit Changes

Write clear, concise commit messages following [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format:
# <type>(<scope>): <subject>
#
# Examples:
git commit -m "feat(parser): add support for pattern matching"
git commit -m "fix(emitter): preserve comments in method definitions"
git commit -m "docs: update user guide with new examples"
git commit -m "test: add tests for error handling"
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

## Testing

### Ruby Tests (RSpec)

```bash
# Run all tests
bundle exec rspec

# Run with documentation format
bundle exec rspec --format documentation

# Run specific file
bundle exec rspec spec/error_handling_spec.rb

# Run specific example
bundle exec rspec spec/error_handling_spec.rb:10
```

### Rust Tests (Cargo)

```bash
cd ext/rfmt

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_error_handler_creation
```

### Writing Tests

#### Ruby Tests Example

```ruby
# spec/my_feature_spec.rb
require 'spec_helper'

RSpec.describe 'My Feature' do
  it 'does something useful' do
    source = "class Foo\nend"
    result = Rfmt.format(source)

    expect(result).to include('class Foo')
    expect(result).to end_with("end\n")
  end
end
```

#### Rust Tests Example

```rust
// ext/rfmt/src/my_module.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function("input");
        assert_eq!(result, "expected");
    }
}
```

### Test Coverage

We aim for:
- **90%+ test coverage** for new code
- **100% coverage** for error handling paths
- Both positive and negative test cases

## Code Style

### Ruby Code Style

Follow standard Ruby conventions:

```ruby
# Good
def format_code(source)
  return '' if source.empty?

  result = process(source)
  result.strip
end

# Use 2-space indentation
# Use snake_case for variables and methods
# Use SCREAMING_SNAKE_CASE for constants
# Keep lines under 100 characters
```

Check with RuboCop:

```bash
bundle exec rubocop
```

### Rust Code Style

Follow standard Rust conventions:

```rust
// Good
pub fn format_code(source: &str) -> Result<String> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let result = process(source)?;
    Ok(result.trim().to_string())
}

// Use 4-space indentation
// Use snake_case for variables and functions
// Use PascalCase for types
// Keep lines under 100 characters
```

Format with:

```bash
cd ext/rfmt
cargo fmt
```

Lint with:

```bash
cd ext/rfmt
cargo clippy
```

### Documentation

#### Ruby Documentation (YARD)

```ruby
# Formats Ruby source code
#
# @param source [String] the Ruby code to format
# @param config [Hash] optional configuration overrides
# @return [String] formatted Ruby code
# @raise [Rfmt::ParseError] if source has syntax errors
# @example
#   Rfmt.format("class Foo;end")
#   #=> "class Foo\nend\n"
def self.format(source, config: {})
  # ...
end
```

#### Rust Documentation

```rust
/// Formats Ruby source code according to configuration.
///
/// # Arguments
///
/// * `source` - The Ruby source code to format
/// * `config` - Configuration options
///
/// # Returns
///
/// Returns formatted Ruby source code or an error
///
/// # Errors
///
/// This function will return an error if:
/// - The source code has syntax errors
/// - The configuration is invalid
///
/// # Examples
///
/// ```
/// let formatted = format_ruby("class Foo\nend", &config)?;
/// assert_eq!(formatted, "class Foo\nend\n");
/// ```
pub fn format_ruby(source: &str, config: &Config) -> Result<String> {
    // ...
}
```

## Submitting Changes

### Pull Request Process

1. **Update your branch**

```bash
git fetch upstream
git rebase upstream/main
```

2. **Run all checks**

```bash
# Ruby tests
bundle exec rspec

# Rust tests
cd ext/rfmt && cargo test

# Format checks
cargo fmt --check
cargo clippy

# Build check
bundle exec rake compile
```

3. **Push to your fork**

```bash
git push origin feature/my-feature
```

4. **Create Pull Request**

Go to GitHub and create a pull request from your fork. Include:

- **Clear title**: Summarize the change
- **Description**: Explain what and why
- **Related issues**: Reference any related issues
- **Test results**: Confirm all tests pass
- **Breaking changes**: Note any breaking changes

### Pull Request Template

```markdown
## Description
Brief description of what this PR does.

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2

## Testing
- [ ] All existing tests pass
- [ ] Added new tests for new functionality
- [ ] Tested manually with [describe scenario]

## Related Issues
Fixes #123
Related to #456

## Breaking Changes
None / Describe breaking changes
```

### Code Review

- Respond to feedback promptly
- Make requested changes in new commits
- Don't force-push after review starts
- Ask for clarification if feedback is unclear

## Reporting Issues

### Bug Reports

Include:

1. **rfmt version**: `rfmt --version`
2. **Ruby version**: `ruby -v`
3. **Platform**: OS and architecture
4. **Expected behavior**: What should happen
5. **Actual behavior**: What actually happened
6. **Reproduction**: Minimal code to reproduce
7. **Error messages**: Full error output

**Example:**

```markdown
## Bug Report

**rfmt version**: 0.1.0
**Ruby version**: 3.3.0
**Platform**: macOS 14.0 (arm64)

### Expected Behavior
Comments should be preserved after formatting

### Actual Behavior
Comments are removed from the output

### Reproduction
```ruby
# This is a comment
class Foo
end
```

### Error Output
No errors, but comment is missing in result
```

## Feature Requests

Include:

1. **Use case**: Why is this feature needed?
2. **Proposed solution**: How should it work?
3. **Alternatives**: Other approaches considered?
4. **Examples**: Code examples if applicable

## Development Tips

### Debugging

#### Enable Rust Logging

```bash
RUST_LOG=debug bundle exec rspec
```

#### Enable Rust Backtrace

```bash
RUST_BACKTRACE=1 bundle exec rspec
```

#### Interactive Debugging (Ruby)

```ruby
require 'pry'
require 'rfmt'

binding.pry
result = Rfmt.format(source)
```

### Common Tasks

#### Rebuild After Rust Changes

```bash
bundle exec rake clean
bundle exec rake compile
```

#### Update Dependencies

```bash
# Ruby
bundle update

# Rust
cd ext/rfmt
cargo update
```

#### Generate Documentation

```bash
# Ruby docs
bundle exec yard doc

# Rust docs
cd ext/rfmt
cargo doc --open
```

## Questions?

- 📖 Read the [User Guide](docs/user_guide.md)
- 💬 Ask in [Discussions](https://github.com/fs0414/rfmt/discussions)
- 📧 Email: fujitanisora0414@gmail.com

## License

By contributing to rfmt, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to rfmt! 🎉

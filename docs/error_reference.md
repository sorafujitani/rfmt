# kenshin Error Reference

Complete reference for all error codes and their solutions.

## Error Code Format

All kenshin errors follow this format:

```
[Kenshin::ErrorType] Error message
Additional context and details

Help: https://kenshin.dev/errors/EXXX
```

## Error Codes

### E001: ParseError

**Type:** `Kenshin::ParseError`

**Description:** Ruby syntax error in the source code being formatted.

**Common Causes:**
- Missing `end` keyword
- Unclosed string or parenthesis
- Invalid Ruby syntax
- Malformed block

**Example Error:**

```
[Kenshin::ParseError] Parse error in app/models/user.rb:15:10
Expected closing 'end' for class definition

Code:
  13 | class User < ApplicationRecord
  14 |   def initialize(name)
  15 |     @name = name
     |          ^
  16 | # Missing 'end' for method
  17 | end

Help: https://kenshin.dev/errors/E001
```

**Solutions:**

1. **Fix the syntax error before formatting:**
   ```ruby
   # Before (invalid)
   class User
     def initialize
       @name = name
     # Missing 'end'
   end

   # After (valid)
   class User
     def initialize
       @name = name
     end
   end
   ```

2. **Check for unbalanced blocks:**
   ```ruby
   # Before (invalid)
   users.each do |user|
     puts user.name
   # Missing 'end' for block

   # After (valid)
   users.each do |user|
     puts user.name
   end
   ```

3. **Verify string delimiters:**
   ```ruby
   # Before (invalid)
   message = "Hello, world

   # After (valid)
   message = "Hello, world"
   ```

**Related Issues:**
- [#42](https://github.com/sorafujitani/rfmt/issues/42): Improved error messages for parse errors
- [#15](https://github.com/sorafujitani/rfmt/issues/15): Support for heredoc syntax

---

### E002: ConfigError

**Type:** `Kenshin::ConfigError`

**Description:** Invalid or malformed configuration file (`.kenshin.yml`).

**Common Causes:**
- Invalid YAML syntax
- Unknown configuration keys
- Invalid values for configuration options
- Missing required fields

**Example Error:**

```
[Kenshin::ConfigError] Configuration error: Invalid value for 'indent_width'
File: .kenshin.yml

Suggestion: Use a positive integer value (e.g., 2, 4)

Help: https://kenshin.dev/errors/E002
```

**Solutions:**

1. **Validate YAML syntax:**
   ```yaml
   # Before (invalid)
   formatting
     indent_width: 2

   # After (valid)
   formatting:
     indent_width: 2
   ```

2. **Use correct data types:**
   ```yaml
   # Before (invalid)
   formatting:
     indent_width: "2"  # String instead of integer
     line_length: two   # Invalid value

   # After (valid)
   formatting:
     indent_width: 2
     line_length: 100
   ```

3. **Check configuration keys:**
   ```yaml
   # Before (invalid)
   formatting:
     indentation: 2      # Wrong key name

   # After (valid)
   formatting:
     indent_width: 2
   ```

4. **Verify enum values:**
   ```yaml
   # Before (invalid)
   formatting:
     indent_style: "space"  # Should be "spaces"

   # After (valid)
   formatting:
     indent_style: "spaces"
   ```

**Valid Configuration Schema:**

```yaml
version: "1.0"

formatting:
  line_length: 100        # Integer (1-500)
  indent_width: 2         # Integer (1-8)
  indent_style: "spaces"  # "spaces" or "tabs"
  quote_style: "double"   # "double" or "single"

include:                  # Array of glob patterns
  - "**/*.rb"

exclude:                  # Array of glob patterns
  - "vendor/**/*"
```

**Related Issues:**
- [#23](https://github.com/sorafujitani/rfmt/issues/23): Better error messages for config errors

---

### E003: IoError

**Type:** `Kenshin::IOError`

**Description:** File system operation failed (read, write, or access).

**Common Causes:**
- File doesn't exist
- Insufficient permissions
- File is locked by another process
- Disk is full
- Network drive unavailable

**Example Error:**

```
[Kenshin::IOError] IO error for file app/models/user.rb: Permission denied

Help: https://kenshin.dev/errors/E003
```

**Solutions:**

1. **Check file exists:**
   ```bash
   ls -la app/models/user.rb
   ```

2. **Verify permissions:**
   ```bash
   # Read permission
   chmod u+r file.rb

   # Write permission (for in-place formatting)
   chmod u+w file.rb
   ```

3. **Check disk space:**
   ```bash
   df -h .
   ```

4. **Close file in other programs:**
   - Close editors that might have the file open
   - Check for background processes using `lsof`

5. **Use sudo (if appropriate):**
   ```bash
   sudo kenshin format system_file.rb
   ```

**Related Issues:**
- [#31](https://github.com/sorafujitani/rfmt/issues/31): Better error recovery for locked files

---

### E004: FormattingError

**Type:** `Kenshin::FormattingError`

**Description:** Error occurred during the formatting process.

**Common Causes:**
- Internal formatter bug
- Unsupported Ruby syntax edge case
- Corrupted AST
- Memory exhaustion on very large files

**Example Error:**

```
[Kenshin::FormattingError] Formatting error: Failed to emit node
Node type: def_node
Location: 42:15

Help: https://kenshin.dev/errors/E004
```

**Solutions:**

1. **Try formatting a simpler version:**
   - Comment out complex code
   - Simplify nested structures
   - Format in smaller chunks

2. **Update kenshin:**
   ```bash
   gem update kenshin
   ```

3. **Report the issue:**
   This is likely a bug. Please report it with:
   - Your Ruby code (or minimal reproduction)
   - kenshin version (`kenshin --version`)
   - Ruby version (`ruby -v`)
   - Error message

4. **Workaround with partial formatting:**
   ```bash
   # Format individual methods instead of entire file
   kenshin format app/models/user.rb:10-50
   ```

**Related Issues:**
- [#55](https://github.com/sorafujitani/rfmt/issues/55): Handling of complex nested blocks

---

### E005: RuleError

**Type:** `Kenshin::RuleError`

**Description:** A formatting rule failed to apply.

**Common Causes:**
- Conflicting formatting rules
- Rule precondition not met
- Bug in rule implementation

**Example Error:**

```
[Kenshin::RuleError] Rule application error: Rule 'IndentationRule' failed
Cannot determine indentation level for orphaned node

Help: https://kenshin.dev/errors/E005
```

**Solutions:**

1. **Check for syntax errors first:**
   Ensure your code parses correctly with `ruby -c file.rb`

2. **Simplify the code structure:**
   Complex nested structures might confuse the formatter

3. **Update kenshin:**
   ```bash
   gem update kenshin
   ```

4. **Report the issue:**
   This is likely a bug in the formatting rules

**Related Issues:**
- [#67](https://github.com/sorafujitani/rfmt/issues/67): Rule conflict resolution

---

### E006: UnsupportedFeature

**Type:** `Kenshin::UnsupportedFeature`

**Description:** Code uses a Ruby feature not yet supported by kenshin.

**Common Causes:**
- Experimental Ruby syntax
- Ruby 3.4+ features (if using older kenshin)
- Edge cases in language features

**Example Error:**

```
[Kenshin::UnsupportedFeature] Unsupported feature: Pattern matching with pinning operator

This feature is planned for a future release.
Please track: https://github.com/sorafujitani/rfmt/issues/89

Help: https://kenshin.dev/errors/E006
```

**Solutions:**

1. **Check roadmap:**
   See if the feature is planned: [ROADMAP.md](../ROADMAP.md)

2. **Use alternative syntax:**
   If possible, rewrite using supported features

3. **Skip formatting for that section:**
   ```ruby
   # kenshin:disable
   case value
   in ^expected_value
     puts "matched"
   end
   # kenshin:enable
   ```

4. **Request feature:**
   File an issue with:
   - Code example using the feature
   - Use case description
   - Ruby version where it's valid

**Currently Unsupported Features:**
- Numbered block parameters (`_1`, `_2`)
- Some Ruby 3.3+ syntax features
- Complex pattern matching edge cases

**Related Issues:**
- [#89](https://github.com/sorafujitani/rfmt/issues/89): Pattern matching support
- [#102](https://github.com/sorafujitani/rfmt/issues/102): Numbered parameters

---

### E007: PrismError

**Type:** `Kenshin::PrismError`

**Description:** Error in the embedded prism parser integration. Parsing happens inside the Rust extension via the statically linked ruby-prism crate; the prism gem is not used at runtime.

**Common Causes:**
- Internal parser error
- Unexpected AST structure

**Example Error:**

```
[Kenshin::PrismError] Prism integration error: unexpected node structure

Help: https://kenshin.dev/errors/E007
```

**Solutions:**

1. **Update kenshin:**
   ```bash
   gem update kenshin
   ```

2. **Rebuild the extension (source installs):**
   ```bash
   bundle exec rake clean
   bundle exec rake compile
   ```

3. **Report the issue:**
   This is an internal error. Please report with:
   - kenshin version
   - Code that triggers the error

**Related Issues:**
- [#118](https://github.com/sorafujitani/rfmt/issues/118): Prism 1.0 compatibility

---

### E008: FormatError

**Type:** `Kenshin::FormatError`

**Description:** Generic formatting error (catch-all).

**Common Causes:**
- Various formatting failures
- Edge cases not covered by other errors

**Example Error:**

```
[Kenshin::FormatError] Format error: Buffer overflow during emission

Help: https://kenshin.dev/errors/E008
```

**Solutions:**

1. **Simplify the code:**
   Break down complex structures

2. **Check file size:**
   ```bash
   wc -l file.rb  # Very large files might cause issues
   ```

3. **Update kenshin:**
   ```bash
   gem update kenshin
   ```

4. **Report the issue:**
   Include full error message and code sample

---

### E999: InternalError

**Type:** `Kenshin::InternalError`

**Description:** Internal bug in kenshin. This should never happen!

**Common Causes:**
- Unhandled edge case
- Bug in kenshin code
- Memory corruption
- Platform-specific issue

**Example Error:**

```
[Kenshin::InternalError] Internal error: Unexpected null pointer in AST traversal

Backtrace:
  at /path/to/kenshin/src/emitter.rs:123
  at /path/to/kenshin/src/formatter.rs:456

Please report this as a bug at: https://github.com/sorafujitani/rfmt/issues

Help: https://kenshin.dev/errors/E999
```

**Solutions:**

1. **Report immediately:**
   This is a bug! Please create an issue with:
   - Full error message including backtrace
   - Code that triggers the error (or minimal reproduction)
   - kenshin version (`kenshin --version`)
   - Ruby version (`ruby -v`)
   - Platform (OS and architecture)

2. **Workaround:**
   - Skip the problematic file temporarily
   - Format in smaller chunks
   - Use an older version if this is a regression

3. **Collect debug info:**
   ```bash
   RUST_BACKTRACE=1 kenshin format file.rb 2> error.log
   ```

**Related Issues:**
- [#new](https://github.com/sorafujitani/rfmt/issues/new): Report new bug

---

## Debugging Tips

### Enable Verbose Output

```bash
kenshin format --verbose file.rb
```

### Check Rust Backtrace

```bash
RUST_BACKTRACE=1 kenshin format file.rb
```

### Enable Debug Logging

```ruby
# Set log level before requiring kenshin
ENV['KENSHIN_LOG_LEVEL'] = 'debug'
require 'kenshin'
```

### Get Debug Information

```ruby
require 'kenshin'

# Print version and platform info
puts Kenshin.rust_version
```

## Getting Help

If you encounter an error not covered here:

1. **Search existing issues:** https://github.com/sorafujitani/rfmt/issues
2. **Check discussions:** https://github.com/sorafujitani/rfmt/discussions
3. **Create a new issue:** https://github.com/sorafujitani/rfmt/issues/new

When reporting issues, include:

- Error code and full message
- kenshin version (`kenshin --version`)
- Ruby version (`ruby -v`)
- Code sample (minimal reproduction)
- Configuration file (`.kenshin.yml`)
- Platform (OS and architecture)

## Related Documentation

- [User Guide](user_guide.md)
- [API Documentation](api_documentation.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [Troubleshooting](user_guide.md#troubleshooting)

---

**Version:** 0.1.0
**Last Updated:** 2025-11-24
**License:** MIT

# rfmt

<div align="center">

⚡ A fast, opinionated Ruby code formatter written in Rust ⚡

[![Gem Version](https://badge.fury.io/rb/rfmt.svg)](https://badge.fury.io/rb/rfmt)
[![Test Status](https://github.com/fujitanisora/rfmt/workflows/test/badge.svg)](https://github.com/fujitanisora/rfmt/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Installation](#installation) •
[Usage](#usage) •
[Features](#features) •
[Documentation](#documentation) •
[Contributing](#contributing)

</div>

---

## What is rfmt?

**rfmt** is a lightning-fast Ruby code formatter that enforces consistent style across your codebase. It's designed to be:

- **Fast**: Written in Rust for maximum performance
- **Opinionated**: Minimal configuration, consistent results
- **Idempotent**: Running it multiple times produces the same output
- **Comment-preserving**: Your comments stay exactly where they should
- **Production-ready**: Comprehensive error handling and logging

## Features

### ⚡ Performance

rfmt is built with Rust, making it significantly faster than pure-Ruby formatters:

- Formats thousands of lines per second
- Parallel processing support
- Smart caching for unchanged files

### 🎨 Consistent Style

rfmt enforces a consistent code style across your entire project:

- Automatic indentation
- Consistent spacing and alignment
- Quote style normalization
- Method definition formatting

### 🔍 Smart Error Handling

rfmt provides detailed, actionable error messages:

- **Error codes** (E001-E999) for easy troubleshooting
- **Code snippets** showing exactly where errors occur
- **Help URLs** linking to detailed documentation
- **Recovery strategies** to handle partial formatting

### 📊 Comprehensive Logging

Built-in logging system for debugging and monitoring:

- Multiple log levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Structured output for easy parsing
- Performance metrics and timing information
- Debug context for complex operations

### 🧩 Editor Integration

Works with your favorite editor:

- Visual Studio Code
- RubyMine / IntelliJ IDEA
- Vim / Neovim
- Emacs
- Sublime Text

## Installation

### Requirements

- Ruby 3.0 or higher
- Rust 1.70 or higher (for building from source)

### From RubyGems

```bash
gem install rfmt
```

### In Your Gemfile

```ruby
gem 'rfmt'
```

Then run:

```bash
bundle install
```

### From Source

```bash
git clone https://github.com/fujitanisora/rfmt.git
cd rfmt
bundle install
bundle exec rake compile
```

## Usage

### Command Line

Format a single file:

```bash
rfmt format lib/user.rb
```

Format multiple files:

```bash
rfmt format lib/**/*.rb
```

Check if files need formatting (CI/CD):

```bash
rfmt check .
```

### Ruby API

```ruby
require 'rfmt'

source = <<~RUBY
  class User
    def initialize(name)
      @name=name
    end
  end
RUBY

formatted = Rfmt.format(source)
puts formatted
```

**Output:**

```ruby
class User
  def initialize(name)
    @name = name
  end
end
```

### Configuration

#### Initializing Configuration

Create a configuration file with default settings:

```bash
rfmt init
```

This creates a `.rfmt.yml` file with sensible defaults:

```yaml
version: "1.0"

formatting:
  line_length: 100        # Maximum line length (40-500)
  indent_width: 2         # Spaces/tabs per indent (1-8)
  indent_style: "spaces"  # "spaces" or "tabs"
  quote_style: "double"   # "double", "single", or "consistent"

include:
  - "**/*.rb"
  - "**/*.rake"
  - "**/Rakefile"
  - "**/Gemfile"

exclude:
  - "vendor/**/*"
  - "tmp/**/*"
  - "node_modules/**/*"
  - "db/schema.rb"
```

**Options:**

```bash
# Specify custom path
rfmt init --path config/rfmt.yml

# Overwrite existing configuration
rfmt init --force
```

#### Configuration File Discovery

rfmt automatically searches for configuration files in this order:

1. Current directory (`.rfmt.yml` or `.rfmt.yaml`)
2. Parent directories (up to root)
3. User home directory (`~/.rfmt.yml` or `~/.rfmt.yaml`)
4. Default settings (if no file found)

#### Ruby API for Configuration

```ruby
require 'rfmt'

# Generate configuration file
Rfmt::Config.init('.rfmt.yml', force: false)

# Find configuration file
config_path = Rfmt::Config.find
# => "/Users/username/project/.rfmt.yml"

# Check if configuration exists
Rfmt::Config.exists?
# => true

# Load configuration
config = Rfmt::Config.load
# => {"version"=>"1.0", "formatting"=>{"line_length"=>100, ...}, ...}
```

## Error Handling

rfmt provides detailed error messages with actionable solutions:

```
[Rfmt::ParseError] Parse error in app/models/user.rb:15:10
Expected closing 'end' for class definition

Code:
  13 | class User < ApplicationRecord
  14 |   def initialize(name)
  15 |     @name = name
     |          ^
  16 | # Missing 'end' for method
  17 | end

Help: https://rfmt.dev/errors/E001
```

### Error Codes

| Code | Type | Description |
|------|------|-------------|
| E001 | ParseError | Ruby syntax error |
| E002 | ConfigError | Invalid configuration |
| E003 | IoError | File read/write error |
| E004 | FormattingError | Formatting process error |
| E005 | RuleError | Rule application failed |
| E006 | UnsupportedFeature | Feature not yet supported |
| E007 | PrismError | Parser integration error |
| E008 | FormatError | General formatting error |
| E999 | InternalError | Internal bug (please report!) |

See the [Error Reference](docs/error_reference.md) for detailed information.

## Logging

rfmt includes a comprehensive logging system:

```ruby
# Logs are automatically output during initialization
require 'rfmt'
# [INFO] rfmt - Initializing rfmt Rust extension
# [INFO] rfmt - rfmt Rust extension initialized successfully
```

Log levels:
- **ERROR**: Critical errors
- **WARN**: Warnings
- **INFO**: General information (default)
- **DEBUG**: Debug information
- **TRACE**: Detailed trace information

## Examples

### Before Formatting

```ruby
class User<ApplicationRecord
has_many :posts
validates :email,presence: true
def full_name
"#{first_name} #{last_name}"
end
end
```

### After Formatting

```ruby
class User < ApplicationRecord
  has_many :posts
  validates :email, presence: true

  def full_name
    "#{first_name} #{last_name}"
  end
end
```

## Documentation

Comprehensive documentation is available:

- 📖 [User Guide](docs/user_guide.md) - Complete usage guide
- 🌐 [User Guide (日本語)](docs/user_guide.ja.md) - Japanese version
- 🔍 [Error Reference](docs/error_reference.md) - All error codes and solutions
- 🔍 [Error Reference (日本語)](docs/error_reference.ja.md) - Japanese version
- 🤝 [Contributing Guide](CONTRIBUTING.md) - How to contribute
- 📊 [Phase 4 Implementation](docs/phase4_implementation_summary.md) - Recent changes

## Development

After checking out the repo:

```bash
# Install dependencies
bundle install

# Compile Rust extension
bundle exec rake compile

# Run tests
bundle exec rspec

# Run Rust tests
cd ext/rfmt && cargo test
```

### Running Tests

```bash
# All tests
bundle exec rspec

# Specific test file
bundle exec rspec spec/error_handling_spec.rb

# With documentation format
bundle exec rspec --format documentation
```

### Test Results

All 187 tests passing:
- 172 existing tests
- 15 new tests for error handling and logging

## Performance

rfmt is designed for speed:

| File Size | Format Time |
|-----------|-------------|
| 100 lines | < 10ms |
| 1,000 lines | < 50ms |
| 10,000 lines | < 500ms |

*Benchmarks run on M1 MacBook Pro*

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features:

- [ ] Pattern matching support
- [ ] Numbered parameters
- [ ] Additional formatting rules
- [ ] Plugin system
- [ ] Language server protocol (LSP)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

**Quick Start:**

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/my-feature`)
3. Make your changes and add tests
4. Run tests (`bundle exec rspec`)
5. Commit your changes (`git commit -m 'feat: add some feature'`)
6. Push to the branch (`git push origin feature/my-feature`)
7. Open a Pull Request

## Architecture

rfmt is built with a hybrid Ruby-Rust architecture:

```
┌─────────────────┐
│   Ruby Layer    │  ← User API, Prism parser
├─────────────────┤
│  FFI Interface  │  ← Magnus (Ruby-Rust bridge)
├─────────────────┤
│   Rust Layer    │  ← Formatting engine
│                 │    - Parser (AST)
│                 │    - Formatter (Rules)
│                 │    - Emitter (Output)
│                 │    - Error Handler
│                 │    - Logger
└─────────────────┘
```

## Technology Stack

- **Ruby**: 3.0+ (Prism parser, FFI interface)
- **Rust**: 1.70+ (Core formatting engine)
- **Magnus**: Ruby-Rust FFI bridge
- **Prism**: Modern Ruby parser
- **RSpec**: Ruby testing
- **Cargo**: Rust build system

## Comparison with Other Tools

### rfmt vs RuboCop

| Feature | rfmt | RuboCop |
|---------|------|---------|
| **Primary Purpose** | Code formatting | Linting + formatting |
| **Speed** | Very fast (Rust) | Moderate (Ruby) |
| **Configuration** | Minimal | Extensive |
| **Code Quality Checks** | No | Yes |
| **Bug Detection** | No | Yes |

**Recommendation**: Use rfmt for consistent formatting, RuboCop for code quality checks.

### rfmt vs Prettier (Ruby plugin)

| Feature | rfmt | Prettier |
|---------|------|----------|
| **Native Ruby Support** | Yes | Via plugin |
| **Speed** | Very fast | Fast |
| **Ruby-specific Features** | Full support | Limited |
| **Comment Preservation** | Excellent | Good |

## Project Status

rfmt is under active development. Current phase:

- ✅ Phase 1: Foundation (Complete)
- ✅ Phase 2: Core Formatting (Complete)
- ✅ Phase 3: Advanced Features (Complete)
- ✅ Phase 4: Production Quality (Logging & Error Control Complete)
  - ✅ Error Handling System
  - ✅ Logging System
  - ⬜ Documentation (In Progress)
  - ⬜ Security
  - ⬜ Release Process
  - ⬜ Editor Integration

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Code of Conduct

Everyone interacting in the rfmt project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](CODE_OF_CONDUCT.md).

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issues](https://github.com/fujitanisora/rfmt/issues)
- 💬 [Discussions](https://github.com/fujitanisora/rfmt/discussions)
- 📧 Email: fujitanisora0414@gmail.com

## Acknowledgments

- Built with [Prism](https://github.com/ruby/prism) - Modern Ruby parser
- Powered by [Rust](https://www.rust-lang.org/) - Performance and safety
- FFI via [Magnus](https://github.com/matsadler/magnus) - Ruby-Rust bridge

---

<div align="center">

Made with ❤️ by [Fujitani Sora](https://github.com/fs0414)

**⭐ Star us on GitHub — it motivates us a lot!**

</div>

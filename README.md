# rfmt

<div align="center">

A Ruby code formatter written in Rust

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

**rfmt** is a Ruby code formatter that enforces consistent style across your codebase. Key characteristics:

- **Rust implementation**: Provides faster execution than Ruby-based tools
- **Opinionated**: Minimal configuration with consistent output
- **Idempotent**: Running multiple times produces identical results
- **Comment preservation**: Maintains existing comment placement
- **Error handling**: Includes structured error messages and logging

## Features

### Performance

Built with Rust for improved execution speed:

- Benchmark shows 7-58x faster than RuboCop depending on project size (see Performance Benchmarks section)
- Processes 168 files/second in tested Rails project
- Supports parallel processing

### Consistent Style

Enforces code style rules:

- Automatic indentation
- Spacing and alignment normalization
- Quote style standardization
- Method definition formatting

### Error Handling

Provides structured error messages:

- Error codes (E001-E999) for categorization
- Code snippets showing error locations
- Help URLs linking to documentation
- Recovery strategies for handling errors

### Logging

Built-in logging system:

- 5 log levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Structured output format
- Performance metrics and timing data
- Debug context information

## Performance Benchmarks

Performance comparison with RuboCop on a Rails project (111 files, 3,231 lines):

### Benchmark Results (Rails Project)

| Scenario | rfmt | RuboCop | Speedup |
|----------|------|---------|---------|
| **Single File** | ~190ms | ~1.35s | **7.3x faster** |
| **Directory (14 files)** | 168ms | 1.67s | **10.0x faster** |
| **Full Project (111 files)** | 173ms | 10.09s | **58.5x faster** |
| **Check Mode (CI/CD)** | 172ms | 1.55s | **9.0x faster** |

### Key Metrics

- Single file: Formats in ~190ms
- Scaling: 58x faster on full project (111 files)
- CI/CD: Check time reduced from 10.09s to 0.173s (98% reduction)
- Variance: Low standard deviation across runs

### Throughput Comparison

| Directory | rfmt | RuboCop | Difference |
|-----------|------|---------|------------|
| app/models (14 files) | 83.5 files/s | 8.4 files/s | **10x throughput** |
| test/ (30 files) | 168.1 files/s | 18.1 files/s | **9.3x throughput** |

*Benchmark environment: Apple Silicon (arm64), macOS Darwin 23.6.0, Ruby 3.4.5*

See [detailed benchmark report](docspriv/benchmark_report.md) for full data.

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

This creates a `.rfmt.yml` file with default settings:

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

rfmt provides structured error messages:

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

rfmt includes a logging system:

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

Documentation is available in the [docs](docs/) directory. See [User Guide](docs/user_guide.md) or [Contributing Guide](CONTRIBUTING.md) for details.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Comparison with Other Tools

### rfmt vs RuboCop

| Feature | rfmt | RuboCop |
|---------|------|---------|
| **Primary Purpose** | Code formatting | Linting + formatting |
| **Speed** | 58x faster (tested benchmark) | Baseline |
| **Configuration** | Minimal | Extensive |
| **Code Quality Checks** | No | Yes |
| **Bug Detection** | No | Yes |

**Note**: rfmt focuses on formatting speed, while RuboCop provides additional code quality analysis. They can be used together.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Code of Conduct

Everyone interacting in the rfmt project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](CODE_OF_CONDUCT.md).

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issues](https://github.com/fujitanisora/rfmt/issues)
- 📧 Email: fujitanisora0414@gmail.com

## Acknowledgments

- Built with [Prism](https://github.com/ruby/prism) - Modern Ruby parser
- Powered by [Rust](https://www.rust-lang.org/) - Performance and safety
- FFI via [Magnus](https://github.com/matsadler/magnus) - Ruby-Rust bridge

---

<div align="center">

Created by [Fujitani Sora](https://github.com/fs0414)

</div>

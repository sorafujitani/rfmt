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

- **Opinionated**: Minimal configuration with consistent output
- **Idempotent**: Running multiple times produces identical results
- **Comment preservation**: Maintains existing comment placement
- **Rust implementation**: Core formatter implemented in Rust

## Features

### Performance

Built with Rust for improved execution speed. See Performance Benchmarks section for details.

### Consistent Style

Enforces code style rules:

- Automatic indentation
- Spacing and alignment normalization
- Quote style standardization
- Method definition formatting

## Performance Benchmarks

Performance comparison with RuboCop on a Rails project (111 files, 3,241 lines):

| Test Type | Files | rfmt | RuboCop | Ratio |
|-----------|-------|------|---------|-------|
| Single File | 1 | 191ms | 1.38s | 7.2x |
| Directory | 14 | 176ms | 1.68s | 9.6x |
| Full Project (check) | 111 | 172ms | 4.36s | 25.4x |

**Observations:**
- rfmt execution time remains constant (172-191ms) regardless of file count
- Low variance across runs (standard deviation: 8-23ms)

**Test Environment:**
- CPU: Apple Silicon (arm64)
- Ruby: 3.4.5
- rfmt: 0.2.4, RuboCop: 1.81.7

See [detailed benchmark report](docs/benchmark.md) for complete data.

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

### Initialize Configuration

First, create a configuration file with default settings:

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
rfmt init --path config/.rfmt.yml

# Overwrite existing configuration
rfmt init --force
```

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

Show diff without modifying files:

```bash
rfmt format lib/user.rb --diff
```

### Ruby API

**Input (unformatted code):**

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

**Output (formatted code):**

```ruby
class User
  def initialize(name)
    @name=name
  end
end
```

### Configuration

#### Configuration File Discovery

rfmt automatically searches for configuration files in this order:

1. Current directory (`.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, or `rfmt.yaml`)
2. Parent directories (up to root)
3. User home directory (`.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, or `rfmt.yaml`)
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

Documentation is available in the [docs](docs/) directory:

- [User Guide](docs/user_guide.md) - Comprehensive usage guide
- [Error Reference](docs/error_reference.md) - Error codes and troubleshooting
- [Contributing Guide](CONTRIBUTING.md) - How to contribute

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Comparison with Other Tools

### rfmt vs RuboCop

| Feature | rfmt | RuboCop |
|---------|------|---------|
| **Primary Purpose** | Code formatting | Linting + formatting |
| **Configuration** | Minimal | Extensive |
| **Code Quality Checks** | No | Yes |
| **Bug Detection** | No | Yes |

**Note**: rfmt focuses on code formatting, while RuboCop provides additional code quality analysis. They can be used together.

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

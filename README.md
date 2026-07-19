# kenshin

<div align="center">

A Ruby code formatter written in Rust

[![Gem Version](https://badge.fury.io/rb/kenshin.svg)](https://rubygems.org/gems/kenshin)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Installation](#installation) •
[Usage](#usage) •
[Features](#features) •
[Editor Integration](#editor-integration) •
[Documentation](#documentation) •
[Contributing](#contributing)

<a href="https://flatt.tech/oss/gmo/trampoline" target="_blank"><img src="https://flatt.tech/assets/images/badges/gmo-oss.svg" height="24px"/></a>

</div>

---

## What is kenshin?

[RubyGems reference](https://rubygems.org/gems/kenshin)
[DeepWiki](https://deepwiki.com/fs0414/rfmt)

**kenshin** is a Ruby code formatter that enforces consistent style across your codebase. Key characteristics:

- **Opinionated**: Minimal configuration with consistent output
- **Idempotent**: Running multiple times produces identical results
- **Comment preservation**: Maintains existing comment placement
- **Rust implementation**: Parsing and formatting both run natively in Rust (via the [ruby-prism](https://crates.io/crates/ruby-prism) crate); Ruby provides the CLI and LSP shell

## Features

### Performance

Built with Rust for improved execution speed. See Performance Benchmarks section for details.

### Consistent Style

Enforces code style rules:

- Automatic indentation
- Spacing and alignment normalization
- Quote style standardization
- Method definition formatting

## Performance

Parsing and formatting both run natively in Rust (the [ruby-prism](https://crates.io/crates/ruby-prism) crate, with prism statically linked), so the per-file cost is well under a millisecond:

| Pipeline | In-process format time |
|----------|------------------------|
| Before native parsing (Ruby Prism parse + JSON handoff to Rust; historical, not reproducible from this checkout) | 4.28 ms/file |
| Now (parsing and formatting in Rust) | 0.19 ms/file |

Measured with `scripts/bench_format.rb` over kenshin's own `lib/` corpus on arm64 macOS, Ruby 3.4. Reproduce with:

```bash
bundle exec ruby scripts/bench_format.rb
```

A cold CLI invocation (`kenshin --check FILE`) takes roughly 0.1-0.25 s of wall-clock time; that is Ruby VM startup, not formatting.

For more detail and a historical comparison against RuboCop, see [Performance Benchmarks](docs/benchmark.md).

## Installation

### Requirements

- Ruby 3.3 or higher
- Rust 1.70 or higher (for building from source)

### From RubyGems

```bash
gem install kenshin
```

### In Your Gemfile

```ruby
gem 'kenshin'
```

Then run:

```bash
bundle install
```

### From Source

```bash
git clone https://github.com/sorafujitani/rfmt.git
cd rfmt
bundle install
bundle exec rake compile
```

## Usage

### Initialize Configuration

First, create a configuration file with default settings:

```bash
kenshin init
```

This creates a `.kenshin.yml` file with default settings:

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
kenshin init --path config/.kenshin.yml

# Overwrite existing configuration
kenshin init --force
```

### Command Line

Format a single file:

```bash
kenshin lib/user.rb
```

Format multiple files:

```bash
kenshin lib/**/*.rb
```

Format all files in your project:

```bash
kenshin .
```

Check if files need formatting (CI/CD):

```bash
kenshin check .
```

Show diff without modifying files:

```bash
kenshin lib/user.rb --diff
```

Quiet mode (minimal output):

```bash
kenshin --quiet lib/**/*.rb
```

Enable verbose output for debugging:

```bash
kenshin --verbose lib/user.rb
```

#### Common Options

| Option | Description |
|--------|-------------|
| `--check` | Check formatting without writing files |
| `--diff` | Show diff of changes |
| `--quiet` | Minimal output |
| `--verbose` | Detailed output with timing |

### Output Modes

**Normal mode** (default):
```bash
$ kenshin app/
Processing 25 file(s)...
✓ Formatted app/controllers/users_controller.rb
✓ Formatted app/models/user.rb

✓ Processed 25 files
  (3 formatted, 22 unchanged)
```

**Quiet mode** (`--quiet` or `-q`):
```bash
$ kenshin --quiet app/
✓ 3 files formatted
```

**Verbose mode** (`--verbose` or `-v`):
```bash
$ kenshin --verbose app/
Processing 25 file(s)...
Using sequential processing for 25 files
✓ Formatted app/controllers/users_controller.rb  
✓ app/models/application_record.rb already formatted
...

✓ Processed 25 files
  (3 formatted, 22 unchanged)

Details:
  Total files: 25
  Total time: 0.45s
  Files/sec: 55.6
```

### Parallel Processing

kenshin automatically chooses the optimal processing mode:

- **< 20 files**: Sequential processing (fastest for small batches)  
- **20-49 files**: Automatic based on average file size
- **≥ 50 files**: Parallel processing (utilizes multiple cores)

You can override this behavior:

```bash
# Force parallel processing
kenshin --parallel app/

# Force sequential processing  
kenshin --no-parallel app/
```

### Cache Management

kenshin uses caching to improve performance on large codebases:

```bash
# Clear cache if needed
kenshin cache clear

# View cache statistics  
kenshin cache stats
```

### Ruby API

**Input (unformatted code):**

```ruby
require 'kenshin'

source = <<~RUBY
  class User
  def initialize(name)
  @name=name
  end
  end
RUBY

formatted = Kenshin.format(source)
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

kenshin automatically searches for configuration files in this order:

1. Current directory (`.kenshin.yml`, `.kenshin.yaml`, `kenshin.yml`, or `kenshin.yaml`)
2. Parent directories (up to root)
3. User home directory (`.kenshin.yml`, `.kenshin.yaml`, `kenshin.yml`, or `kenshin.yaml`)
4. Default settings (if no file found)

#### Ruby API for Configuration

```ruby
require 'kenshin'

# Generate configuration file
Kenshin::Config.init('.kenshin.yml', force: false)

# Find configuration file
config_path = Kenshin::Config.find
# => "/Users/username/project/.kenshin.yml"

# Check if configuration exists
Kenshin::Config.exists?
# => true

# Load configuration
config = Kenshin::Config.load
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

## Editor Integration

kenshin can integrate with editors in two ways:

- Standalone LSP: run `kenshin-lsp` directly from your editor. This works well for single
  Ruby scripts or projects without a Gemfile.
- Ruby LSP add-on: use kenshin as the formatter inside
  [Ruby LSP](https://shopify.github.io/ruby-lsp/).

For detailed setup instructions, see [Editor Integration Guide](docs/editors.md).

### Standalone LSP

After installing kenshin, configure your editor's Ruby language server command to `kenshin-lsp`.

```bash
gem install kenshin
kenshin-lsp
```

Example Neovim configuration (with `nvim-lspconfig`):

```lua
local configs = require("lspconfig.configs")
local lspconfig = require("lspconfig")

if not configs.kenshin then
  configs.kenshin = {
    default_config = {
      cmd = { "kenshin-lsp" },
      filetypes = { "ruby" },
      root_dir = lspconfig.util.root_pattern(".kenshin.yml", ".git"),
      single_file_support = true,
    },
  }
end

lspconfig.kenshin.setup({})
```

Helix, Emacs, and Zed configurations are covered in the
[Editor Integration Guide](docs/editors.md).

### VSCode (Quick Start)

1. Install [Ruby LSP extension](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp)
2. Add to your `settings.json`:

```json
{
  "rubyLsp.formatter": "kenshin",
  "editor.formatOnSave": true,
  "[ruby]": {
    "editor.defaultFormatter": "Shopify.ruby-lsp"
  }
}
```

### Neovim

```lua
require("lspconfig").ruby_lsp.setup({
  init_options = {
    formatter = "kenshin"
  }
})
```

See [Editor Integration Guide](docs/editors.md) for Helix, Emacs, Sublime Text, and more.

## Development

### Setup

After cloning the repository:

```bash
bundle install
bundle exec lefthook install
```

### Git Hooks

This project uses [lefthook](https://github.com/evilmartians/lefthook) for automated validation before push:

**Pre-push checks:**
- RuboCop (Ruby linting)
- cargo fmt --check (Rust formatting)
- cargo clippy (Rust linting)

**Skip hooks temporarily:**
```bash
# Skip all hooks for this push
LEFTHOOK=0 git push

# Skip specific hook
LEFTHOOK_EXCLUDE=rubocop git push
```

### Running Tests

```bash
# Ruby tests
bundle exec rspec

# Rust tests
cargo test --manifest-path ext/kenshin/Cargo.toml

# All tests
bundle exec rake dev:test_all
```

## Documentation

Documentation is available in the [docs](docs/) directory:

- [User Guide](docs/user_guide.md) - Comprehensive usage guide
- [Error Reference](docs/error_reference.md) - Error codes and troubleshooting
- [Contributing Guide](CONTRIBUTING.md) - How to contribute

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Comparison with Other Tools

### kenshin vs RuboCop

| Feature | kenshin | RuboCop |
|---------|------|---------|
| **Primary Purpose** | Code formatting | Linting + formatting |
| **Configuration** | Minimal | Extensive |
| **Code Quality Checks** | No | Yes |
| **Bug Detection** | No | Yes |

**Note**: kenshin focuses on code formatting, while RuboCop provides additional code quality analysis. They can be used together.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Code of Conduct

Everyone interacting in the kenshin project's codebases, issue trackers, chat rooms and mailing lists is expected to follow the [code of conduct](CODE_OF_CONDUCT.md).

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issues](https://github.com/sorafujitani/rfmt/issues)
- 📧 Email: fujitanisora0414@gmail.com

## Acknowledgments

- Built with [Prism](https://github.com/ruby/prism) - Modern Ruby parser
- Powered by [Rust](https://www.rust-lang.org/) - Performance and safety
- FFI via [Magnus](https://github.com/matsadler/magnus) - Ruby-Rust bridge

---

<div align="center">

Created by [Fujitani Sora](https://github.com/fs0414)

</div>

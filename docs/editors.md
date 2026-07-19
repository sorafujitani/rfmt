# Editor Integration

kenshin integrates with editors either as a standalone LSP server or as a formatter
inside [Ruby LSP](https://shopify.github.io/ruby-lsp/).

Use the standalone LSP when you want formatting for a single Ruby script or a
project without a Gemfile. Use the Ruby LSP add-on when your editor already runs
Ruby LSP and you want kenshin as one of its formatter backends.

## Prerequisites

1. Install kenshin gem:
   ```bash
   gem install kenshin
   ```

2. Configure either `kenshin-lsp` directly or Ruby LSP in your editor

## Standalone LSP

The `kenshin-lsp` executable starts a stdio LSP server that provides document
formatting. It does not require a project Gemfile or Ruby LSP. The server reads
`.kenshin.yml` from the workspace root, parent directories, or your home directory,
matching the normal kenshin configuration discovery behavior.

> **VSCode users**: VSCode integrates through Ruby LSP rather than the
> standalone server. See [Ruby LSP Add-on](#ruby-lsp-add-on) below.

### Neovim

With `nvim-lspconfig`, register a small custom server:

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

### Emacs eglot

```elisp
(require 'eglot)

(add-to-list 'eglot-server-programs
             '(ruby-mode . ("kenshin-lsp")))

(add-hook 'ruby-mode-hook 'eglot-ensure)
```

### Zed

Add to `settings.json`:

```json
{
  "languages": {
    "Ruby": {
      "format_on_save": "on"
    }
  },
  "lsp": {
    "kenshin": {
      "binary": {
        "path": "kenshin-lsp"
      }
    }
  }
}
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[language-server.kenshin]
command = "kenshin-lsp"

[[language]]
name = "ruby"
language-servers = ["kenshin"]
auto-format = true
```

## Ruby LSP Add-on

The Ruby LSP add-on is useful when your editor already uses Ruby LSP for
diagnostics, navigation, and other Ruby language features.

## VSCode

### Prerequisites

- [Ruby LSP extension](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp) installed
- kenshin gem installed

### Basic Setup

Add to your `settings.json` (user or workspace settings):

```json
{
  "rubyLsp.formatter": "kenshin"
}
```

### Format on Save

To automatically format on save, add the following settings:

```json
{
  "rubyLsp.formatter": "kenshin",
  "editor.formatOnSave": true,
  "[ruby]": {
    "editor.defaultFormatter": "Shopify.ruby-lsp"
  }
}
```

### Settings Reference

| Setting | Description |
|---------|-------------|
| `rubyLsp.formatter` | Specifies the formatter used by Ruby LSP |
| `editor.formatOnSave` | Enables automatic formatting on save |
| `editor.defaultFormatter` | Specifies the default formatter for Ruby files |

### Project-Specific Settings

For per-project configuration, add to `.vscode/settings.json`:

```json
{
  "rubyLsp.formatter": "kenshin",
  "editor.formatOnSave": true,
  "[ruby]": {
    "editor.defaultFormatter": "Shopify.ruby-lsp"
  }
}
```

## Neovim

### nvim-lspconfig

```lua
require("lspconfig").ruby_lsp.setup({
  init_options = {
    formatter = "kenshin"
  }
})
```

### Format on Save (optional)

```lua
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.rb",
  callback = function()
    vim.lsp.buf.format()
  end,
})
```

## Helix

### Configuration

Add to `~/.config/helix/languages.toml`:

```toml
[language-server.ruby-lsp]
command = "ruby-lsp"

[[language]]
name = "ruby"
language-servers = ["ruby-lsp"]
auto-format = true
```

Create `.ruby-lsp/config.yml` in your project root:

```yaml
formatter: kenshin
```

## Emacs

### eglot

```elisp
(require 'eglot)

(add-to-list 'eglot-server-programs
             '(ruby-mode . ("ruby-lsp")))

(add-hook 'ruby-mode-hook 'eglot-ensure)
```

Configure formatter in `.ruby-lsp/config.yml`:
```yaml
formatter: kenshin
```

## Zed

### Configuration

Add to your `settings.json` (global or `.zed/settings.json` for project-specific):

```json
{
  "lsp": {
    "ruby-lsp": {
      "initialization_options": {
        "formatter": "kenshin"
      }
    }
  }
}
```

### Format on Save

```json
{
  "languages": {
    "Ruby": {
      "format_on_save": "on"
    }
  },
  "lsp": {
    "ruby-lsp": {
      "initialization_options": {
        "formatter": "kenshin"
      }
    }
  }
}
```

## Troubleshooting

### Add-on Not Detected

1. Verify kenshin is installed:
   ```bash
   gem list kenshin
   ```

2. Check Ruby LSP version (requires >= 0.17.0):
   ```bash
   gem list ruby-lsp
   ```

3. Restart your editor/LSP server

### Formatting Not Working

1. Check LSP logs for errors
2. Verify `.kenshin.yml` is valid YAML
3. Test kenshin CLI directly:
   ```bash
   kenshin format test.rb
   ```

### VSCode: Formatting Not Triggered

1. Verify Ruby LSP extension is enabled
2. Check kenshin gem is installed: `gem list kenshin`
3. Open VSCode Output panel and select "Ruby LSP" to check logs
4. Run "Ruby LSP: Restart" from the Command Palette

## Configuration

kenshin reads configuration from `.kenshin.yml`. See [Configuration Guide](./configuration.md) for details.

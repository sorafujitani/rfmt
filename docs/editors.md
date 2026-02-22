# Editor Integration

rfmt integrates with editors through [Ruby LSP](https://shopify.github.io/ruby-lsp/).

## Prerequisites

1. Install rfmt gem:
   ```bash
   gem install rfmt
   ```

2. Ensure Ruby LSP is configured in your editor

## VSCode

### Prerequisites

- [Ruby LSP extension](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp) installed
- rfmt gem installed

### Basic Setup

Add to your `settings.json` (user or workspace settings):

```json
{
  "rubyLsp.formatter": "rfmt"
}
```

### Format on Save

To automatically format on save, add the following settings:

```json
{
  "rubyLsp.formatter": "rfmt",
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
  "rubyLsp.formatter": "rfmt",
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
    formatter = "rfmt"
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
formatter: rfmt
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
formatter: rfmt
```

## Zed

### Configuration

Add to your `settings.json` (global or `.zed/settings.json` for project-specific):

```json
{
  "lsp": {
    "ruby-lsp": {
      "initialization_options": {
        "formatter": "rfmt"
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
        "formatter": "rfmt"
      }
    }
  }
}
```

## Troubleshooting

### Add-on Not Detected

1. Verify rfmt is installed:
   ```bash
   gem list rfmt
   ```

2. Check Ruby LSP version (requires >= 0.17.0):
   ```bash
   gem list ruby-lsp
   ```

3. Restart your editor/LSP server

### Formatting Not Working

1. Check LSP logs for errors
2. Verify `.rfmt.yml` is valid YAML
3. Test rfmt CLI directly:
   ```bash
   rfmt format test.rb
   ```

### VSCode: Formatting Not Triggered

1. Verify Ruby LSP extension is enabled
2. Check rfmt gem is installed: `gem list rfmt`
3. Open VSCode Output panel and select "Ruby LSP" to check logs
4. Run "Ruby LSP: Restart" from the Command Palette

## Configuration

rfmt reads configuration from `.rfmt.yml`. See [Configuration Guide](./configuration.md) for details.

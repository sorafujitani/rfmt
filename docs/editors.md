# Editor Integration

rfmt integrates with editors through [Ruby LSP](https://shopify.github.io/ruby-lsp/).

## Prerequisites

1. Install rfmt gem:
   ```bash
   gem install rfmt
   ```

2. Ensure Ruby LSP is configured in your editor

## VSCode

### Installation

1. Install [Ruby LSP extension](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp)

2. Add to your `settings.json`:
   ```json
   {
     "rubyLsp.formatter": "rfmt"
   }
   ```

### Format on Save

Enable format on save:
```json
{
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

## Sublime Text

### LSP Package

1. Install [LSP package](https://packagecontrol.io/packages/LSP)
2. Install [LSP-ruby-lsp](https://packagecontrol.io/packages/LSP-ruby-lsp)
3. Configure in LSP.sublime-settings:
   ```json
   {
     "clients": {
       "ruby-lsp": {
         "initializationOptions": {
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

## Configuration

rfmt reads configuration from `.rfmt.yml`. See [Configuration Guide](./configuration.md) for details.

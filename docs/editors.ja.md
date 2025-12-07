# エディタ統合

rfmtは[Ruby LSP](https://shopify.github.io/ruby-lsp/)を通じてエディタと統合します。

## 前提条件

1. rfmt gemをインストール:
   ```bash
   gem install rfmt
   ```

2. エディタでRuby LSPが設定されていることを確認

## VSCode

### インストール

1. [Ruby LSP拡張](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp)をインストール

2. `settings.json`に追加:
   ```json
   {
     "rubyLsp.formatter": "rfmt"
   }
   ```

### 保存時フォーマット

保存時フォーマットを有効にする:
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

### 保存時フォーマット（オプション）

```lua
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.rb",
  callback = function()
    vim.lsp.buf.format()
  end,
})
```

## Helix

### 設定

`~/.config/helix/languages.toml`に追加:

```toml
[language-server.ruby-lsp]
command = "ruby-lsp"

[[language]]
name = "ruby"
language-servers = ["ruby-lsp"]
auto-format = true
```

プロジェクトルートに`.ruby-lsp/config.yml`を作成:

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

`.ruby-lsp/config.yml`でフォーマッターを設定:
```yaml
formatter: rfmt
```

## トラブルシューティング

### アドオンが検出されない

1. rfmtがインストールされているか確認:
   ```bash
   gem list rfmt
   ```

2. Ruby LSPのバージョンを確認（0.17.0以上が必要）:
   ```bash
   gem list ruby-lsp
   ```

3. エディタ/LSPサーバーを再起動

### フォーマットが動作しない

1. LSPログでエラーを確認
2. `.rfmt.yml`が有効なYAMLか確認
3. rfmt CLIを直接テスト:
   ```bash
   rfmt format test.rb
   ```

## 設定

rfmtは`.rfmt.yml`から設定を読み込みます。詳細は[設定ガイド](./configuration.md)を参照してください。

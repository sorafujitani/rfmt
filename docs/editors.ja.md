# エディタ統合

rfmtは[Ruby LSP](https://shopify.github.io/ruby-lsp/)を通じてエディタと統合します。

## 前提条件

1. rfmt gemをインストール:
   ```bash
   gem install rfmt
   ```

2. エディタでRuby LSPが設定されていることを確認

## VSCode

### 前提条件

- [Ruby LSP拡張](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp)がインストールされていること
- rfmt gemがインストールされていること

### 基本設定

`settings.json`（ユーザー設定またはワークスペース設定）に以下を追加:

```json
{
  "rubyLsp.formatter": "rfmt"
}
```

### Format on Save（保存時自動フォーマット）

保存時に自動でフォーマットを実行するには、以下の設定を追加:

```json
{
  "rubyLsp.formatter": "rfmt",
  "editor.formatOnSave": true,
  "[ruby]": {
    "editor.defaultFormatter": "Shopify.ruby-lsp"
  }
}
```

### 設定項目の説明

| 設定 | 説明 |
|------|------|
| `rubyLsp.formatter` | Ruby LSPで使用するフォーマッターを指定 |
| `editor.formatOnSave` | 保存時に自動フォーマットを有効化 |
| `editor.defaultFormatter` | Rubyファイルのデフォルトフォーマッターを指定 |

### プロジェクト固有の設定

プロジェクトごとに設定する場合は、`.vscode/settings.json`に記述:

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

## Zed

### 設定

`settings.json`（グローバルまたはプロジェクト固有の`.zed/settings.json`）に追加:

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

### 保存時フォーマット

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

### VSCodeでフォーマットが実行されない

1. Ruby LSP拡張が有効になっているか確認
2. rfmt gemがインストールされているか確認: `gem list rfmt`
3. VSCodeの出力パネルで「Ruby LSP」を選択しログを確認
4. コマンドパレットから「Ruby LSP: Restart」を実行

## 設定

rfmtは`.rfmt.yml`から設定を読み込みます。詳細は[設定ガイド](./configuration.md)を参照してください。

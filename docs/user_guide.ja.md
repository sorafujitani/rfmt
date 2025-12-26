# rfmt ユーザーガイド

## 目次

- [インストール](#インストール)
- [基本的な使い方](#基本的な使い方)
- [設定](#設定)
- [コマンドラインインターフェース](#コマンドラインインターフェース)
- [Ruby API](#ruby-api)
- [エディタ統合](#エディタ統合)
- [エラーハンドリング](#エラーハンドリング)
- [トラブルシューティング](#トラブルシューティング)
- [FAQ](#faq)

## インストール

### 必要環境

- Ruby 3.0 以上
- Rust 1.70 以上（ソースからビルドする場合）

### RubyGemsからインストール

```bash
gem install rfmt
```

### ソースからインストール

```bash
git clone https://github.com/fs0414/rfmt.git
cd rfmt
bundle install
bundle exec rake compile
```

### インストールの確認

```bash
rfmt version
```

## 基本的な使い方

### 単一ファイルのフォーマット

```bash
rfmt lib/my_file.rb
```

ファイルを変更せずにプレビューするには：

```bash
rfmt --check lib/my_file.rb
```

### 複数ファイルのフォーマット

```bash
rfmt lib/**/*.rb
```

### プロジェクト全体のフォーマット

```bash
rfmt .
```

### フォーマットが必要かチェック

`--check` フラグを使用して、変更を加えずにフォーマットを検証：

```bash
rfmt check lib/**/*.rb
```

CI/CDパイプラインで便利です。フォーマットが必要なファイルがある場合、ゼロ以外のステータスで終了します。

### 標準入力からのフォーマット

```bash
echo "class Foo;def bar;42;end;end" | rfmt -
```

## 設定

プロジェクトルートに `.rfmt.yml` ファイルを作成して、フォーマット動作をカスタマイズできます：

```yaml
version: "1.0"

formatting:
  # 最大行長
  line_length: 100

  # インデント幅（スペース数）
  indent_width: 2

  # インデントスタイル: "spaces" または "tabs"
  indent_style: "spaces"

  # 引用符スタイル: "double" または "single"
  quote_style: "double"

# 含めるファイル（globパターン）
include:
  - "**/*.rb"
  - "**/*.rake"
  - "**/Rakefile"
  - "**/Gemfile"

# 除外するファイル（globパターン）
exclude:
  - "vendor/**/*"
  - "tmp/**/*"
  - "node_modules/**/*"
  - "db/schema.rb"
```

### 設定の優先順位

rfmtは以下の順序で設定を探します：

1. カレントディレクトリの `.rfmt.yml`
2. 親ディレクトリの `.rfmt.yml`（ツリーを上に辿る）
3. `~/.rfmt.yml`（ユーザーレベルの設定）
4. デフォルト設定

### 設定オプション

#### `formatting.line_length`

**型:** Integer
**デフォルト:** 100
**説明:** 折り返し前の最大行長

```yaml
formatting:
  line_length: 120  # より長い行を許可
```

#### `formatting.indent_width`

**型:** Integer
**デフォルト:** 2
**説明:** インデントレベルごとのスペース（またはタブ）数

```yaml
formatting:
  indent_width: 4  # レベルごとに4スペース
```

#### `formatting.indent_style`

**型:** String (`"spaces"` または `"tabs"`)
**デフォルト:** `"spaces"`
**説明:** インデントにスペースまたはタブを使用

```yaml
formatting:
  indent_style: "tabs"
```

#### `formatting.quote_style`

**型:** String (`"double"` または `"single"`)
**デフォルト:** `"double"`
**説明:** 文字列の推奨引用符スタイル

```yaml
formatting:
  quote_style: "single"  # シングルクォートを使用
```

## コマンドラインインターフェース

### グローバルオプション

すべてのコマンドで使用可能なオプション：

- `--config PATH`: カスタム設定ファイルのパス
- `--verbose` または `-v`: 詳細な出力とデバッグログを有効化

### コマンド

#### `rfmt [FILES...]`（デフォルト）

Rubyファイルをフォーマットします。これがデフォルトコマンドです。

**オプション:**
- `--check`: ファイルを変更せずにフォーマットが必要かチェック
- `--config PATH`: 設定ファイルのパス
- `--diff`: 変更の差分を表示
- `--verbose`: 詳細な出力を有効化

**例:**

```bash
# ファイルをフォーマットして変更
rfmt lib/user.rb lib/post.rb

# フォーマットをチェック（CI/CD用）
rfmt --check lib/**/*.rb

# 変更せずに差分を表示
rfmt --diff lib/user.rb
```

#### `rfmt check [FILES...]`

ファイルがフォーマットを必要とするかチェック（`rfmt --check` のエイリアス）。

```bash
rfmt check .
```

#### `rfmt version`

バージョン情報を表示します。

```bash
rfmt version
```

### 終了コード

- `0`: 成功（すべてのファイルがフォーマット済み、または変更不要）
- `1`: エラーが発生
- `2`: フォーマットが必要（`--check` 使用時）

## Ruby API

### 基本的なフォーマット

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

**出力:**
```ruby
class User
  def initialize(name)
    @name = name
  end
end
```

### 設定付きフォーマット

```ruby
require 'rfmt'

config = {
  formatting: {
    indent_width: 4,
    quote_style: 'single'
  }
}

formatted = Rfmt.format(source, config: config)
```

### ASTの解析と検査

```ruby
require 'rfmt'

source = "class Foo\nend"

# JSON ASTに解析
ast_json = Rfmt::PrismBridge.parse(source)
ast = JSON.parse(ast_json)

# 構造を検査
puts ast['ast']['node_type']  # => "program_node"
puts ast['comments'].length   # => 0
```

### エラーハンドリング

```ruby
require 'rfmt'

begin
  result = Rfmt.format(invalid_source)
rescue Rfmt::ParseError => e
  puts "構文エラー: #{e.message}"
  # エラーには以下が含まれます:
  # - エラーコード（例: E001）
  # - 行番号と列番号
  # - エラーを示すコードスニペット
rescue Rfmt::Error => e
  puts "フォーマットエラー: #{e.message}"
end
```

## エラーハンドリング

rfmtは問題を素早く修正するための詳細なエラーメッセージを提供します。

### エラーコード

すべてのエラーにはエラーコードとヘルプURLが含まれます：

| コード | タイプ | 説明 |
|------|------|-------------|
| E001 | ParseError | ソースコードのRuby構文エラー |
| E002 | ConfigError | 無効な設定ファイル |
| E003 | IoError | ファイル読み書きエラー |
| E004 | FormattingError | フォーマット処理中のエラー |
| E005 | RuleError | フォーマットルール適用の失敗 |
| E006 | UnsupportedFeature | 未サポートの機能 |
| E007 | PrismError | Prismパーサー統合エラー |
| E008 | FormatError | 一般的なフォーマットエラー |
| E999 | InternalError | 内部バグ（報告してください） |

### エラーフォーマット

```
[Rfmt::ParseError] example.rb:5:10の構文エラー
クラス定義の終了'end'が必要です

コード:
   3 | class User
   4 |   def initialize
   5 |     @name = name
     |          ^
   6 | end

ヘルプ: https://rfmt.dev/errors/E001
```

### よくあるエラー

#### E001: 構文エラー

**原因:** コードにRubyの構文エラーがある

**解決方法:** フォーマット前に構文エラーを修正

```ruby
# 悪い例
class User
  def initialize
    @name = name
  # メソッドの'end'が不足
end

# 良い例
class User
  def initialize
    @name = name
  end
end
```

#### E002: 設定エラー

**原因:** 無効な `.rfmt.yml` 設定ファイル

**解決方法:** スキーマに対して設定を確認

```yaml
# 悪い例
formatting:
  line_length: "100"  # 文字列ではなく整数であるべき

# 良い例
formatting:
  line_length: 100
```

#### E006: 未サポート機能

**原因:** コードがrfmtでまだサポートされていないRuby機能を使用

**解決方法:** [ロードマップ](https://github.com/fs0414/rfmt/blob/main/ROADMAP.md)を確認するか、issueを作成

## トラブルシューティング

### 大きなファイルでrfmtが遅い

**解決方法:** rfmtは高速に設計されていますが、非常に大きなファイル（10,000行以上）は時間がかかる場合があります。以下を検討してください：

1. 大きなファイルを小さなモジュールに分割
2. `--config` を使用して高コストなチェックを無効化
3. 複数のファイルで並列にrfmtを実行

### コメントが移動される

**問題:** rfmtはすべてのコメントを保持しますが、一貫性のために位置を変更する場合があります

**解決方法:** これは期待される動作です。rfmtはコード構造に対する相対的なコメント位置を維持します。

### rfmtがコードの動作を変更した

**問題:** フォーマットは動作を変更すべきではありません

**解決方法:** これはバグです！以下の情報でissueを作成してください：
- 元のコード
- フォーマット後のコード
- 期待される動作
- 実際の動作

### CI/CDパイプラインがrfmtで失敗する

**よくある原因:**

1. **異なるrfmtバージョン:** Gemfileでバージョンを固定
   ```ruby
   gem 'rfmt', '~> 0.1.0'
   ```

2. **設定が見つからない:** `.rfmt.yml` がgitにコミットされていることを確認

3. **ファイルがフォーマットを必要とする:** まずローカルで `rfmt .` を実行

### デバッグ情報の取得

問題が発生した場合、デバッグログを有効にして詳細情報を確認できます：

**--verboseフラグの使用:**
```bash
rfmt file.rb --verbose
# または
rfmt file.rb -v
```

**環境変数の使用:**
```bash
# DEBUGでデバッグログを有効化
DEBUG=1 rfmt file.rb

# rfmt固有のデバッグログを有効化
RFMT_DEBUG=1 rfmt file.rb

# ログレベルを直接制御
RFMT_LOG=debug rfmt file.rb
```

デバッグログで表示される内容：
- 初期化メッセージ
- 設定ファイルの検出
- ファイル処理の詳細
- 内部Rust拡張の操作

## FAQ

### rfmtはコメントを保持しますか？

**はい！** rfmtは元の位置のすべてのコメントを保持します。行コメント、ブロックコメント、ドキュメントコメントすべてが維持されます。

### フォーマットは冪等ですか？

**はい！** 同じファイルに対してrfmtを複数回実行しても同じ結果が得られます。これはテストスイートで保証されています。

### 特定のルールを無効化できますか？

まだできません。rfmtは設定なしで一貫したスタイルに従います。これは議論を減らすための設計です。強い用途がある場合は、issueを作成してください。

### rfmtとRuboCopの比較は？

rfmtは**フォーマッター**で、RuboCopは**リンター**です：

| 機能 | rfmt | RuboCop |
|---------|------|---------|
| コードフォーマット | ✅ | ✅（自動修正付き） |
| スタイル強制 | ✅ | ✅ |
| コードスメル検出 | ❌ | ✅ |
| バグ検出 | ❌ | ✅ |
| パフォーマンス | 非常に高速 | 中程度 |
| 設定 | 最小限 | 広範囲 |

**推奨:** 両方を使用！ 一貫したフォーマットにはrfmtを、コード品質チェックにはRuboCopを使用してください。

### rfmtはRailsで動作しますか？

**はい！** rfmtはRailsアプリケーションを含むあらゆるRubyコードで動作します。以下を正しく処理します：

- モデル、コントローラー、ビュー
- マイグレーション
- ルーティング（`config/routes.rb`）
- Rakeタスク
- イニシャライザー

### pre-commitフックでrfmtを使用できますか？

**はい！** `.pre-commit-config.yaml` の例：

```yaml
repos:
  - repo: local
    hooks:
      - id: rfmt
        name: rfmt
        entry: bundle exec rfmt
        language: system
        files: \.rb$
```

または [Lefthook](https://github.com/evilmartians/lefthook) で：

```yaml
# lefthook.yml
pre-commit:
  commands:
    rfmt:
      glob: "*.rb"
      run: bundle exec rfmt {staged_files}
```

### サポートされているRubyバージョンは？

rfmtは **Ruby 3.0以上** をサポートします。以下でテストしています：

- Ruby 3.0
- Ruby 3.1
- Ruby 3.2
- Ruby 3.3

### コントリビュートするには？

詳細は[コントリビューティングガイド](../CONTRIBUTING.md)を参照してください：

- 開発環境のセットアップ
- テストの実行
- プルリクエストの提出
- コードスタイルガイドライン

### ヘルプはどこで得られますか？

- 📖 ドキュメント: https://rfmt.dev/docs
- 🐛 Issues: https://github.com/fs0414/rfmt/issues
- 💬 Discussions: https://github.com/fs0414/rfmt/discussions
- 📧 Email: fujitanisora0414@gmail.com

## 次のステップ

- [APIドキュメント](api_documentation.md)を読む
- [コントリビューティング](../CONTRIBUTING.md)について学ぶ
- [ロードマップ](../ROADMAP.md)を確認
- [エラーリファレンス](error_reference.ja.md)をレビュー

---

**バージョン:** 0.2.4
**最終更新:** 2025-11-25
**ライセンス:** MIT

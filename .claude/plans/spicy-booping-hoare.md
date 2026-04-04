# VSCode format on save 動作確認 & テスト充実化

## Context

rfmtにはRuby LSP Addon（`lib/ruby_lsp/rfmt/`）によるformat on save機能が既に実装されているが、動作確認が不十分で、テストも最小限の状態。ruby-lsp 0.26.4のインターフェースと照合した結果、**未実装の必須メソッドが2つ**見つかった。これらを修正し、テストを充実させることで、format on saveの信頼性を確保する。

## 発見された問題

### 1. `version` メソッド未実装（addon.rb）
- `RubyLsp::Addon` 基底クラスで `version` は abstract method（呼び出されると `AbstractMethodInvokedError` が発生）
- ruby-lsp がアドオンのバージョン互換性チェック時に呼び出す
- **参照**: `.nix-gem-home/gems/ruby-lsp-0.26.4/lib/ruby_lsp/addon.rb:222-225`

### 2. `run_range_formatting` メソッド未実装（formatter_runner.rb）
- Formatter インターフェースで定義されている（`run_formatting`, `run_range_formatting`, `run_diagnostic` の3つ）
- 範囲フォーマット（選択範囲のみフォーマット）で使用される
- **参照**: `.nix-gem-home/gems/ruby-lsp-0.26.4/lib/ruby_lsp/requests/support/formatter.rb:18`

## 実装計画

### Step 1: `addon.rb` に `version` メソッドを追加

**ファイル**: `lib/ruby_lsp/rfmt/addon.rb`

```ruby
def version
  ::Rfmt::VERSION
end
```

既存の `Rfmt::VERSION`（`lib/rfmt/version.rb`）を再利用する。

### Step 2: `formatter_runner.rb` に `run_range_formatting` メソッドを追加

**ファイル**: `lib/ruby_lsp/rfmt/formatter_runner.rb`

```ruby
def run_range_formatting(uri, source, base_indentation)
  ::Rfmt.format(source)
rescue ::Rfmt::Error
  nil
end
```

### Step 3: `formatter_runner_spec.rb` にテストケースを追加

**ファイル**: `spec/ruby_lsp/rfmt/formatter_runner_spec.rb`

追加するテストケース:

| テスト | 目的 |
|-------|------|
| フォーマット結果の内容検証 | `be_a(String)` だけでなく、インデントが正しいか等を検証 |
| べき等性（idempotency） | 2回フォーマットしても同じ結果になること（format on save は毎保存で実行されるため重要） |
| 複数メソッドのクラス | 実用的なサイズのコードでの検証 |
| 非Rfmt::Errorの例外が伝播すること | `rescue ::Rfmt::Error` のスコープが正しいことの確認 |
| `run_range_formatting` の基本動作 | 新規追加メソッドのテスト |

### Step 4: `addon_spec.rb` にテストケースを追加

**ファイル**: `spec/ruby_lsp/rfmt/addon_spec.rb`

追加するテストケース:
- `version` が `Rfmt::VERSION` を返すこと

### Step 5: スモークテストスクリプトの作成

**ファイル**: `script/smoke_test_formatter.rb`

ruby-lsp を起動せずに FormatterRunner を直接呼び出し、format on save と同等の処理パスを検証する簡易スクリプト。手動での動作確認やCI上でも実行可能。

## 変更対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `lib/ruby_lsp/rfmt/addon.rb` | `version` メソッド追加 |
| `lib/ruby_lsp/rfmt/formatter_runner.rb` | `run_range_formatting` メソッド追加 |
| `spec/ruby_lsp/rfmt/formatter_runner_spec.rb` | テストケース5件追加 |
| `spec/ruby_lsp/rfmt/addon_spec.rb` | `version` テスト追加 |
| `script/smoke_test_formatter.rb` | 新規作成（スモークテスト） |

## 検証方法

```bash
# 1. 全テスト実行
bundle exec rspec spec/ruby_lsp/

# 2. スモークテスト実行
bundle exec ruby script/smoke_test_formatter.rb

# 3. 全体のテストスイートが壊れていないことを確認
bundle exec rspec
```

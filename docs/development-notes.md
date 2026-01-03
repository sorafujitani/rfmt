# Development Notes

rfmt開発における知見・注意事項をまとめたドキュメント。

---

## 動作確認

### 安全な確認方法

rfmtは開発中のフォーマッターであり、未対応ノードがある状態でCLIを使用するとソースコードを破壊する可能性がある。

| 方法 | 安全性 | 用途 |
|------|--------|------|
| `bundle exec rspec` | ✅ 安全 | 自動テスト |
| IRB経由 `Rfmt.format(source)` | ✅ 安全 | 手動確認 |
| `ruby -e` 経由 | ✅ 安全 | ワンライナー確認 |
| `echo 'code' \| bundle exec rfmt -` | ⚠️ 注意 | 出力確認のみ |
| `bundle exec rfmt .` | ❌ 危険 | 使用禁止（開発中） |

### 推奨手順

```bash
# 1. ビルド
bundle exec rake compile

# 2. 自動テスト
bundle exec rspec

# 3. 手動確認（IRB経由）
bundle exec irb -r./lib/rfmt -e "puts Rfmt.format(':hello')"

# 4. 変更確認
git status
git diff
```

### 発生しうる破壊的変更

CLIでフォーマットした際に以下の破壊が発生した事例がある：

- heredocの内容消失（`<<~YAML ... YAML`）
- `self.`の削除（クラスメソッド定義）
- rescue/raise節の消失
- `end`の対応崩れ（構文エラー）

### リカバリー

意図しない変更が発生した場合：

```bash
# 変更したファイル以外を元に戻す
git checkout HEAD -- <file>

# または全て元に戻す
git checkout HEAD -- .
```

---

## テスト

### テスト実行

```bash
# 全テスト
bundle exec rspec

# 特定ファイル
bundle exec rspec spec/node_types_spec.rb

# 詳細出力
bundle exec rspec --format documentation

# Rustテスト
cargo test --manifest-path ext/rfmt/Cargo.toml
```

### テスト作成時の注意

- `require 'spec_helper'` を必ず含める
- heredocを使う場合は構文が壊れやすいので、単純な文字列リテラルを推奨

```ruby
# 推奨: 単純な文字列
source = "def foo\n  @name\nend"

# 注意: heredocは外部ツールで壊れる可能性あり
source = <<~RUBY
  def foo
    @name
  end
RUBY
```

---

## ビルド

### Rust拡張のコンパイル

```bash
bundle exec rake compile
```

### クリーンビルド

```bash
bundle exec rake clean
bundle exec rake compile
```

---

## Git

### コミット前の確認

```bash
# 変更ファイル確認
git status

# 差分確認
git diff

# 意図しない変更がないか確認してからコミット
```

---

## 追加予定

- [ ] デバッグ方法
- [ ] パフォーマンス計測
- [ ] リリース手順

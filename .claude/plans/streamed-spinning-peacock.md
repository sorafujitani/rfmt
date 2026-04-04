# Issue #87: ポストフィックスif/unlessのインラインコメントが次の行に移動するバグ修正

## 問題概要

```ruby
# 入力
some_method if condition # steep:ignore

# 実際の出力（バグ）
some_method if condition
# steep:ignore

# 期待される出力
some_method if condition # steep:ignore
```

## 根本原因

`ext/rfmt/src/emitter/mod.rs:1003` のポストフィックスif処理で `emit_trailing_comments()` が呼ばれずに `return Ok(())` している。

他の形式（inline_then: L1064、通常if: L1087）では正しく呼ばれており、ポストフィックスifのみ欠落。

## 修正計画（t-wada TDDサイクル）

### Cycle 1: Red → Green（基本ケース）

#### Red: 失敗テストを書く

`spec/conditional_formatting_spec.rb` に追加:

```ruby
describe 'postfix if/unless with inline comments (Issue #87)' do
  it 'preserves inline comment after postfix if' do
    source = "some_method if condition # comment\n"
    result = Rfmt.format(source)
    expect(result).to eq("some_method if condition # comment\n")
  end
end
```

確認: `bundle exec rspec spec/conditional_formatting_spec.rb` → FAIL

#### Green: 最小限の修正

`ext/rfmt/src/emitter/mod.rs` L1003の `return Ok(());` の前に1行追加:

```rust
self.emit_trailing_comments(node.location.end_line)?;
return Ok(());
```

確認: `bundle exec rake compile && bundle exec rspec spec/conditional_formatting_spec.rb` → PASS

### Cycle 2: Red → Green（unless版）

#### Red: unless版テスト追加

```ruby
it 'preserves inline comment after postfix unless' do
  source = "some_method unless condition # comment\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method unless condition # comment\n")
end
```

#### Green: Cycle 1の修正で既にPASSするはず（同じコードパスを通るため）

### Cycle 3: Red → Green（実ユースケース steep:ignore）

```ruby
it 'preserves tool directive comment (steep:ignore) after postfix if' do
  source = "some_method if condition # steep:ignore\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method if condition # steep:ignore\n")
end
```

### Cycle 4: リグレッション防止テスト

```ruby
it 'preserves postfix if without comment (regression)' do
  source = "some_method if condition\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method if condition\n")
end

it 'preserves postfix unless without comment (regression)' do
  source = "some_method unless condition\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method unless condition\n")
end
```

### Refactor

修正が1行追加のみのため、リファクタリングは不要。

### 全体リグレッション確認

```bash
bundle exec rspec
```

## 修正対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `ext/rfmt/src/emitter/mod.rs` | L1003前に `emit_trailing_comments` 呼び出し追加（1行） |
| `spec/conditional_formatting_spec.rb` | ポストフィックスif/unless + コメントのテスト追加（5ケース） |

## 検証手順

1. `bundle exec rake compile` — Rust拡張のビルド成功
2. `bundle exec rspec spec/conditional_formatting_spec.rb` — 新規テスト全PASS
3. `bundle exec rspec` — 全テストPASS（リグレッションなし）

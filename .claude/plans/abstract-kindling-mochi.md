# Issue #87: modifier if のインラインコメントが次の行に移動される

## Context

modifier if/unless（後置if）の行末に付いたインラインコメント（例: `# steep:ignore`）が、フォーマット後に次の行へ移動されてしまうバグ。`# steep:ignore` のようなツールディレクティブは対象行に留まる必要があるため、コメント位置の変更はセマンティクスの破壊につながる。

**入力**: `some_method if condition # steep:ignore`
**期待**: そのまま保持
**実際**: コメントが次の行に分離される

## 原因

`ext/rfmt/src/emitter/mod.rs` の `emit_if_unless` 関数内、postfix if パス（行985-1004）で `return Ok(())` する前に `emit_trailing_comments` を呼んでいない。

同じ行のコメントが未出力のまま残り、後続処理で standalone コメントとして次の行に出力される。

ternary operator パス（行1013-1039）にも同じ問題がある。

**比較:**
- postfix if（行1003）: `return Ok(());` ← `emit_trailing_comments` なし
- ternary（行1038）: `return Ok(());` ← `emit_trailing_comments` なし
- inline then（行1064）: `self.emit_trailing_comments(...)` あり ← 正常

## 修正

### 1. postfix if パスに `emit_trailing_comments` を追加

**ファイル**: `ext/rfmt/src/emitter/mod.rs` 行1002付近

`return Ok(());` の直前に追加:
```rust
self.emit_trailing_comments(node.location.end_line)?;
return Ok(());
```

### 2. ternary operator パスにも同様に追加

**ファイル**: `ext/rfmt/src/emitter/mod.rs` 行1037付近

`return Ok(());` の直前に追加:
```rust
self.emit_trailing_comments(node.location.end_line)?;
return Ok(());
```

### 3. テスト追加

**ファイル**: `spec/conditional_formatting_spec.rb`（既存の postfix if テストの後に追加）

```ruby
it 'preserves inline comments on postfix if' do
  source = "some_method if condition # steep:ignore\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method if condition # steep:ignore\n")
end

it 'preserves inline comments on postfix unless' do
  source = "some_method unless condition # steep:ignore\n"
  result = Rfmt.format(source)
  expect(result).to eq("some_method unless condition # steep:ignore\n")
end
```

## 検証

```bash
bundle exec rake compile
bundle exec rspec
bundle exec rspec spec/conditional_formatting_spec.rb
```

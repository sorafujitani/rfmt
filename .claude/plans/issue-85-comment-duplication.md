# Issue #85: チェインメソッド＋ブロック時のコメント重複バグ修正

## 問題の概要

**入力コード:**
```ruby
some_method # comment
  .method_with_block { do_something }
```

**期待される出力:** 変更なし（既にフォーマット済み）

**実際の出力:**
```ruby
some_method # comment
  .method_with_block { do_something }
# comment  ← 重複して追加される
```

---

## 根本原因

### コード箇所
`ext/rfmt/src/emitter/mod.rs`

### 問題のメカニズム

1. **`emit_call()`** (L1134-1164) がブロック付きメソッドチェーンを処理
2. **`emit_call_without_block()`** (L1184-1202) がソースから `call_node.start_offset` ～ `block_node.start_offset` を抽出
   - この時、`# comment` は抽出テキストに**含まれている**
   - しかし `emitted_comment_indices` に**登録されない**
3. **`emit_brace_block()`** (L1276-1312) でブロック処理後に `emit_trailing_comments(block_end_line)` を呼び出し
   - `block_end_line` = 2（ブロック終了行）
   - コメントは行1にあるため、ここでは出力されない（問題なし）

4. **真の問題箇所**: `emit_call()` が完了した後、親ノードの処理で再度 `emit_trailing_comments()` が呼ばれる可能性
   - または、`emit_statements()` などで末尾コメント処理が行われる

### 問題の本質
`emit_call_without_block()` でソーステキストを抽出した際、そのテキストに含まれるインラインコメントを `emitted_comment_indices` にマークしていないため、後続の処理で同じコメントが再出力される。

---

## 修正方針

### 修正箇所
`emit_call_without_block()` 関数 (L1184-1202)

### 修正内容
ソーステキスト抽出後、その範囲内に含まれるコメントを `emitted_comment_indices` にマークする処理を追加。

```rust
fn emit_call_without_block(
    &mut self,
    call_node: &Node,
    block_node: &Node,
    indent_level: usize,
) -> Result<()> {
    self.emit_indent(indent_level)?;

    if !self.source.is_empty() {
        let start = call_node.location.start_offset;
        let end = block_node.location.start_offset;

        if let Some(text) = self.source.get(start..end) {
            write!(self.buffer, "{}", text.trim_end())?;

            // ★追加: 抽出範囲内のコメントを emitted としてマーク
            let call_start_line = call_node.location.start_line;
            let block_start_line = block_node.location.start_line;

            for (idx, comment) in self.all_comments.iter().enumerate() {
                if !self.emitted_comment_indices.contains(&idx)
                    && comment.location.start_line >= call_start_line
                    && comment.location.start_line < block_start_line
                {
                    self.emitted_comment_indices.insert(idx);
                }
            }
        }
    }

    Ok(())
}
```

---

## 実装タスク

1. [ ] `emit_call_without_block()` にコメントマーキング処理を追加
2. [ ] テストケースを追加
   - インラインコメント + ブレースブロック
   - インラインコメント + do-endブロック
   - 複数行チェーン + コメント
3. [ ] 既存テストが通ることを確認

---

## 検証方法

### 再現テスト
```bash
# テストファイル作成
echo 'some_method # comment
  .method_with_block { do_something }' > /tmp/test_comment.rb

# フォーマット実行（--diffで差分確認）
bundle exec rfmt /tmp/test_comment.rb --diff

# 期待: 差分なし
```

### ユニットテスト実行
```bash
bundle exec rspec
```

---

## 関連ファイル

- `ext/rfmt/src/emitter/mod.rs` - 主な修正対象
  - `emit_call()`: L1134-1164
  - `emit_call_without_block()`: L1184-1202（修正箇所）
  - `emit_trailing_comments()`: L427-440
  - `emit_brace_block()`: L1276-1312

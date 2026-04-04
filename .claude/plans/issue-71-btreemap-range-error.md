# Issue #71 修正計画: BTreeMap範囲エラー（Endless Method + Comment）

## 1. 問題の原因特定

### 1.1 問題の所在
- **ファイルパス**: `ext/rfmt/src/emitter/mod.rs`
- **問題箇所1**: `get_comment_indices_in_range()` (L146-152)
- **問題箇所2**: `emit_comments_before_end()` (L257)
- **問題箇所3**: `has_comments_in_range()` (L230-238)

### 1.2 根本原因
**分類**: ロジックエラー（境界条件の考慮不足）

**原因**:
- `def a = nil`（endless method）は単一行で`start_line == end_line`
- `emit_comments_before_end()`で`(start_line + 1)..end_line`という範囲を作成
- endless methodの場合`4..3`のような無効な範囲になる
- `BTreeMap.range()`は`start > end`でパニック

**トリガー条件**:
```ruby
class Test
  # comment
  def a = nil  # endless method
end
```

### 1.3 問題パターンと学び

**パターン**: "Range Boundary Error" - 境界値を考慮しない範囲生成

**根本的な誤解**:
- 誤解: `start_line + 1`は常に`end_line`以下である
- 正解: endless methodでは`start_line == end_line`なので無効になる

---

## 2. 解決策

### 2.1 修正方針
`get_comment_indices_in_range()`と`has_comments_in_range()`に範囲チェックを追加し、`start_line >= end_line`の場合は早期リターンする。

### 2.2 変更差分

**ファイル**: `ext/rfmt/src/emitter/mod.rs`

```diff
# get_comment_indices_in_range() (L146-152)
fn get_comment_indices_in_range(&self, start_line: usize, end_line: usize) -> Vec<usize> {
+   // Guard against invalid range (e.g., endless methods where start_line >= end_line)
+   if start_line >= end_line {
+       return Vec::new();
+   }
    self.comments_by_line
        .range(start_line..end_line)
        .flat_map(|(_, indices)| indices.iter().copied())
        .filter(|&idx| !self.emitted_comment_indices.contains(&idx))
        .collect()
}
```

```diff
# has_comments_in_range() (L230-238)
fn has_comments_in_range(&self, start_line: usize, end_line: usize) -> bool {
+   // Guard against invalid range (e.g., endless methods where start_line >= end_line)
+   if start_line >= end_line {
+       return false;
+   }
    self.comments_by_line
        .range(start_line..end_line)
        .flat_map(|(_, indices)| indices.iter())
        .any(|&idx| {
            !self.emitted_comment_indices.contains(&idx)
                && self.all_comments[idx].location.end_line < end_line
        })
}
```

### 2.3 なぜこの修正で解決するか

- **技術的根拠**: 無効な範囲`4..3`がBTreeMapに渡される前にチェックされる
- **Before**: `range(4..3)` → パニック
- **After**: `4 >= 3`で早期リターン → 空の結果を返す（endless methodにはボディ内コメントがない）

### 2.4 影響分析
- **破壊的変更**: なし
- **パフォーマンス**: 変化なし（早期リターンでむしろ微改善）
- **影響モジュール**: emitter/mod.rs のみ
- **下位互換性**: 保たれる

---

## 3. 修正計画

### Phase 1: 修正実装
1. `get_comment_indices_in_range()`に範囲チェック追加
2. `has_comments_in_range()`に範囲チェック追加

### Phase 2: テスト追加
1. `spec/rfmt_spec.rb`にendless method + commentのテスト追加
2. 再現コードでの動作確認

### Phase 3: 検証
1. `bundle exec rspec`で全テスト実行
2. 再現コードでのフォーマット確認

---

## 4. 修正対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `ext/rfmt/src/emitter/mod.rs` | 2箇所に範囲チェック追加 |
| `spec/rfmt_spec.rb` | endless method + commentのテスト追加 |

---

## 5. 検証方法

```bash
# テスト用ファイル作成
echo 'class Test
  # comment
  def a = nil
end' > /tmp/test_endless.rb

# フォーマット実行（エラーが出ないことを確認）
bundle exec rfmt /tmp/test_endless.rb

# RSpecテスト実行
bundle exec rspec
```

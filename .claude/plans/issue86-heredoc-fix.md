# Issue #86 修正計画 - t-wada TDDアプローチ

## バグ概要

**Issue**: [#86](https://github.com/fs0414/rfmt/issues/86) - Heredocコンテンツと終了識別子が誤って削除される

**現象**: メソッド呼び出しの引数として複数のHeredocを使用すると、本体と終了識別子が削除される

```ruby
# 入力
puts(<<~HEREDOC, <<~HEREDOC2)
  This is a heredoc.
HEREDOC
  This is another heredoc.
HEREDOC2

# 現在の出力（バグ）
puts(<<~HEREDOC, <<~HEREDOC2)
# ← Heredoc内容が全て削除される
```

**根本原因**: `lib/rfmt/prism_bridge.rb`の`extract_location`メソッドが直接の子ノードのみをチェックしている。CallNodeの場合、Heredocは`ArgumentsNode`を介した孫ノードにあるため検出されない。

---

## TDDサイクル

### Phase 1: Red（失敗テスト作成）

**ファイル**: `spec/rfmt_spec.rb` (行185の後に追加)

```ruby
describe 'heredoc in method call arguments (Issue #86)' do
  it 'preserves multiple heredocs as method arguments' do
    source = <<~RUBY
      puts(<<~HEREDOC, <<~HEREDOC2)
        This is a heredoc.
      HEREDOC
        This is another heredoc.
      HEREDOC2
    RUBY
    result = Rfmt.format(source)

    expect(result).to include('This is a heredoc.')
    expect(result).to include('This is another heredoc.')
    expect(result).to match(/^HEREDOC$/m)
    expect(result).to match(/^HEREDOC2$/m)
    expect(Prism.parse(result).errors).to be_empty
  end

  it 'preserves single heredoc as method argument' do
    source = <<~RUBY
      puts(<<~HEREDOC)
        Single heredoc content.
      HEREDOC
    RUBY
    result = Rfmt.format(source)

    expect(result).to include('Single heredoc content.')
    expect(result).to match(/^HEREDOC$/m)
    expect(Prism.parse(result).errors).to be_empty
  end

  it 'preserves heredoc with method chain' do
    source = <<~RUBY
      foo.bar(<<~SQL)
        SELECT * FROM users
      SQL
    RUBY
    result = Rfmt.format(source)

    expect(result).to include('SELECT * FROM users')
    expect(result).to match(/^SQL$/m)
    expect(Prism.parse(result).errors).to be_empty
  end

  it 'preserves heredoc with block' do
    source = <<~RUBY
      process(<<~DATA) do |result|
        content here
      DATA
        puts result
      end
    RUBY
    result = Rfmt.format(source)

    expect(result).to include('content here')
    expect(result).to match(/^DATA$/m)
    expect(Prism.parse(result).errors).to be_empty
  end
end
```

**検証**: `bundle exec rspec spec/rfmt_spec.rb -e "Issue #86"` → 全テスト失敗を確認

---

### Phase 2: Green（最小実装）

**ファイル**: `lib/rfmt/prism_bridge.rb`

**修正内容**: `extract_location`メソッドに再帰的な`closing_loc`検索を追加

```ruby
# Extract location information from node
def self.extract_location(node)
  loc = node.location

  end_offset = loc.end_offset
  end_line = loc.end_line
  end_column = loc.end_column

  # Check this node's closing_loc
  if node.respond_to?(:closing_loc) && node.closing_loc
    closing = node.closing_loc
    if closing.end_offset > end_offset
      end_offset = closing.end_offset
      end_line = closing.end_line
      end_column = closing.end_column
    end
  end

  # Recursively check all descendant nodes for heredoc closing_loc
  # Issue #86: Handles CallNode -> ArgumentsNode -> StringNode (heredoc)
  max_closing = find_max_closing_loc_recursive(node)
  if max_closing && max_closing[:end_offset] > end_offset
    end_offset = max_closing[:end_offset]
    end_line = max_closing[:end_line]
    end_column = max_closing[:end_column]
  end

  {
    start_line: loc.start_line,
    start_column: loc.start_column,
    end_line: end_line,
    end_column: end_column,
    start_offset: loc.start_offset,
    end_offset: end_offset
  }
end

# Recursively find the maximum closing_loc among all descendant nodes
# Returns nil if no closing_loc found, otherwise { end_offset:, end_line:, end_column: }
def self.find_max_closing_loc_recursive(node, depth: 0)
  return nil if depth > 10 # Prevent infinite recursion

  max_closing = nil

  node.child_nodes.compact.each do |child|
    # Check if this child has a closing_loc (heredoc)
    if child.respond_to?(:closing_loc) && child.closing_loc
      closing = child.closing_loc
      if max_closing.nil? || closing.end_offset > max_closing[:end_offset]
        max_closing = {
          end_offset: closing.end_offset,
          end_line: closing.end_line,
          end_column: closing.end_column
        }
      end
    end

    # Recursively check grandchildren
    child_max = find_max_closing_loc_recursive(child, depth: depth + 1)
    if child_max && (max_closing.nil? || child_max[:end_offset] > max_closing[:end_offset])
      max_closing = child_max
    end
  end

  max_closing
end
```

**検証**: `bundle exec rspec spec/rfmt_spec.rb -e "Issue #86"` → 全テストパス

---

### Phase 3: Refactor

1. 既存の直接子ノードチェック（行128-138）を削除し、再帰メソッドに統一
2. コメント追加（Issue #74と#86の参照）
3. 全回帰テスト実行

---

## 検証手順

```bash
# 1. 失敗テスト確認（Red）
bundle exec rspec spec/rfmt_spec.rb -e "Issue #86" --format documentation

# 2. 実装後のテスト確認（Green）
bundle exec rspec spec/rfmt_spec.rb -e "Issue #86" --format documentation

# 3. 回帰テスト（既存機能の確認）
bundle exec rspec

# 4. 手動検証
echo 'puts(<<~HEREDOC, <<~HEREDOC2)
  This is a heredoc.
HEREDOC
  This is another heredoc.
HEREDOC2' > /tmp/test.rb
bundle exec rfmt /tmp/test.rb --diff
```

---

## 修正対象ファイル

| ファイル | 変更内容 |
|----------|----------|
| `spec/rfmt_spec.rb` | Issue #86 失敗テスト追加（行185後） |
| `lib/rfmt/prism_bridge.rb` | `extract_location`に再帰検索追加、`find_max_closing_loc_recursive`メソッド追加 |

---

## 実装順序

1. **Red**: `spec/rfmt_spec.rb`にテスト追加 → 失敗確認
2. **Green**: `lib/rfmt/prism_bridge.rb`に再帰メソッド実装 → テストパス
3. **Refactor**: コード整理、全テスト実行

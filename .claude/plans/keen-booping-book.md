# Issue #73: BlockNode内のrescue句でコードが削除されるバグの修正

## 問題概要

`rescue`句を含む`do...end`ブロックをフォーマットすると、ブロック本体とrescue句が削除される。

```ruby
# 入力
foo.each do |x|
  x
rescue StandardError => e
  e
end

# 現在の出力（バグ）
foo.each do |x|
end
```

## 根本原因

### 調査結果: BlockNodeのbodyに来る可能性のあるノードタイプ

| ノードタイプ | 条件 | 発生するブロック形式 |
|------------|------|-------------------|
| `StatementsNode` | 通常のブロック | `do...end`, `{ }` |
| `BeginNode` | rescue/ensure/else付き | `do...end`のみ |
| `nil` | 空のブロック | 両方 |

**重要**: `{ }`ブロックでは`rescue`/`ensure`は**構文エラー**になるため、`BeginNode`は来ない。

### BeginNodeの構造（rescue/ensure/else付きブロック）

```
BlockNode
└── body: BeginNode
    ├── statements: StatementsNode (ブロック本体)
    ├── rescue_clause: RescueNode (オプション)
    ├── else_clause: ElseNode (オプション、rescueの後)
    └── ensure_clause: EnsureNode (オプション)
```

### 問題箇所

| ファイル | 問題 |
|---------|------|
| `ext/rfmt/src/emitter/mod.rs` | `emit_do_end_block`が`BeginNode`を無視 |
| `lib/rfmt/prism_bridge.rb` | `BeginNode`で`else_clause`が欠落 |

## TDDサイクル実装計画

### Phase 1: Red - 失敗するテストを作成

**新規ファイル**: `spec/block_rescue_formatting_spec.rb`

```ruby
# frozen_string_literal: true

require 'spec_helper'

RSpec.describe Rfmt, 'Block with Rescue Formatting' do
  describe 'do...end block with rescue' do
    it 'preserves block body and rescue clause' do
      source = <<~RUBY
        foo.each do |x|
          x
        rescue StandardError => e
          e
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('x')
      expect(result).to include('rescue StandardError => e')
    end

    it 'formats block with multiple rescue clauses' do
      source = <<~RUBY
        data.map do |d|
          transform(d)
        rescue TypeError => e
          handle_type_error(e)
        rescue StandardError => e
          handle_standard_error(e)
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('rescue TypeError')
      expect(result).to include('rescue StandardError')
    end

    it 'formats block with rescue and else' do
      source = <<~RUBY
        items.each do |item|
          process(item)
        rescue => e
          handle_error(e)
        else
          success
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('else')
      expect(result).to include('success')
    end

    it 'formats block with rescue, else, and ensure' do
      source = <<~RUBY
        file.each_line do |line|
          parse(line)
        rescue ParseError => e
          log_error(e)
        else
          mark_success
        ensure
          cleanup
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('rescue ParseError')
      expect(result).to include('else')
      expect(result).to include('ensure')
    end

    it 'formats block with only ensure' do
      source = <<~RUBY
        conn.transaction do |tx|
          execute(tx)
        ensure
          tx.close
        end
      RUBY
      result = Rfmt.format(source)
      expect(result).to include('ensure')
      expect(result).to include('tx.close')
    end
  end
end
```

### Phase 2: Green - 最小限の修正

#### 修正1: Ruby側 (`lib/rfmt/prism_bridge.rb`)

`BeginNode`の子ノードに`else_clause`を追加：

```ruby
when Prism::BeginNode
  [
    node.statements,
    node.rescue_clause,
    node.else_clause,      # 追加
    node.ensure_clause
  ].compact
```

#### 修正2: Rust側 (`ext/rfmt/src/emitter/mod.rs`)

`emit_do_end_block`関数を修正して`BeginNode`を処理：

```rust
fn emit_do_end_block(&mut self, block_node: &Node, indent_level: usize) -> Result<()> {
    write!(self.buffer, " do")?;
    self.emit_block_parameters(block_node)?;
    self.emit_trailing_comments(block_node.location.start_line)?;
    self.buffer.push('\n');

    let block_start_line = block_node.location.start_line;
    let block_end_line = block_node.location.end_line;
    let mut last_stmt_end_line = block_start_line;

    for child in &block_node.children {
        match &child.node_type {
            NodeType::StatementsNode => {
                self.emit_statements(child, indent_level + 1)?;
                if let Some(last_child) = child.children.last() {
                    last_stmt_end_line = last_child.location.end_line;
                }
                self.buffer.push('\n');
                break;
            }
            NodeType::BeginNode => {
                // rescue/ensure/else付きブロック本体
                // emit_beginの暗黙的begin処理を利用
                self.emit_begin(child, indent_level + 1)?;
                last_stmt_end_line = child.location.end_line;
                break;
            }
            _ => {
                // パラメータノードはスキップ
            }
        }
    }

    // コメント処理とend出力（既存コードと同じ）
    let had_internal_comments =
        self.has_comments_in_range(block_start_line + 1, block_end_line);
    if had_internal_comments {
        self.emit_comments_in_range_with_prev_line(
            block_start_line + 1,
            block_end_line,
            indent_level + 1,
            last_stmt_end_line,
        )?;
    }

    if had_internal_comments && !self.buffer.ends_with('\n') {
        self.buffer.push('\n');
    }

    self.emit_indent(indent_level)?;
    write!(self.buffer, "end")?;
    self.emit_trailing_comments(block_end_line)?;

    Ok(())
}
```

### Phase 3: Refactor

- 既存の`emit_begin`の暗黙的begin処理を活用
- 改行・インデントの調整が必要な場合は微調整

## 修正対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `spec/block_rescue_formatting_spec.rb` | 新規作成 - 失敗テスト |
| `lib/rfmt/prism_bridge.rb` | `BeginNode`に`else_clause`追加 |
| `ext/rfmt/src/emitter/mod.rs` | `emit_do_end_block`で`BeginNode`処理 |

## 検証手順

```bash
# 1. 新規テスト実行（最初は失敗）
bundle exec rspec spec/block_rescue_formatting_spec.rb

# 2. Ruby側修正後、Rust拡張ビルド
bundle exec rake compile

# 3. 新規テスト実行（通過確認）
bundle exec rspec spec/block_rescue_formatting_spec.rb

# 4. 全テスト実行（回帰確認）
bundle exec rspec

# 5. 手動確認
cat <<'EOF' | bundle exec rfmt
foo.each do |x|
  x
rescue StandardError => e
  e
else
  success
ensure
  cleanup
end
EOF
```

## 注意点

- `emit_begin`の暗黙的begin処理では`emit_node`が呼ばれ、各ノードタイプ用のemit関数が呼び出される
- `emit_rescue`と`emit_ensure`は`indent_level.saturating_sub(1)`でインデントを調整する
- ブロック内の場合、`emit_begin`に渡すインデントレベルは`indent_level + 1`とする

# rfmt - Ruby Formatter

## プロジェクト概要
RubyコードフォーマッターでRust拡張を使用

## 開発コマンド
```bash
# テスト実行
bundle exec rspec

# フォーマッター実行
bundle exec rfmt <file>

# Rust拡張のビルド
bundle exec rake compile
```

## Claude Code設定

### プランファイルの保存場所
プランモードで作成されるプランファイルは、以下のディレクトリに保存すること:
- **推奨**: `.claude/plans/` （カレントディレクトリ直下）
- グローバルの`~/.claude/plans/`ではなく、プロジェクトローカルの`.claude/plans/`を使用する

### 日本語での応答
- 日本語で応答すること

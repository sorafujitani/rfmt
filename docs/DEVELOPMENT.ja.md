# 開発ガイド

kenshinのテスト、ビルド、リリース手順を説明します。

## 目次

- [前提条件](#前提条件)
- [ビルド](#ビルド)
- [テスト](#テスト)
- [開発ワークフロー](#開発ワークフロー)
- [リリース手順](#リリース手順)
- [トラブルシューティング](#トラブルシューティング)

## 前提条件

### 必要なツール

- **Ruby**: 3.3以降
- **Rust**: 1.70以降（rustupでインストール）
- **Bundler**: `gem install bundler`
- **Rake**: Ruby標準ライブラリに含まれる

### システム依存関係

**Rustのインストール** (rustup経由 - すべてのプラットフォームで共通):
```bash
# rustupとRustツールチェーンをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 画面の指示に従い、その後：
source $HOME/.cargo/env

# インストールの確認
rustc --version
cargo --version
```

**追加の依存関係**:

- **macOS**: Xcode Command Line Tools
  ```bash
  xcode-select --install
  ```

- **Linux (Debian/Ubuntu)**:
  ```bash
  sudo apt-get update
  sudo apt-get install build-essential
  ```

- **Windows**:
  - https://rustup.rs からrustup-init.exeをダウンロードして実行
  - Visual Studio C++ Build Toolsをインストール

### セットアップ

```bash
# リポジトリのクローン
git clone https://github.com/yourusername/kenshin.git
cd kenshin

# Ruby依存関係のインストール
bundle install

# ネイティブ拡張のビルド
bundle exec rake compile
```

## ビルド

### クリーンビルド

```bash
# すべてのビルド成果物を削除
bundle exec rake clean
bundle exec rake clobber

# 最初から再ビルド
bundle exec rake compile
```

### 開発ビルド

```bash
# 変更されたファイルのみを素早く再ビルド
bundle exec rake compile
```

### ビルドオプション

```bash
# デバッグモードでビルド（コンパイルは速いが実行は遅い）
cd ext/kenshin
cargo build

# リリースモードでビルド（rake compileのデフォルト）
cargo build --release

# ビルドせずにコンパイルエラーをチェック
cargo check
```

## テスト

### Rubyテスト

#### すべてのテストを実行

```bash
# すべてのRSpecテスト
bundle exec rake spec

# または
bundle exec rspec
```

#### 特定のテストを実行

```bash
# 単一のテストファイル
bundle exec rspec spec/formatter_spec.rb

# 行番号を指定して特定のテスト
bundle exec rspec spec/formatter_spec.rb:45

# パターンに一致するテスト
bundle exec rspec spec/formatter_spec.rb -e "indentation"
```

#### テスト出力オプション

```bash
# ドキュメント形式（詳細）
bundle exec rspec --format documentation

# プログレス形式（デフォルト）
bundle exec rspec --format progress

# 失敗したテストのみ
bundle exec rspec --only-failures
```

#### CLIテスト ⭐

コマンドラインインターフェース機能をテスト：

```bash
# すべてのCLIテストを実行
bundle exec rspec spec/cli_spec.rb

# 特定のCLIテストを実行
bundle exec rspec spec/cli_spec.rb -e "format with diff option"
```

**CLIテストのカバレッジ:**
- versionコマンド（`kenshin version`）
- formatコマンドの各種オプション（`--write`, `--no-write`, `--check`, `--diff`, `--verbose`）
- チェックモードの終了コード（フォーマット済みで0、未フォーマットで1）
- 3つの形式での差分表示（unified, color, side_by_side）
- 複数ファイルの処理
- エラーハンドリング（構文エラー、ファイル未存在）
- initコマンド（`.kenshin.yml`の作成）
- configコマンド（設定の表示）

**テストケースの例:**
```ruby
# write オプションでのフォーマットテスト
it 'formats and writes to file' do
  cli.options = { write: true }
  cli.format(temp_file.path)

  formatted = File.read(temp_file.path)
  expect(formatted).to eq(formatted_code)
end

# チェックモードの終了コードテスト
it 'exits with code 1 when formatting is needed' do
  cli.options = { check: true, write: false }

  expect do
    cli.format(temp_file.path)
  end.to raise_error(SystemExit) do |error|
    expect(error.status).to eq(1)
  end
end

# 差分表示のテスト
it 'shows unified diff' do
  cli.options = { diff: true, write: false, diff_format: 'unified' }
  expect { cli.format(temp_file.path) }.not_to raise_error
end
```

#### 設定システムのテスト ⭐

YAML設定システムをテスト：

```bash
# すべての設定テストを実行
bundle exec rspec spec/configuration_spec.rb

# 特定の設定テストを実行
bundle exec rspec spec/configuration_spec.rb -e "discovers .kenshin.yml"
```

**設定テストのカバレッジ:**
- 設定ファイルの自動発見（`.kenshin.yml`, `.kenshin.yaml`, `kenshin.yml`, `kenshin.yaml`）
- デフォルト設定の読み込み
- カスタム設定ファイルの読み込み
- 設定のマージ（ネストされたハッシュの深いマージ）
- バリデーション（line_length > 0, indent_width > 0）
- ファイルパターンマッチング（include/exclude）
- フォーマット設定の取得

**テストケースの例:**
```ruby
# 設定ファイル発見のテスト
it 'discovers .kenshin.yml' do
  File.write('.kenshin.yml', "version: '1.0'")
  config = described_class.discover
  expect(config).to be_a(described_class)
end

# 設定バリデーションのテスト
it 'validates positive line_length' do
  expect do
    described_class.new('formatting' => { 'line_length' => -1 })
  end.to raise_error(Kenshin::Configuration::ConfigError, 'line_length must be positive')
end

# ファイルパターンマッチングのテスト
it 'includes files matching include patterns' do
  config = described_class.new
  files = config.files_to_format(base_path: temp_dir)
  expect(files).to include(File.join(temp_dir, 'lib', 'test.rb'))
end
```

### Rustテスト

#### すべてのRustテストを実行

```bash
cd ext/kenshin

# すべてのテスト
cargo test

# ライブラリテストのみ（統合テストを除く）
cargo test --lib

# 出力付き
cargo test -- --nocapture
```

#### 特定のRustテストを実行

```bash
# 特定のモジュールのテスト
cargo test ast::tests

# 単一のテスト
cargo test test_node_creation

# パターンに一致するテスト
cargo test parse
```

### コーパスチェック

コーパスチェックはリポジトリ内のすべての Ruby ファイルをフォーマットし、再フォーマット後の出力が構造的に同一の AST にパースされることを prism gem（開発専用の依存）で検証します。パリティフィクスチャとあわせて、フォーマッタ変更の主要な安全網です:

```bash
bundle exec ruby scripts/corpus_check.rb
```

### カバレッジ

```bash
# カバレッジツールのインストール（初回のみ）
cargo install cargo-tarpaulin

# カバレッジレポートの生成
cd ext/kenshin
cargo tarpaulin --out Html --output-dir ../../coverage
```

## 開発ワークフロー

### 1. 変更を加える

以下のファイルを編集：
- `lib/` - Rubyコード
- `ext/kenshin/src/` - Rustコード
- `spec/` - テスト

### 2. ビルド & テスト

```bash
# Rustコードを変更した後
bundle exec rake compile

# テストを実行
bundle exec rake spec

# Rustテストを実行
cd ext/kenshin && cargo test
```

### 3. 確認

```bash
# IRBで手動テスト
bundle exec irb -I lib -r kenshin

# IRB内で:
input = "class Foo\nend"
puts Kenshin.format(input)
```

### 4. フォーマット & Lint

```bash
# Rustコードをフォーマット
cd ext/kenshin
cargo fmt

# Lintをチェック
cargo clippy

# Rubyコードをフォーマット
bundle exec rubocop -a
```

### 5. 完全なテストスイートを実行

```bash
# すべてのテスト
bundle exec rake

# または
bundle exec rake spec
cd ext/kenshin && cargo test
```

## リリース手順

### リリース前チェックリスト

- [ ] すべてのテストが通過
- [ ] `lib/kenshin/version.rb`でバージョンを更新
- [ ] `ext/kenshin/Cargo.toml`でバージョンを更新
- [ ] CHANGELOG.mdを更新
- [ ] ドキュメントを更新
- [ ] コミットされていない変更がない

### バージョン更新

1. **Rubyバージョンの更新** (`lib/kenshin/version.rb`):

```ruby
module Kenshin
  VERSION = "0.2.0"  # ここを更新
end
```

2. **Rustバージョンの更新** (`ext/kenshin/Cargo.toml`):

```toml
[package]
name = "kenshin"
version = "0.2.0"  # ここを更新
```

3. **CHANGELOG.mdの更新**:

```markdown
## [0.2.0] - 2025-01-15

### 追加
- 新機能X
- 新機能Y

### 修正
- バグ修正Z
```

### Gemのビルド

```bash
# Gemパッケージをビルド
gem build kenshin.gemspec

# これにより kenshin-0.2.0.gem が作成されます
```

### Gemをローカルでテスト

```bash
# ローカルにインストール
gem install kenshin-0.2.0.gem

# テスト
irb
> require 'kenshin'
> Kenshin.format("class Foo\nend")
```

### RubyGemsへの公開

#### 初回セットアップ

```bash
# https://rubygems.org でRubyGemsアカウントを作成

# APIキーを取得
curl -u your_username https://rubygems.org/api/v1/api_key.yaml > ~/.gem/credentials
chmod 0600 ~/.gem/credentials
```

#### RubyGemsへのプッシュ

```bash
# Gemをプッシュ
gem push kenshin-0.2.0.gem

# https://rubygems.org/gems/kenshin で確認
```

### リリース後の作業

1. **Gitタグの作成**:

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

2. **GitHubリリースの作成**:

- https://github.com/yourusername/kenshin/releases/new にアクセス
- タグ `v0.2.0` を選択
- リリースタイトルを設定: `v0.2.0`
- CHANGELOGエントリを説明にコピー
- `kenshin-0.2.0.gem` ファイルを添付
- リリースを公開

3. **告知**:

- 必要に応じてREADMEを更新
- 重要なリリースの場合はRubyフォーラム/コミュニティに投稿

## トラブルシューティング

### ビルドの問題

#### 問題: "cargo: command not found"

```bash
# rustup経由でRustをインストール（公式の方法）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 現在のセッションでCargoをPATHに追加
source $HOME/.cargo/env

# 永続化するには、rustupインストーラーがシェルプロファイルに以下を追加します：
# export PATH="$HOME/.cargo/bin:$PATH"

# インストールの確認
cargo --version
```

#### 問題: "magnus version mismatch"

```bash
# クリーンして再ビルド
bundle exec rake clobber
bundle exec rake compile
```

#### 問題: git pull後にビルドが失敗

```bash
# 依存関係を更新
bundle install
cd ext/kenshin && cargo update

# クリーン再ビルド
bundle exec rake clobber compile
```

### テストの問題

#### 問題: 変更後にテストが失敗

```bash
# 拡張機能を再ビルド
bundle exec rake compile

# Rubyキャッシュをクリア
rm -rf tmp/

# テストを再実行
bundle exec rspec
```

#### 問題: "Cannot load such file -- kenshin/kenshin"

```bash
# 拡張機能がビルドされていないか、正しい場所にない
bundle exec rake compile

# 拡張機能の存在を確認
ls -la lib/kenshin/kenshin.bundle  # macOS
ls -la lib/kenshin/kenshin.so      # Linux
```

### 実行時の問題

#### 問題: "Prism integration error"

パースは Rust 拡張の内部で静的リンクされた ruby-prism crate により行われるため、このエラーは依存関係の問題ではなく kenshin 自体のバグを示します。Gemfile の prism gem はコーパスチェックとパリティフィクスチャで使う開発専用の依存であり、更新しても実行時のパースには影響しません。エラーを引き起こした入力を添えて issue を作成してください。

#### 問題: Segmentation fault

通常、Rustコードのバグを示します。デバッグするには：

```bash
# デバッグバージョンをビルド
cd ext/kenshin
cargo build

# デバッグ付きで実行
RUST_BACKTRACE=1 bundle exec ruby your_test.rb
```

### パフォーマンスの問題

#### 問題: フォーマットが遅い

```bash
# リリースビルドを使用していることを確認
bundle exec rake compile  # デフォルトで --release を使用

# 確認
file lib/kenshin/kenshin.bundle
# デバッグの場合は "not stripped"、リリースの場合は "stripped" と表示されるはず
```

## 開発のヒント

### 高速な反復

```bash
# ターミナル1: ファイル変更を監視
while true; do
  inotifywait -e modify ext/kenshin/src/*.rs
  bundle exec rake compile
done

# ターミナル2: テストを実行
bundle exec rspec
```

### デバッグ

#### Ruby側

```ruby
# コードに追加
require 'debug'
binding.break  # Ruby 3.1+

# または
require 'pry'
binding.pry
```

#### Rust側

```rust
// コードに追加
dbg!(&some_variable);

// または
eprintln!("Debug: {:?}", some_value);
```

実行：

```bash
RUST_BACKTRACE=1 bundle exec rspec
```

### ベンチマーク

```ruby
require 'benchmark'
require 'kenshin'

code = File.read('large_file.rb')

Benchmark.bm do |x|
  x.report("format:") { Kenshin.format(code) }
end
```

### メモリプロファイリング

```bash
# ツールのインストール
gem install memory_profiler

# プロファイルスクリプトの作成
cat > profile_memory.rb <<'EOF'
require 'memory_profiler'
require 'kenshin'

code = File.read('large_file.rb')

report = MemoryProfiler.report do
  Kenshin.format(code)
end

report.pretty_print
EOF

# 実行
ruby profile_memory.rb
```

## 継続的インテグレーション

GitHub Actionsワークフローの例 (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        ruby: ['3.3', '3.4', '4.0']

    steps:
    - uses: actions/checkout@v4

    - uses: ruby/setup-ruby@v1
      with:
        ruby-version: ${{ matrix.ruby }}
        bundler-cache: true

    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: 依存関係のインストール
      run: bundle install

    - name: 拡張機能のビルド
      run: bundle exec rake compile

    - name: Rubyテストの実行
      run: bundle exec rspec

    - name: Rustテストの実行
      run: cd ext/kenshin && cargo test
```

## 追加リソース

- [Rustドキュメント](https://doc.rust-jp.rs/book-ja/)
- [Magnusドキュメント](https://docs.rs/magnus/)
- [RSpecドキュメント](https://rspec.info/documentation/)
- [RubyGemsガイド](https://guides.rubygems.org/)

## サポート

- GitHub Issues: https://github.com/yourusername/kenshin/issues
- Discussions: https://github.com/yourusername/kenshin/discussions

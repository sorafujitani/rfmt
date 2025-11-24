# 開発ガイド

rfmtのテスト、ビルド、リリース手順を説明します。

## 目次

- [前提条件](#前提条件)
- [ビルド](#ビルド)
- [テスト](#テスト)
- [開発ワークフロー](#開発ワークフロー)
- [リリース手順](#リリース手順)
- [トラブルシューティング](#トラブルシューティング)

## 前提条件

### 必要なツール

- **Ruby**: 3.0以降
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
git clone https://github.com/yourusername/rfmt.git
cd rfmt

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
cd ext/rfmt
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
- versionコマンド（`rfmt version`）
- formatコマンドの各種オプション（`--write`, `--no-write`, `--check`, `--diff`, `--verbose`）
- チェックモードの終了コード（フォーマット済みで0、未フォーマットで1）
- 3つの形式での差分表示（unified, color, side_by_side）
- 複数ファイルの処理
- エラーハンドリング（構文エラー、ファイル未存在）
- initコマンド（`.rfmt.yml`の作成）
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
bundle exec rspec spec/configuration_spec.rb -e "discovers .rfmt.yml"
```

**設定テストのカバレッジ:**
- 設定ファイルの自動発見（`.rfmt.yml`, `.rfmt.yaml`, `rfmt.yml`, `rfmt.yaml`）
- デフォルト設定の読み込み
- カスタム設定ファイルの読み込み
- 設定のマージ（ネストされたハッシュの深いマージ）
- バリデーション（line_length > 0, indent_width > 0）
- ファイルパターンマッチング（include/exclude）
- フォーマット設定の取得

**テストケースの例:**
```ruby
# 設定ファイル発見のテスト
it 'discovers .rfmt.yml' do
  File.write('.rfmt.yml', "version: '1.0'")
  config = described_class.discover
  expect(config).to be_a(described_class)
end

# 設定バリデーションのテスト
it 'validates positive line_length' do
  expect do
    described_class.new('formatting' => { 'line_length' => -1 })
  end.to raise_error(Rfmt::Configuration::ConfigError, 'line_length must be positive')
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
cd ext/rfmt

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

### カバレッジ

```bash
# カバレッジツールのインストール（初回のみ）
cargo install cargo-tarpaulin

# カバレッジレポートの生成
cd ext/rfmt
cargo tarpaulin --out Html --output-dir ../../coverage
```

## 開発ワークフロー

### 1. 変更を加える

以下のファイルを編集：
- `lib/` - Rubyコード
- `ext/rfmt/src/` - Rustコード
- `spec/` - テスト

### 2. ビルド & テスト

```bash
# Rustコードを変更した後
bundle exec rake compile

# テストを実行
bundle exec rake spec

# Rustテストを実行
cd ext/rfmt && cargo test
```

### 3. 確認

```bash
# IRBで手動テスト
bundle exec irb -I lib -r rfmt

# IRB内で:
input = "class Foo\nend"
puts Rfmt.format(input)
```

### 4. フォーマット & Lint

```bash
# Rustコードをフォーマット
cd ext/rfmt
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
cd ext/rfmt && cargo test
```

## リリース手順

### リリース前チェックリスト

- [ ] すべてのテストが通過
- [ ] `lib/rfmt/version.rb`でバージョンを更新
- [ ] `ext/rfmt/Cargo.toml`でバージョンを更新
- [ ] CHANGELOG.mdを更新
- [ ] ドキュメントを更新
- [ ] コミットされていない変更がない

### バージョン更新

1. **Rubyバージョンの更新** (`lib/rfmt/version.rb`):

```ruby
module Rfmt
  VERSION = "0.2.0"  # ここを更新
end
```

2. **Rustバージョンの更新** (`ext/rfmt/Cargo.toml`):

```toml
[package]
name = "rfmt"
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
gem build rfmt.gemspec

# これにより rfmt-0.2.0.gem が作成されます
```

### Gemをローカルでテスト

```bash
# ローカルにインストール
gem install rfmt-0.2.0.gem

# テスト
irb
> require 'rfmt'
> Rfmt.format("class Foo\nend")
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
gem push rfmt-0.2.0.gem

# https://rubygems.org/gems/rfmt で確認
```

### リリース後の作業

1. **Gitタグの作成**:

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

2. **GitHubリリースの作成**:

- https://github.com/yourusername/rfmt/releases/new にアクセス
- タグ `v0.2.0` を選択
- リリースタイトルを設定: `v0.2.0`
- CHANGELOGエントリを説明にコピー
- `rfmt-0.2.0.gem` ファイルを添付
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
cd ext/rfmt && cargo update

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

#### 問題: "Cannot load such file -- rfmt/rfmt"

```bash
# 拡張機能がビルドされていないか、正しい場所にない
bundle exec rake compile

# 拡張機能の存在を確認
ls -la lib/rfmt/rfmt.bundle  # macOS
ls -la lib/rfmt/rfmt.so      # Linux
```

### 実行時の問題

#### 問題: "Prism integration error"

```bash
# Prism gemのバージョンを確認
bundle list | grep prism

# ~> 1.6.0 であるべき
# 必要に応じて更新
bundle update prism
```

#### 問題: Segmentation fault

通常、Rustコードのバグを示します。デバッグするには：

```bash
# デバッグバージョンをビルド
cd ext/rfmt
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
file lib/rfmt/rfmt.bundle
# デバッグの場合は "not stripped"、リリースの場合は "stripped" と表示されるはず
```

## 開発のヒント

### 高速な反復

```bash
# ターミナル1: ファイル変更を監視
while true; do
  inotifywait -e modify ext/rfmt/src/*.rs
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
require 'rfmt'

code = File.read('large_file.rb')

Benchmark.bm do |x|
  x.report("format:") { Rfmt.format(code) }
end
```

### メモリプロファイリング

```bash
# ツールのインストール
gem install memory_profiler

# プロファイルスクリプトの作成
cat > profile_memory.rb <<'EOF'
require 'memory_profiler'
require 'rfmt'

code = File.read('large_file.rb')

report = MemoryProfiler.report do
  Rfmt.format(code)
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
        ruby: ['3.0', '3.1', '3.2', '3.3']

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
      run: cd ext/rfmt && cargo test
```

## 追加リソース

- [Rustドキュメント](https://doc.rust-jp.rs/book-ja/)
- [Magnusドキュメント](https://docs.rs/magnus/)
- [RSpecドキュメント](https://rspec.info/documentation/)
- [RubyGemsガイド](https://guides.rubygems.org/)

## サポート

- GitHub Issues: https://github.com/yourusername/rfmt/issues
- Discussions: https://github.com/yourusername/rfmt/discussions

# rfmt formatフローの理解（プリントデバッグ学習）

## 目的

rfmtのformat処理の全体フローを、プリントデバッグを通じて手を動かしながら理解する。

## サンプルコード

学習全体を通して、以下のシンプルなRubyコードをフォーマット対象として使用する：

```ruby
def hello( name )
if name
puts "Hello, #{name}!"
end
end
```

---

## フロー概要図

```
exe/rfmt
  ↓
Rfmt::CLI.start(args)
  ↓
Rfmt.format(source)
  ├─ PrismBridge.parse(source)     ← Ruby側: ソース → AST JSON
  └─ format_code(source, json)     ← Rust側: AST JSON → フォーマット済みコード
      ├─ PrismAdapter::parse(json)   → Rust内部AST
      ├─ Config::discover()          → 設定読み込み
      └─ Emitter::emit(ast)          → コード生成
```

---

## ビルドシステムと動作確認方法

### プロジェクト構成

rfmtは **Ruby + Rust ハイブリッド** のgemプロジェクト。

```
rfmt/
├── exe/rfmt                  ← CLIエントリーポイント（Rubyスクリプト）
├── lib/                      ← Ruby側コード
│   └── rfmt/
│       ├── rfmt.bundle       ← コンパイル済みRust拡張（.so/.dylib）
│       ├── cli.rb            ← Thor CLIフレームワーク
│       ├── prism_bridge.rb   ← Prism パーサー統合
│       └── ...
├── ext/rfmt/                 ← Rust側コード
│   ├── Cargo.toml            ← Rust依存関係
│   └── src/
│       ├── lib.rs            ← Magnus FFIエントリーポイント
│       ├── emitter/mod.rs    ← コード生成（最大モジュール）
│       ├── parser/           ← JSON→AST変換
│       └── ...
├── Rakefile                  ← ビルド・テストタスク定義
├── Gemfile                   ← Ruby依存関係
└── rfmt.gemspec
```

### 主要な依存関係

| ライブラリ | 言語 | 役割 |
|-----------|------|------|
| **Prism** | Ruby | Ruby公式パーサー。ソースコード → AST |
| **Magnus** | Rust | Rust ↔ Ruby FFIバインディング |
| **RbSys** | Ruby/Rust | Rustネイティブ拡張のビルドシステム |
| **Thor** | Ruby | CLIフレームワーク |
| **serde_json** | Rust | JSONシリアライズ/デシリアライズ |

### ビルドコマンド

```bash
# Rust拡張のコンパイル（Ruby側の変更には不要）
bundle exec rake compile

# 全テスト実行（Ruby + Rust）
bundle exec rake dev:test_all

# Rubyテストのみ
bundle exec rspec

# Rustテストのみ
bundle exec rake dev:test_rust

# ビルド成果物のクリーン
bundle exec rake dev:clean
```

### 動作確認方法

```bash
# 方法1: ファイルを指定してformat（--no-write で上書きしない）
bundle exec rfmt format --no-write sample.rb

# 方法2: Rubyワンライナーで直接呼ぶ（デバッグ時に便利）
echo 'def hello; end' | bundle exec ruby -e "require 'rfmt'; puts Rfmt.format(STDIN.read)"

# 方法3: IRBで対話的に試す
bundle exec rake console
# irb> Rfmt.format("def hello; end")

# 方法4: checkモード（差分確認のみ、ファイル変更なし）
bundle exec rfmt check sample.rb
```

### デバッグ時のワークフロー

```
┌─────────────────────────────────────────────┐
│  Ruby側 を変更した場合                        │
│  → そのまま実行可能（再コンパイル不要）          │
├─────────────────────────────────────────────┤
│  Rust側 を変更した場合                        │
│  → bundle exec rake compile が必要            │
│  → lib/rfmt/rfmt.bundle が再生成される        │
├─────────────────────────────────────────────┤
│  デバッグ出力の使い分け                        │
│  Ruby側: $stderr.puts "DEBUG: ..."           │
│  Rust側: eprintln!("DEBUG: ...")              │
│  → どちらもstderrに出るため、stdoutの結果と    │
│    混ざらない                                 │
└─────────────────────────────────────────────┘
```

### 環境変数

| 変数 | 効果 |
|------|------|
| `DEBUG=1` | エラー時にバックトレース表示 |
| `RFMT_VERBOSE=1` | Ruby警告を表示 |

---

## Phase一覧

| Phase | 内容 | ファイル | 状態 |
|-------|------|----------|------|
| 1 | エントリーポイントの追跡（CLI → format呼び出し） | `exe/rfmt`, `lib/rfmt.rb` | ⬜ 未着手 |
| 2 | Prismパース結果の確認（Ruby → AST JSON） | `lib/rfmt/prism_bridge.rb` | ⬜ 未着手 |
| 3 | Rust側への橋渡し（JSON → Rust内部AST） | `ext/rfmt/src/lib.rs`, `parser/prism_adapter.rs` | ⬜ 未着手 |
| 4 | Emitterの動作確認（AST → コード生成の全体） | `ext/rfmt/src/emitter/mod.rs` | ⬜ 未着手 |
| 5 | 特定ノードのemit追跡（if/defなど具体的ノード） | `ext/rfmt/src/emitter/mod.rs` | ⬜ 未着手 |

---

## Phase 1: エントリーポイントの追跡

**目的**: CLIから`Rfmt.format(source)`が呼ばれるまでの流れを確認する

**デバッグ方法**: Ruby側に`puts`を追加

**対象ファイル**:
- `exe/rfmt` - コマンド起動
- `lib/rfmt/cli.rb` - CLIコマンド処理
- `lib/rfmt.rb` - `Rfmt.format()`メソッド

**確認ポイント**:
- [ ] CLIでどのサブコマンドが選択されるか
- [ ] ファイル一覧がどう解決されるか
- [ ] `Rfmt.format(source)`に渡されるソースの内容

**学んだこと**:
（Phase完了後に記録）

---

## Phase 2: Prismパース結果の確認

**目的**: Rubyソースコードがどのような構造のAST JSONに変換されるか確認する

**デバッグ方法**: `PrismBridge.parse()`内に`puts`/`pp`を追加

**対象ファイル**:
- `lib/rfmt/prism_bridge.rb` - パース処理とJSON変換

**確認ポイント**:
- [ ] `Prism.parse()`の結果オブジェクトの構造
- [ ] `convert_node()`がどのようにノードを再帰変換するか
- [ ] 最終的なJSON文字列の構造（特にnode_type、children、metadata）
- [ ] コメント情報がどう収集されるか

**学んだこと**:
（Phase完了後に記録）

---

## Phase 3: Rust側への橋渡し

**目的**: Ruby側で生成されたJSON文字列がRust側でどのようにパースされ内部ASTに変換されるか確認する

**デバッグ方法**: Rust側に`eprintln!`マクロを追加 → `bundle exec rake compile` → 実行

**対象ファイル**:
- `ext/rfmt/src/lib.rs` - Rustエントリーポイント
- `ext/rfmt/src/parser/prism_adapter.rs` - JSONパース・AST変換

**確認ポイント**:
- [ ] `format_ruby_code()`に渡される引数
- [ ] `PrismAdapter::parse()`でJSON→Rust構造体への変換
- [ ] Rust側の`Node`構造体の中身

**学んだこと**:
（Phase完了後に記録）

---

## Phase 4: Emitterの動作確認

**目的**: ASTからフォーマット済みコードが生成される全体像を確認する

**デバッグ方法**: `eprintln!`でEmitterの各ステップを追跡

**対象ファイル**:
- `ext/rfmt/src/emitter/mod.rs` - コード生成ロジック

**確認ポイント**:
- [ ] `emit()`メソッドの全体的な流れ
- [ ] コメント収集とインデックス構築
- [ ] `emit_node()`のディスパッチ（どのノードが何の関数に振り分けられるか）
- [ ] bufferにどのタイミングで何が書き込まれるか

**学んだこと**:
（Phase完了後に記録）

---

## Phase 5: 特定ノードのemit追跡

**目的**: 具体的なノード（def、if、文字列補間など）のemit処理を詳しく追跡する

**デバッグ方法**: 各emit_*メソッドに`eprintln!`を追加

**対象ファイル**:
- `ext/rfmt/src/emitter/mod.rs`

**確認ポイント**:
- [ ] `emit_method()`（def文）の処理フロー
- [ ] `emit_if_unless()`（if文）の処理フロー
- [ ] インデントレベルの管理方法
- [ ] ソースコードの部分抽出（offsetの使い方）

**学んだこと**:
（Phase完了後に記録）

---

## 完了条件

- [ ] サンプルRubyコードの「パース → AST変換 → emit」の各段階で、中間データを自分の目で確認できた
- [ ] 各Phaseの「学んだこと」セクションが記入されている
- [ ] フォーマットフロー全体を自分の言葉で説明できる

## 注意事項

- デバッグ用の`puts`/`eprintln!`は学習後に必ず削除する
- Rust側を変更した場合は`bundle exec rake compile`が必要
- デバッグ出力は`stderr`（`eprintln!`）を使うとformatの出力と混ざらない

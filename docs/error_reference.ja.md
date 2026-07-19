# kenshin エラーリファレンス

すべてのエラーコードとその解決方法の完全なリファレンスです。

## エラーコードフォーマット

すべてのkenshinエラーは以下のフォーマットに従います：

```
[Kenshin::ErrorType] エラーメッセージ
追加のコンテキストと詳細

ヘルプ: https://kenshin.dev/errors/EXXX
```

## エラーコード

### E001: ParseError

**タイプ:** `Kenshin::ParseError`

**説明:** フォーマット対象のソースコードにRuby構文エラーがあります。

**よくある原因:**
- `end` キーワードの欠落
- 閉じられていない文字列または括弧
- 無効なRuby構文
- 不正なブロック

**エラー例:**

```
[Kenshin::ParseError] app/models/user.rb:15:10の構文エラー
クラス定義の終了'end'が必要です

コード:
  13 | class User < ApplicationRecord
  14 |   def initialize(name)
  15 |     @name = name
     |          ^
  16 | # メソッドの'end'が不足
  17 | end

ヘルプ: https://kenshin.dev/errors/E001
```

**解決方法:**

1. **フォーマット前に構文エラーを修正:**
   ```ruby
   # 前（無効）
   class User
     def initialize
       @name = name
     # 'end'が不足
   end

   # 後（有効）
   class User
     def initialize
       @name = name
     end
   end
   ```

2. **バランスの取れていないブロックを確認:**
   ```ruby
   # 前（無効）
   users.each do |user|
     puts user.name
   # ブロックの'end'が不足

   # 後（有効）
   users.each do |user|
     puts user.name
   end
   ```

3. **文字列デリミタを確認:**
   ```ruby
   # 前（無効）
   message = "Hello, world

   # 後（有効）
   message = "Hello, world"
   ```

**関連Issue:**
- [#42](https://github.com/sorafujitani/rfmt/issues/42): 構文エラーのエラーメッセージ改善
- [#15](https://github.com/sorafujitani/rfmt/issues/15): ヒアドキュメント構文のサポート

---

### E002: ConfigError

**タイプ:** `Kenshin::ConfigError`

**説明:** 設定ファイル（`.kenshin.yml`）が無効または不正な形式です。

**よくある原因:**
- 無効なYAML構文
- 不明な設定キー
- 設定オプションの無効な値
- 必須フィールドの欠落

**エラー例:**

```
[Kenshin::ConfigError] 設定エラー: 'indent_width'の値が無効です
ファイル: .kenshin.yml

提案: 正の整数値を使用してください（例: 2, 4）

ヘルプ: https://kenshin.dev/errors/E002
```

**解決方法:**

1. **YAML構文を検証:**
   ```yaml
   # 前（無効）
   formatting
     indent_width: 2

   # 後（有効）
   formatting:
     indent_width: 2
   ```

2. **正しいデータ型を使用:**
   ```yaml
   # 前（無効）
   formatting:
     indent_width: "2"  # 整数ではなく文字列
     line_length: two   # 無効な値

   # 後（有効）
   formatting:
     indent_width: 2
     line_length: 100
   ```

3. **設定キーを確認:**
   ```yaml
   # 前（無効）
   formatting:
     indentation: 2      # 誤ったキー名

   # 後（有効）
   formatting:
     indent_width: 2
   ```

4. **列挙値を確認:**
   ```yaml
   # 前（無効）
   formatting:
     indent_style: "space"  # "spaces"であるべき

   # 後（有効）
   formatting:
     indent_style: "spaces"
   ```

**有効な設定スキーマ:**

```yaml
version: "1.0"

formatting:
  line_length: 100        # 整数（1-500）
  indent_width: 2         # 整数（1-8）
  indent_style: "spaces"  # "spaces" または "tabs"
  quote_style: "double"   # "double" または "single"

include:                  # Globパターンの配列
  - "**/*.rb"

exclude:                  # Globパターンの配列
  - "vendor/**/*"
```

**関連Issue:**
- [#23](https://github.com/sorafujitani/rfmt/issues/23): 設定エラーのエラーメッセージ改善

---

### E003: IoError

**タイプ:** `Kenshin::IOError`

**説明:** ファイルシステム操作が失敗しました（読み取り、書き込み、アクセス）。

**よくある原因:**
- ファイルが存在しない
- 権限不足
- 他のプロセスによるファイルロック
- ディスクの空き容量不足
- ネットワークドライブが利用不可

**エラー例:**

```
[Kenshin::IOError] app/models/user.rbのIOエラー: 権限が拒否されました

ヘルプ: https://kenshin.dev/errors/E003
```

**解決方法:**

1. **ファイルの存在を確認:**
   ```bash
   ls -la app/models/user.rb
   ```

2. **権限を確認:**
   ```bash
   # 読み取り権限
   chmod u+r file.rb

   # 書き込み権限（インプレースフォーマット用）
   chmod u+w file.rb
   ```

3. **ディスク容量を確認:**
   ```bash
   df -h .
   ```

4. **他のプログラムでファイルを閉じる:**
   - ファイルを開いているエディタを閉じる
   - `lsof`でバックグラウンドプロセスを確認

5. **sudoを使用（適切な場合）:**
   ```bash
   sudo kenshin format system_file.rb
   ```

**関連Issue:**
- [#31](https://github.com/sorafujitani/rfmt/issues/31): ロックされたファイルのエラー回復改善

---

### E004: FormattingError

**タイプ:** `Kenshin::FormattingError`

**説明:** フォーマット処理中にエラーが発生しました。

**よくある原因:**
- 内部フォーマッターのバグ
- サポートされていないRuby構文のエッジケース
- 破損したAST
- 非常に大きなファイルでのメモリ不足

**エラー例:**

```
[Kenshin::FormattingError] フォーマットエラー: ノードの出力に失敗
ノードタイプ: def_node
位置: 42:15

ヘルプ: https://kenshin.dev/errors/E004
```

**解決方法:**

1. **より単純なバージョンでフォーマットを試す:**
   - 複雑なコードをコメントアウト
   - ネストした構造を簡略化
   - 小さなチャンクでフォーマット

2. **kenshinを更新:**
   ```bash
   gem update kenshin
   ```

3. **問題を報告:**
   これはバグの可能性があります。以下の情報で報告してください：
   - Rubyコード（または最小限の再現）
   - kenshinバージョン（`kenshin --version`）
   - Rubyバージョン（`ruby -v`）
   - エラーメッセージ

4. **部分的なフォーマットの回避策:**
   ```bash
   # ファイル全体ではなく個別のメソッドをフォーマット
   kenshin format app/models/user.rb:10-50
   ```

**関連Issue:**
- [#55](https://github.com/sorafujitani/rfmt/issues/55): 複雑なネストブロックの処理

---

### E005: RuleError

**タイプ:** `Kenshin::RuleError`

**説明:** フォーマットルールの適用に失敗しました。

**よくある原因:**
- 競合するフォーマットルール
- ルールの前提条件が満たされていない
- ルール実装のバグ

**エラー例:**

```
[Kenshin::RuleError] ルール適用エラー: ルール'IndentationRule'が失敗しました
孤立したノードのインデントレベルを決定できません

ヘルプ: https://kenshin.dev/errors/E005
```

**解決方法:**

1. **まず構文エラーを確認:**
   `ruby -c file.rb` でコードが正しく解析されることを確認

2. **コード構造を簡略化:**
   複雑なネスト構造がフォーマッターを混乱させる可能性があります

3. **kenshinを更新:**
   ```bash
   gem update kenshin
   ```

4. **問題を報告:**
   これはフォーマットルールのバグの可能性があります

**関連Issue:**
- [#67](https://github.com/sorafujitani/rfmt/issues/67): ルール競合の解決

---

### E006: UnsupportedFeature

**タイプ:** `Kenshin::UnsupportedFeature`

**説明:** コードがkenshinでまだサポートされていないRuby機能を使用しています。

**よくある原因:**
- 実験的なRuby構文
- Ruby 3.4+の機能（古いkenshinを使用している場合）
- 言語機能のエッジケース

**エラー例:**

```
[Kenshin::UnsupportedFeature] 未サポート機能: ピンニング演算子を使用したパターンマッチング

この機能は将来のリリースで予定されています。
追跡: https://github.com/sorafujitani/rfmt/issues/89

ヘルプ: https://kenshin.dev/errors/E006
```

**解決方法:**

1. **ロードマップを確認:**
   機能が計画されているか確認: [ROADMAP.md](../ROADMAP.md)

2. **代替構文を使用:**
   可能であれば、サポートされている機能で書き直す

3. **そのセクションのフォーマットをスキップ:**
   ```ruby
   # kenshin:disable
   case value
   in ^expected_value
     puts "matched"
   end
   # kenshin:enable
   ```

4. **機能をリクエスト:**
   以下の情報でissueを作成：
   - 機能を使用したコード例
   - ユースケースの説明
   - それが有効なRubyバージョン

**現在未サポートの機能:**
- 番号付きブロックパラメータ（`_1`、`_2`）
- 一部のRuby 3.3+構文機能
- 複雑なパターンマッチングのエッジケース

**関連Issue:**
- [#89](https://github.com/sorafujitani/rfmt/issues/89): パターンマッチングサポート
- [#102](https://github.com/sorafujitani/rfmt/issues/102): 番号付きパラメータ

---

### E007: PrismError

**タイプ:** `Kenshin::PrismError`

**説明:** 組み込みprismパーサー統合のエラーです。パースは Rust 拡張の内部で静的リンクされた ruby-prism crate により行われ、prism gem は実行時には使われません。

**よくある原因:**
- 内部パーサーエラー
- 予期しないAST構造

**エラー例:**

```
[Kenshin::PrismError] Prism統合エラー: 予期しないノード構造

ヘルプ: https://kenshin.dev/errors/E007
```

**解決方法:**

1. **kenshinを更新:**
   ```bash
   gem update kenshin
   ```

2. **拡張を再ビルド（ソースインストールの場合）:**
   ```bash
   bundle exec rake clean
   bundle exec rake compile
   ```

3. **問題を報告:**
   これは内部エラーです。以下の情報で報告してください：
   - kenshinバージョン
   - エラーをトリガーするコード

**関連Issue:**
- [#118](https://github.com/sorafujitani/rfmt/issues/118): Prism 1.0互換性

---

### E008: FormatError

**タイプ:** `Kenshin::FormatError`

**説明:** 一般的なフォーマットエラー（包括的）。

**よくある原因:**
- 様々なフォーマット失敗
- 他のエラーでカバーされないエッジケース

**エラー例:**

```
[Kenshin::FormatError] フォーマットエラー: 出力中のバッファオーバーフロー

ヘルプ: https://kenshin.dev/errors/E008
```

**解決方法:**

1. **コードを簡略化:**
   複雑な構造を分解

2. **ファイルサイズを確認:**
   ```bash
   wc -l file.rb  # 非常に大きなファイルは問題を起こす可能性があります
   ```

3. **kenshinを更新:**
   ```bash
   gem update kenshin
   ```

4. **問題を報告:**
   完全なエラーメッセージとコードサンプルを含めてください

---

### E999: InternalError

**タイプ:** `Kenshin::InternalError`

**説明:** kenshinの内部バグです。これは決して起こらないはずです！

**よくある原因:**
- 未処理のエッジケース
- kenshinコードのバグ
- メモリ破損
- プラットフォーム固有の問題

**エラー例:**

```
[Kenshin::InternalError] 内部エラー: ASTトラバーサル中の予期しないnullポインタ

バックトレース:
  at /path/to/kenshin/src/emitter.rs:123
  at /path/to/kenshin/src/formatter.rs:456

これをバグとして報告してください: https://github.com/sorafujitani/rfmt/issues

ヘルプ: https://kenshin.dev/errors/E999
```

**解決方法:**

1. **すぐに報告:**
   これはバグです！以下の情報でissueを作成してください：
   - バックトレースを含む完全なエラーメッセージ
   - エラーをトリガーするコード（または最小限の再現）
   - kenshinバージョン（`kenshin --version`）
   - Rubyバージョン（`ruby -v`）
   - プラットフォーム（OSとアーキテクチャ）

2. **回避策:**
   - 問題のあるファイルを一時的にスキップ
   - 小さなチャンクでフォーマット
   - これがリグレッションの場合は古いバージョンを使用

3. **デバッグ情報を収集:**
   ```bash
   RUST_BACKTRACE=1 kenshin format file.rb 2> error.log
   ```

**関連Issue:**
- [#new](https://github.com/sorafujitani/rfmt/issues/new): 新しいバグを報告

---

## デバッグのヒント

### 詳細出力を有効化

```bash
kenshin format --verbose file.rb
```

### Rustバックトレースを確認

```bash
RUST_BACKTRACE=1 kenshin format file.rb
```

### デバッグロギングを有効化

```ruby
# kenshinをrequireする前にログレベルを設定
ENV['KENSHIN_LOG_LEVEL'] = 'debug'
require 'kenshin'
```

### デバッグ情報を取得

```ruby
require 'kenshin'

# バージョンとプラットフォーム情報を出力
puts Kenshin.rust_version
```

## ヘルプを得る

ここでカバーされていないエラーに遭遇した場合：

1. **既存のissueを検索:** https://github.com/sorafujitani/rfmt/issues
2. **ディスカッションを確認:** https://github.com/sorafujitani/rfmt/discussions
3. **新しいissueを作成:** https://github.com/sorafujitani/rfmt/issues/new

問題を報告する際は、以下を含めてください：

- エラーコードと完全なメッセージ
- kenshinバージョン（`kenshin --version`）
- Rubyバージョン（`ruby -v`）
- コードサンプル（最小限の再現）
- 設定ファイル（`.kenshin.yml`）
- プラットフォーム（OSとアーキテクチャ）

## 関連ドキュメント

- [ユーザーガイド](user_guide.ja.md)
- [APIドキュメント](api_documentation.md)
- [コントリビューティングガイド](../CONTRIBUTING.md)
- [トラブルシューティング](user_guide.ja.md#トラブルシューティング)

---

**バージョン:** 0.1.0
**最終更新:** 2025-11-24
**ライセンス:** MIT

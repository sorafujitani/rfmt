# リリース手順

このドキュメントでは、rfmtの新しいバージョンをリリースする手順を説明します。

## 自動リリース（推奨）

GitHub Actionsを使用した自動リリースが可能です。

### 手順

1. **バージョンの更新**

   以下のファイルでバージョン番号を更新します：
   - `lib/rfmt/version.rb` - Rubyのバージョン定義
   - `ext/rfmt/Cargo.toml` - Rustクレートのバージョン
   - `Cargo.lock` - 依存関係のロック（`cargo build`で自動更新）

2. **CHANGELOG.mdの更新**

   リリースノートをCHANGELOG.mdに追加します：
   ```markdown
   ## [X.Y.Z] - YYYY-MM-DD

   ### Added
   - 新機能の説明

   ### Changed
   - 変更内容の説明

   ### Fixed
   - バグ修正の説明
   ```

3. **変更のコミットとプッシュ**
   ```bash
   git add lib/rfmt/version.rb ext/rfmt/Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "Bump version to X.Y.Z"
   git push origin main
   ```

4. **GitタグとGitHub Releaseの作成**
   ```bash
   # タグを作成
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

   または、GitHubのWebインターフェースで：
   - [Releases](https://github.com/your-username/rfmt/releases)ページに移動
   - "Draft a new release"をクリック
   - タグ：`vX.Y.Z`（新規作成）
   - リリースタイトル：`vX.Y.Z`
   - 説明：CHANGELOG.mdの内容をコピー
   - "Publish release"をクリック

5. **自動ビルドの確認**

   GitHub Actionsが自動的に以下を実行します：
   - Linux、macOS、Windows向けのネイティブgemをビルド
   - RubyGems.orgへの公開（credentials設定済みの場合）

   進行状況は[Actions](https://github.com/your-username/rfmt/actions)タブで確認できます。

## 手動リリース

自動リリースが利用できない場合や、手動でリリースしたい場合の手順です。

### 前提条件

- RubyGems.orgのアカウントとAPIキー
- 各プラットフォーム（Linux、macOS、Windows）でのビルド環境
- `gem`コマンドがインストールされていること

### 手順

1. **バージョンの更新**

   自動リリースと同じ手順でバージョンを更新します。

2. **CHANGELOG.mdの更新**

   自動リリースと同じ手順でCHANGELOG.mdを更新します。

3. **各プラットフォームでgemをビルド**

   #### Linux (x86_64)
   ```bash
   # Linux環境で実行
   rake native:x86_64-linux:gem
   ```

   #### macOS (ARM64)
   ```bash
   # Apple Silicon Macで実行
   rake native:arm64-darwin:gem
   ```

   #### macOS (x86_64)
   ```bash
   # Intel Macで実行
   rake native:x86_64-darwin:gem
   ```

   #### Windows (x64)
   ```bash
   # Windows環境で実行
   rake native:x64-mingw-ucrt:gem
   ```

4. **gemの公開**

   各プラットフォームで生成されたgemファイルを公開：
   ```bash
   # RubyGems.orgにログイン（初回のみ）
   gem signin

   # 各gemを公開
   gem push pkg/rfmt-X.Y.Z-x86_64-linux.gem
   gem push pkg/rfmt-X.Y.Z-arm64-darwin.gem
   gem push pkg/rfmt-X.Y.Z-x86_64-darwin.gem
   gem push pkg/rfmt-X.Y.Z-x64-mingw-ucrt.gem
   ```

5. **Gitタグの作成**
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

6. **GitHub Releaseの作成**

   [Releases](https://github.com/your-username/rfmt/releases)ページから：
   - "Draft a new release"をクリック
   - タグ：`vX.Y.Z`（既存のタグを選択）
   - リリースタイトル：`vX.Y.Z`
   - 説明：CHANGELOG.mdの内容をコピー
   - ビルドしたgemファイルを添付（オプション）
   - "Publish release"をクリック

## リリース後の確認

1. **RubyGems.orgでの確認**

   https://rubygems.org/gems/rfmt で新しいバージョンが表示されることを確認

2. **インストールテスト**
   ```bash
   gem install rfmt
   rfmt --version
   ```

3. **動作確認**
   ```bash
   echo "def hello; puts 'world'; end" | rfmt
   ```

## トラブルシューティング

### GitHub Actionsのビルドが失敗する

- [Actions](https://github.com/your-username/rfmt/actions)タブでログを確認
- Rust toolchainのバージョンを確認
- 依存関係の問題がないか確認

### RubyGems.orgへの公開が失敗する

- APIキーが正しく設定されているか確認
- リポジトリのSecretsで`RUBYGEMS_API_KEY`が設定されているか確認
- バージョン番号が既存のバージョンと重複していないか確認

### クロスプラットフォームビルドの問題

- 各プラットフォームでRust toolchainが正しくインストールされているか確認
- `rb-sys`の依存関係が満たされているか確認
- プラットフォーム固有のビルドツール（MinGW、Xcodeなど）が利用可能か確認

## 参考資料

- [RubyGems Guides](https://guides.rubygems.org/)
- [GitHub Actions Documentation](https://docs.github.com/actions)
- [rb-sys Documentation](https://github.com/oxidize-rb/rb-sys)

# Release: New Major Version

メジャーバージョン（X.Y.Z の X を +1、Y と Z を 0 にリセット）をリリースする準備を行います。
詳細は RELEASE.ja.md を参照してください。

## タスク

以下の手順を自動的に実行してください：

1. **現在のバージョンを確認**
   - `lib/rfmt/version.rb`から現在のバージョンを読み取る

2. **新しいメジャーバージョンを計算**
   - 現在のバージョンが`X.Y.Z`の場合、新バージョンは`(X+1).0.0`

3. **バージョンファイルを更新**
   - `lib/rfmt/version.rb`のVERSIONを更新
   - `ext/rfmt/Cargo.toml`のversionを更新
   - `cargo update --manifest-path ext/rfmt/Cargo.toml --workspace`を実行してCargo.lockを更新
   - `bundle install`を実行してGemfile.lockを更新（バージョン不整合を解消）

4. **CHANGELOG.mdを更新**
   - 新しいバージョンのセクションを`## [Unreleased]`の下に追加
   - 現在の日付を使用（フォーマット: YYYY-MM-DD）
   - **リリース内容を自動生成**：
     - `git log origin/main..HEAD --oneline`でremoteにpush済みの差分コミットを取得
     - コミットメッセージから変更内容を分類して、セクション構造を作成：
       - `BREAKING`や`breaking:`などを含むコミット → `### Breaking Changes`
       - `feat:`や`Added`などで始まるコミット → `### Added`
       - `chore:`や`Changed`などで始まるコミット → `### Changed`
       - `remove:`や`Removed`などで始まるコミット → `### Removed`
       - `fix:`や`Fixed`などで始まるコミット → `### Fixed`
     - 分類されないコミットは`### Changed`に含める
     ```markdown
     ## [X.0.0] - YYYY-MM-DD

     ### Breaking Changes
     - (git diffから自動生成)

     ### Added
     - (git diffから自動生成)

     ### Changed
     - (git diffから自動生成)

     ### Removed
     - (git diffから自動生成)

     ### Fixed
     - (git diffから自動生成)
     ```

5. **変更内容をユーザーに報告**
   - 更新したファイルとバージョン番号を表示
   - **重要な警告**：メジャーバージョンアップは破壊的変更を含むことをユーザーに明示
   - 次のステップを案内：
     ```bash
     # 変更をコミット
     git add lib/rfmt/version.rb ext/rfmt/Cargo.toml Cargo.lock Gemfile.lock CHANGELOG.md
     git commit -m "Bump version to X.0.0"
     git push origin main

     # タグを作成してリリース（GitHub Actionsが自動ビルド・公開）
     git tag vX.0.0
     git push origin vX.0.0
     ```

## 注意事項

- **メジャーバージョンは後方互換性を破る変更を含む**
- Breaking Changes セクションに互換性を破る変更を明確に記載する必要がある
- バージョン更新のみを行い、実際のコミット・タグ作成はユーザーが行う
- CHANGELOG.mdの内容はgit diffから自動生成されるが、必ずユーザーに確認する
- bundle installでGemfile.lockを更新することで、バージョン不整合エラーを回避する
- 削除された機能は Removed セクションに記載する
- マイグレーションガイドの作成を検討する
- **前提**: 差分がすでにremoteにpush済みであること（`git log origin/main..HEAD`で確認）

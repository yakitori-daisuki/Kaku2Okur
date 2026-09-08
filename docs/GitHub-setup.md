# Folder-local GitHub account / このフォルダだけの GitHub 設定

Repository / リポジトリ: [yakitori-daisuki/Kaku2Okur](https://github.com/yakitori-daisuki/Kaku2Okur)

The repository identity and HTTPS origin are configured locally for `yakitori-daisuki`. The URL is also used by both READMEs and the release downloader. Setting an origin URL does not create the remote repository.

このフォルダの作者情報と HTTPS の origin は `yakitori-daisuki` 用に設定しています。README と Release ダウンローダーも上記 URL を使用します。origin の設定と GitHub 上のリポジトリ作成は別の操作です。

## Sign in / ログイン

Run from the repository root / このフォルダで実行:

```bash
./script/github.sh auth login --hostname github.com --git-protocol https --web
./script/setup_github.sh yakitori-daisuki
```

The setup command checks the authenticated account before changing this repository's author and credential helper. It uses the account's GitHub noreply email.

setup コマンドは認証中のアカウントが一致することを確認してから、このリポジトリだけの作者情報と認証ヘルパーを設定します。メールには GitHub の noreply アドレスを使います。

- CLI account selection: `.github-local/gh` via `GH_CONFIG_DIR`.
- Git commands run by the wrapper: `.github-local/gitconfig` via `GIT_CONFIG_GLOBAL`.
- Normal `git push`: the credential helper in this checkout's `.git/config` calls `script/github.sh`.
- Existing token environment variables are unset within the wrapper.
- Authentication files are ignored by Git, and their directories have mode 700.
- No normal global Git config, global CLI account selection, or SSH config is changed.
- GitHub CLI may store tokens in the OS credential store. Account selection is folder-local; server-side token permissions are not restricted by a local folder.

通常のグローバル Git 設定・CLI のアカウント選択・SSH 設定は変更しません。認証設定は Git 管理対象外の `.github-local` に分離します。トークン本体は OS の認証情報ストアに保存される場合があります。この仕組みはアカウントの選択をこのフォルダに限定するもので、トークンそのものを他のフォルダで使えなくする機能ではありません。

## Create and push / 作成・push

After authentication, create the public repository and push the prepared local commit. These commands assume that `origin` is already configured as above.

認証後、公開リポジトリを作成して、準備済みのローカルコミットを push します。origin は設定済みの前提です。

```bash
./script/github.sh repo create yakitori-daisuki/Kaku2Okur --public \
  --description "A lightweight native sketch-to-paste utility for macOS and Windows"
git push -u origin main
```

If the repository already exists, skip the creation command. Use the same folder-local CLI wrapper for future release operations.

すでにリポジトリが存在する場合は作成コマンドを省略します。以後の Release 操作も、このフォルダの `script/github.sh` を使ってください。

## Release assets / Release のアセット

```bash
./script/build_unsigned_installer.sh
```

Upload the matching installer and its `.sha256` to a non-draft, non-prerelease GitHub Release after completing [distribution checks](Distribution.md). The README downloader references `releases/latest/download`. The architecture-specific filenames must remain unchanged.

[配布確認](Distribution.md)を終えてから、インストーラと対応する `.sha256` を正式版の GitHub Release に公開します。README は `releases/latest/download` を参照するため、CPU ごとのアセット名を変更しないでください。

# Distribution / 配布メモ

## Build / 作成

```bash
./script/build_and_run.sh --test
./script/build_unsigned_installer.sh
```

The script builds for the current Mac's architecture, verifies the app signature, embeds its ZIP and SHA-256 in the installer, then writes an external checksum. Build Apple Silicon and Intel artifacts on their respective architectures. No universal binary is produced.

現在の Mac の CPU 向けにアプリを作成して署名を検証し、ZIP と SHA-256 をインストーラに埋め込み、外部チェックサムも生成します。Apple Silicon 版と Intel 版はそれぞれの環境で作成します。

## Verify / 検証

For Apple Silicon / Apple Silicon の場合:

```bash
cd dist
shasum -a 256 -c Kaku2Okur-install-unsigned-arm64.sh.sha256
bash Kaku2Okur-install-unsigned-arm64.sh --verify-only
```

Verification checks the embedded ZIP checksum, bundle ID `app.kaku2okur.native`, architecture, and code signature. It does not install, launch, or remove quarantine attributes.

検証のみでは、内蔵 ZIP のチェックサム、Bundle ID、CPU、コード署名を確認し、配置・起動・quarantine 属性の解除は行いません。

To test installation in an isolated folder after quitting Kaku2Okur / アプリを終了し、隔離したフォルダに配置して確認する場合:

```bash
bash dist/Kaku2Okur-install-unsigned-arm64.sh \
  --prefix "$PWD/target/installer-check" --allow-unsigned --no-open
```

Run this command from the repository root. Repeat it to verify backup behavior. The installer uses a staging directory next to the destination and preserves an existing installation in a dated backup directory. It does not request sudo.

このコマンドはリポジトリのルートで実行します。2回実行すると既存アプリのバックアップも検証できます。配置先の隣に一時領域を作成してから配置し、以前のアプリを日時付きフォルダへ退避します。sudo は要求しません。

## Before a binary release / バイナリ公開前

The fixed release destination is `https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download`. Both READMEs contain the same download-and-verify command as `script/install_from_release.sh`. After release assets are published, the command detects the Mac's architecture and selects `Kaku2Okur-install-unsigned-arm64.sh` or `Kaku2Okur-install-unsigned-x86_64.sh` and its checksum.

Release の公開先は上記 URL に固定しています。日英 README の1回コピペ用コマンドと `script/install_from_release.sh` は同じ処理です。対応する CPU のインストーラとチェックサムを Release に公開すると利用できます。

```bash
python3 script/test_release_download.py
```

These tests use local fixtures and never download or install an app. 実際のダウンロードやインストールを行わず、ローカルの検証用ファイルで確認します。

- Confirm launch, Input Monitoring, invocation, Japanese input, and paste dispatch on each supported target OS and architecture.
- Check the minimum runtime version of the compiled executable and linked libraries on the oldest supported OS.
- Review dependency license/notice requirements for the exact locked build and select a license for original project code.
- Keep `THIRD_PARTY_NOTICES.md` and `Lucide-LICENSE.txt` with the app.
- Publish both the architecture-specific installer and its `.sha256`, and include the checksum in release notes.
- Clearly describe the build as ad-hoc signed and unnotarized. Do not claim Developer ID identity or Apple notarization.
- Update both READMEs' “not published yet” text only after the assets exist.

対象 OS・CPU ごとの起動、入力監視、呼び出し、日本語入力、貼り付けを実機確認してください。最小対応 OS での動作、固定した依存関係のライセンス・通知も確認し、独自コードのライセンスを選定します。インストーラとチェックサムを両方公開し、未公証であることを明記します。README の「未公開」は、実際のアセット公開後に日英とも更新してください。

## Trust model / 署名の扱い

The ad-hoc signature checks code integrity, not publisher identity. A checksum downloaded alongside an installer detects corruption, but does not protect against compromise of the release account. `--allow-unsigned` explicitly allows quarantine removal only on the newly staged app, after payload and signature verification. No Gatekeeper setting is changed.

ad-hoc 署名は配布者の本人性を証明しません。同じ配布先から取得するチェックサムも、配布アカウントの侵害に対する独立した証明にはなりません。`--allow-unsigned` による quarantine 属性の解除は、検証後の新しいアプリだけが対象です。Gatekeeper の設定は変更しません。

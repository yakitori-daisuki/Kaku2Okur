# Kaku2Okur

[English](README.md) | **日本語**

<img src="apps/native/assets/app-icon/envelope-preview.png" alt="Kaku2Okur のアプリアイコン" width="96">

**描いて、送る。** Kaku2Okur は、ちょっとした図やアイデアを描いて、直前まで使っていたアプリへ画像として渡す軽量デスクトップツールです。

「このボタンを目立たせたい」「この位置に置きたい」「こんな形にしたい」。文章だけでは伝えにくいことを、その場で図にして共有できます。作業中にキャンバスを呼び出し、ペンや図形、文字で説明を添えたら、元のチャットやエディタへ貼り付けます。

## こんなときに

- **画面の修正を伝える**：レイアウトをざっと描き、変更したい場所を矢印で示す。
- **位置や形を説明する**：配置、向き、大きさのイメージを図にする。
- **チャットで相談する**：画像を受け取れる相手や AI の入力欄に、文章と一緒に図を添える。

## 実際の画面

![macOS 上の Kaku2Okur で、長方形・文字・矢印を使って設定画面の改善案を描いた画面](docs/images/canvas-example.png)

この画像は、実際の Kaku2Okur で「設定画面の改善案」を描いているところです。

- **長方形**で画面の外枠、左側のメニュー、保存ボタンを描いています。
- **テキスト**で見出しや項目名を入れています。
- **矢印**で保存ボタンを指し、「ここを目立たせたい」と伝えています。画像内の「Make this stand out」は、この指示を英語で書いたものです。
- **上部のツールバー**から描画ツールを切り替え、右端の青い紙飛行機ボタンで画像を送ります。

スケッチの中にある設定画面は説明用に描いた作例です。[撮影情報](docs/images/README.md)。

## 使い方

1. チャットやエディタなど、画像を受け取れるアプリの入力欄にカーソルを置きます。
2. macOS では **左右の Option**、Windows では **左右の Alt** を同時に押します。
3. ペン・矢印・長方形・楕円・消しゴム・テキストで描きます。
4. **Enter** または **Send** で描いた部分を切り出し、元のアプリへ戻って画像の貼り付けを要求します。

同じ呼び出し操作でもう一度押すと、一時的にキャンバスを隠せます。再び呼び出すと、描きかけの内容から再開します。画像を受け取れるかどうかは貼り付け先アプリによります。貼り付け操作を実行できないときは画像をクリップボードに残すため、手動で貼り付けられます。

「送る」は、描いた図を元のアプリへ画像として貼り付ける操作です。チャットのメッセージ送信は、貼り付け先で内容を確認してから行ってください。

### 日本語で文字を入れる

文字を置きたい場所にポインタを合わせ、そのまま入力を始めます。日本語の変換中は **Enter** で変換を確定でき、文字入力中の Enter で図が送られることはありません。

入力が終わったら、macOS では **Command + Enter**、Windows では **Ctrl + Enter**、または **Done（入力完了）** を押します。配置した文字はドラッグで動かせます。図全体を送りたいときは、文字入力を終えてからキャンバスで Enter を押してください。

## 主な機能

- **ポインタの位置から文字入力。** ツールを切り替えずに入力を始められ、日本語 IME の未確定文字・変換にも対応します。配置した文字はドラッグして移動できます。
- **Hold to Tidy。** ペンで描いた最後に少し止めると、認識できた直線・長方形・楕円を、編集可能なきれいな図形に整えます。
- **送る前に調整。** オブジェクトの移動・サイズ変更・削除、元に戻す・やり直すができます。
- **作業環境に合わせる。** 明るい背景・暗い背景・任意の色を選べ、ディスプレイごとにウィンドウ位置を復元します。
- **Trackpad Sketch Mode（macOS・ベータ）。** 対応トラックパッド全体をキャンバスに対応させ、四隅と各辺の中央の計8か所にツールや操作を割り当てられます。
- **ローカルで軽快に。** Rust・Slint のソフトウェア描画・tiny-skia で動作し、基本操作にブラウザエンジン、JavaScript 実行環境、アカウント、クラウドサービスは不要です。スケッチを永続的なライブラリとして保存しません。

### キーボード操作

| 操作 | macOS | Windows |
| --- | --- | --- |
| 呼び出し／一時的に隠す | 左右の Option | 左右の Alt |
| キャンバスから送る | Enter | Enter |
| 送らずに文字入力を完了 | Command + Enter | Ctrl + Enter |
| 元に戻す | Command + Z | Ctrl + Z |
| やり直す | Command + Shift + Z / Command + Y | Ctrl + Shift + Z / Ctrl + Y |
| 選択中のオブジェクトを削除 | Delete / Backspace | Delete / Backspace |
| 編集取消 → 選択解除 → スケッチ破棄 | Escape | Escape |
| スケッチを破棄して閉じる | Command + W | Ctrl + W |

文字入力中の Enter は IME の変換確定または改行になり、スケッチは送りません。**Done** または文字入力欄の外側をクリックしても入力を完了できます。左右の修飾キーがないキーボード向けの操作も含め、呼び出し方法は Settings から変更できます。

## 動作・ビルド条件

| 対象 | 条件 |
| --- | --- |
| macOS | macOS 14 以降。ビルドには **Rust 1.90 以降**と**フル版 Xcode 26 以降**が必要です。レイヤー付きアイコンの作成に `actool` を使うため、Command Line Tools 単体では不足します。 |
| Windows | Rust 1.90 以降の **MSVC** ツールチェーン、Visual Studio Build Tools の **C++ によるデスクトップ開発**と Windows SDK。 |

macOS を主な対象としています。Windows 用の実装とビルドスクリプトも含みますが、今回の公開準備時の確認環境は Apple Silicon Mac です。Windows と Intel Mac での実機動作は別途確認が必要です。トラックパッドを直接使う Trackpad Sketch Mode は現在 macOS のみです。

## 自己完結型の未公証版をインストール

> [!IMPORTANT]
> 以下は `yakitori-daisuki/Kaku2Okur` の初回リリース用に準備した手順です。[GitHub Releases](https://github.com/yakitori-daisuki/Kaku2Okur/releases) にアセットが公開されるまでは利用できません。

受け取る人に Rust・Xcode・Homebrew・Apple 開発者アカウントは不要です。macOS 標準ツールだけでインストールできます。

### 1回のコピペでインストール

> [!WARNING]
> Developer ID 署名・公証のないビルドを導入します。`--allow-unsigned` は、新しく配置するアプリの quarantine 属性を解除する明示的な許可です。このリポジトリを信頼できる場合だけ実行してください。

次のブロック全体をターミナルに貼り付けます。Mac の CPU を判定し、対応するインストーラと SHA-256 を取得して、検証に成功した場合だけ実行します。

```bash
/bin/bash -c '
set -euo pipefail
case "${1:-}" in
  --allow-unsigned|--verify-only) mode="$1" ;;
  *) echo "usage: $0 --allow-unsigned | --verify-only" >&2; exit 2 ;;
esac
[[ $# -eq 1 ]] || { echo "error: unexpected arguments" >&2; exit 2; }
[[ "$(uname -s)" == Darwin ]] || { echo "error: macOS is required" >&2; exit 1; }
arch="$(uname -m)"
case "$arch" in arm64|x86_64) ;; *) echo "error: unsupported architecture: $arch" >&2; exit 1 ;; esac
umask 077
base="https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download"
name="Kaku2Okur-install-unsigned-$arch.sh"
temp_root="${TMPDIR:-/tmp}"
work="$(/usr/bin/mktemp -d "${temp_root%/}/kaku2okur-download.XXXXXX")"
cleanup() { /bin/rm -rf "$work"; }
trap cleanup EXIT
trap "exit 130" HUP INT TERM
cd "$work"
fetch() {
  /usr/bin/curl --disable --fail --silent --show-error --location \
    --max-redirs 10 --proto "=https" --proto-redir "=https" \
    --connect-timeout 20 --max-time 600 --retry 3 --output "$1" "$2"
}
fetch "$name" "$base/$name"
fetch "$name.sha256" "$base/$name.sha256"
read -r expected listed extra < "$name.sha256"
[[ -z "${extra:-}" && "$listed" == "$name" && ${#expected} -eq 64 && "$expected" != *[!0-9a-f]* ]] || { echo "error: invalid checksum manifest" >&2; exit 1; }
[[ "$(/usr/bin/awk "END { print NR }" "$name.sha256")" == "1" ]] || { echo "error: checksum manifest must contain exactly one line" >&2; exit 1; }
/usr/bin/shasum -a 256 -c "$name.sha256"
if [[ -d /Applications && -w /Applications ]]; then scope="--system"; else scope="--user"; fi
/bin/bash "$name" "$scope" "$mode"
' -- --allow-unsigned
```

書き込み可能なら `/Applications`、そうでなければ `~/Applications` に配置し、最後に一時ファイルを削除します。すでに Kaku2Okur が起動している場合は終了してから実行してください。既存アプリはインストーラがバックアップします。

チェックサムも同じ Release から取得するため、破損や取り違えは検出できますが、配布アカウント自体が侵害された場合の独立した本人性確認にはなりません。

### 手動でダウンロードする場合

| Mac | インストーラ | チェックサム |
| --- | --- | --- |
| Apple Silicon | [arm64 版](https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download/Kaku2Okur-install-unsigned-arm64.sh) | [SHA-256](https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download/Kaku2Okur-install-unsigned-arm64.sh.sha256) |
| Intel | [x86_64 版](https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download/Kaku2Okur-install-unsigned-x86_64.sh) | [SHA-256](https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download/Kaku2Okur-install-unsigned-x86_64.sh.sha256) |

各リンクは、その CPU 向けアセットが公開された後に利用できます。初回のビルド・検証済み対象は Apple Silicon です。2ファイルを同じフォルダへ保存し、Apple Silicon では次の順で実行します。Intel では `arm64` を `x86_64` に置き換えてください。

```bash
cd ~/Downloads
shasum -a 256 -c Kaku2Okur-install-unsigned-arm64.sh.sha256
```

`OK` と表示された場合だけ、内容を確認して実行します。

```bash
less Kaku2Okur-install-unsigned-arm64.sh
bash Kaku2Okur-install-unsigned-arm64.sh --user --allow-unsigned
```

`--user` は `~/Applications`、`--system` は書き込み可能な `/Applications` に配置します。`--no-open` を追加すると起動を省略でき、`--verify-only` なら配置せずに検証だけ行えます。

## ソースからビルド・起動

```bash
git clone https://github.com/yakitori-daisuki/Kaku2Okur.git
cd Kaku2Okur
```

このリポジトリの **Code → Download ZIP** からソースを取得して展開するか、Git で clone し、ターミナルでリポジトリのルートフォルダを開きます。

### macOS

[Rust](https://rustup.rs/) とフル版 [Xcode](https://developer.apple.com/xcode/) を導入し、Xcode を一度起動して初期設定を済ませてください。`xcode-select -p` が Command Line Tools や別の Xcode を指している場合は、使用するフル版 Xcode を選びます。

```bash
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

次の1コマンドでビルドして起動します。

```bash
./script/build_and_run.sh
```

初回は `Cargo.lock` で固定された依存パッケージを取得します。スクリプトは起動中の Kaku2Okur を終了し、実行ファイルとアイコンをビルドして **`dist/Kaku2Okur.app`** を作成し、ad-hoc 署名を検証してキャンバスを開きます。ローカルビルドに Apple 開発者アカウントや署名証明書は不要です。

```bash
./script/build_and_run.sh --test     # 既存の Rust テストを実行
./script/build_and_run.sh --build    # 起動せずにアプリをビルド
./script/build_and_run.sh --hidden   # ビルドしてメニューバーに常駐
./script/build_and_run.sh --debug    # デバッグ版をビルドして起動
./script/build_and_run.sh --help
```

常用する場合は、`dist/Kaku2Okur.app` を `/Applications` または `~/Applications` へコピーして、そのアプリを起動してください。権限を付与したあとは、アプリの置き場所を固定すると扱いやすくなります。

### macOS の権限

**システム設定 → プライバシーとセキュリティ**で、呼び出し用の**入力監視**と、自動貼り付けで必要になる場合の**アクセシビリティ**を許可します。入力監視が未許可の場合は、アプリに **Open Settings**・**Check Again**・**Later** が表示されます。

入力監視が利用できなくても描画はできます。自動貼り付けができなければクリップボードから手動で貼り付けてください。移動・再ビルド後に権限が働かなくなった場合は、システム設定で現在のアプリを選び直して再起動します。

### Windows

PowerShell で実行します。

```powershell
.\script\build_windows.ps1
.\dist\windows\Kaku2Okur.exe
```

出力フォルダには実行ファイル、アイコン、第三者ソフトウェアの通知が入ります。共通部分のテストは次で実行できます。

```powershell
.\script\build_windows.ps1 -Test
```

## 自己完結型の macOS インストーラを作る

[Quick3DLook](https://github.com/yakitori-daisuki/Quick3DLook/blob/main/README.ja.md) と同様に、ローカルでビルドした ad-hoc 署名付きアプリをまとめ、開発環境のない Mac でも使える、チェックサム付きインストーラを作成できます。受け取る人に Rust・Xcode・開発者アカウントは不要です。

```bash
./script/build_unsigned_installer.sh
```

Apple Silicon では次の2ファイルが生成されます。

- `dist/Kaku2Okur-install-unsigned-arm64.sh`
- `dist/Kaku2Okur-install-unsigned-arm64.sh.sha256`

Intel Mac でのビルドでは、ファイル名の `arm64` が `x86_64` になります。ビルドした Mac と同じ CPU 向けのアプリを内蔵し、Universal Binary ではありません。

受け取る人には**両方のファイル**を渡します。ダウンロード先フォルダで、チェックサムの検証に成功した場合だけインストーラを実行します。

```bash
shasum -a 256 -c Kaku2Okur-install-unsigned-arm64.sh.sha256
bash Kaku2Okur-install-unsigned-arm64.sh --user --allow-unsigned
```

書き込み可能な `/Applications` に入れる場合は `--system`、起動しない場合は `--no-open` を使います。既存アプリはバックアップされます。`--verify-only` なら、インストールせず内蔵アプリを検証できます。

> このビルドは ad-hoc 署名で、Developer ID 署名・Apple の公証はありません。`--allow-unsigned` は、新しく配置するアプリの quarantine 属性を解除する明示的な許可です。配布元を信頼できる場合だけ指定してください。SHA-256 は破損を検出できますが、配布者の本人性を独立して証明するものではありません。

ビルド済みリリースはまだ公開していません。上の手順はローカルで作成したインストーラ向けです。Release のアセットとして公開する前に、[配布メモ](docs/Distribution.md)も確認してください。

## プロジェクト構成

| パス | 内容 |
| --- | --- |
| `apps/native` | 現行の Rust アプリ、共通モデル、OS ごとの連携、Slint UI |
| `apps/native/assets/app-icon` | 編集可能な Icon Composer ソースとプレビュー |
| `apps/tauri-legacy` | 保存用の旧 Tauri 試作版。現行ビルドの対象外 |
| `script` | ビルド・テスト・アイコン作成・配布用スクリプト |
| `docs` | 製品仕様、設計判断、画像 |

[製品仕様](docs/product-spec.md)、[アーキテクチャの判断](docs/adr/0002-use-rust-native-slint-shell.md)、[第三者ソフトウェアの通知](THIRD_PARTY_NOTICES.md)を参照してください。依存ライブラリには各ライセンスが適用されます。本プロジェクト独自コードのライセンスは、現時点では未選定です。

メンテナー向け：[このフォルダだけで使う GitHub アカウントの設定](docs/GitHub-setup.md)。

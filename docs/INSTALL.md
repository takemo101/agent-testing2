# インストールガイド

このガイドでは、Pomodoro Timer CLI（`pomodoro`）をmacOSにインストールする方法を説明します。

## インストール方法

### 1. Homebrewを使用する（推奨）

現在準備中です。公開後は以下のコマンドでインストール可能になります。

```bash
brew tap takemo101/pomodoro
brew install pomodoro
```

### 2. GitHub Releasesからバイナリをダウンロードする

1. [GitHub Releases](https://github.com/takemo101/agent-testing2/releases)から最新の `pomodoro-macos-universal` をダウンロードします。
2. ダウンロードしたファイルを `/usr/local/bin` などのパスが通ったディレクトリに移動します。

```bash
# バイナリの移動
sudo mv ~/Downloads/pomodoro /usr/local/bin/

# 実行権限の付与
sudo chmod +x /usr/local/bin/pomodoro
```

### 3. ソースコードからビルドする

Rustの開発環境（Cargo）がインストールされている必要があります。

```bash
# リポジトリのクローン
git clone https://github.com/takemo101/agent-testing2.git
cd agent-testing2

# ビルド（リリース用バイナリ）
cargo build --release

# バイナリのインストール
cp target/release/pomodoro /usr/local/bin/
```

## インストール後の設定

### 通知の許可

初回実行時にmacOSから通知の許可を求められます。ポモドーロタイマーを正しく機能させるために「許可」を選択してください。

もし許可をスキップしてしまった場合は、「システム設定 > 通知 > pomodoro」から手動で有効にできます。

### 自動起動の設定（任意）

ログイン時に自動的にバックグラウンドサービスを開始するには、以下のコマンドを実行します。

```bash
pomodoro install
```

これにより、macOSの LaunchAgent として登録されます。アンインストールする場合は `pomodoro uninstall` を実行してください。

### フォーカスモードの準備（任意）

`--focus-mode` フラグを使用するには、macOSの「集中モード」の設定が必要です。詳細は [フォーカスモード設定ガイド](FOCUS_MODE_SETUP.md) を参照してください。

## アップデート方法

### Homebrewの場合
```bash
brew upgrade pomodoro
```

### バイナリを直接配置した場合
新しいバイナリをダウンロードし、既存のバイナリを上書きしてください。

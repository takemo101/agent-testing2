# Pomodoro Timer CLI (pomodoro)

[![CI](https://github.com/takemo101/agent-testing2/actions/workflows/ci.yml/badge.svg)](https://github.com/takemo101/agent-testing2/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

macOS専用のネイティブ・ポモドーロタイマーCLIツールです。Rustで構築され、macOSの通知システム、メニューバー、フォーカスモード、LaunchAgentと密接に統合されています。

## 特徴

- 🍎 **macOSネイティブ**: macOSの機能を最大限に活用した高速な動作
- 🔔 **通知システム**: アクションボタン付きのネイティブ通知（タイマー終了時に直接再開や休憩が可能）
- 🕒 **メニューバー統合**: タイマーの残り時間をメニューバーにリアルタイム表示
- 🧘 **フォーカスモード連携**: タイマー開始時に自動でフォーカスモード（集中モード）を有効化
- 🚀 **LaunchAgentサポート**: ログイン時にバックグラウンドサービスとして自動起動
- 🔊 **サウンド再生**: 集中を妨げないシステムサウンドによる通知
- 🐚 **シェル補完**: Bash, Zsh, Fish向けの補完スクリプト生成

## クイックスタート

### タイマーの開始
25分の作業セッションを開始します。
```bash
pomodoro start
```

### カスタム設定での開始
作業50分、休憩10分、タスク名を指定して開始します。
```bash
pomodoro start --work 50 --break-time 10 --task "ドキュメント作成"
```

### ステータスの確認
```bash
pomodoro status
```

## システム要件

- **OS**: macOS 12 (Monterey) 以降
- **アーキテクチャ**: Universal Binary (Intel / Apple Silicon)

## ドキュメント

詳細な使用方法や設定については、以下のドキュメントを参照してください。

- [インストールガイド](docs/INSTALL.md) - インストール方法と初期設定
- [利用ガイド](docs/USAGE.md) - 全コマンドのリファレンスと便利な使い方
- [フォーカスモード設定](docs/FOCUS_MODE_SETUP.md) - フォーカスモード連携のセットアップ方法
- [トラブルシューティング](docs/TROUBLESHOOTING.md) - よくある問題と解決策

## インストール

現在、Homebrew経由でのインストールを推奨しています（準備中）。

```bash
# 準備中
brew tap takemo101/pomodoro
brew install pomodoro
```

その他のインストール方法については[インストールガイド](docs/INSTALL.md)をご覧ください。

## ライセンス

[MIT License](LICENSE)

## コントリビューション

Issueの報告やプルリクエストは大歓迎です。開発環境のセットアップについては[設計ドキュメント](docs/designs/detailed/pomodoro-timer/README.md)を参照してください。

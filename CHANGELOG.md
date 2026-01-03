# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

### Added
- **基本的なタイマー機能**: 25分の作業、5分の短い休憩、15分の長い休憩のサイクルを実装。
- **macOSネイティブ通知**: タイマー終了時の通知と、通知内での「休憩開始」「作業再開」アクションボタンを実装。
- **メニューバー統合**: メニューバーへのタイマー残り時間のリアルタイム表示機能を追加。
- **フォーカスモード連携**: Shortcuts.appを経由したmacOS集中モードの自動制御機能を実装。
- **LaunchAgentサポート**: `pomodoro install` コマンドによるログイン時の自動起動設定機能を追加。
- **サウンド通知**: 集中を妨げないシステムサウンドによる通知機能を追加。
- **シェル補完**: Bash, Zsh, Fish向けのコマンド補完スクリプト生成機能を追加。
- **包括的なテストスイート**: 統合テスト、E2Eテスト、およびパフォーマンス検証テストを実装。

[0.1.0]: https://github.com/takemo101/agent-testing2/releases/tag/v0.1.0

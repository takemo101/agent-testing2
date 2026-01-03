# ポモドーロタイマーCLI 詳細設計書インデックス

## メタ情報

| 項目 | 内容 |
|------|------|
| 基本設計書 | [BASIC-CLI-001_pomodoro-timer.md](../../basic/BASIC-CLI-001_pomodoro-timer.md) |
| 対応要件 | REQ-CLI-001 |
| 作成日 | 2026-01-03 |
| 最終更新日 | 2026-01-03 |

---

## 詳細設計書一覧

### コンポーネント別

| # | 詳細設計書 | 対象機能 | 優先度 | ステータス |
|---|-----------|---------|--------|-----------|
| 1 | [Daemonサーバー詳細設計書](./daemon-server.md) | F-001, F-003, F-005, F-009 | 高 | 作成完了 |
| 2 | [CLIクライアント詳細設計書](./cli-client.md) | F-003, F-004, F-006, F-007, F-008 | 高 | 作成完了 |
| 3 | [通知システム詳細設計書](./notification-system.md) | F-002, F-017 | 高 | 作成完了 |
| 4 | [メニューバーUI詳細設計書](./menubar-ui.md) | F-018 | 中 | 作成完了 |
| 5 | [フォーカスモード連携詳細設計書](./focus-mode.md) | F-019 | 中 | 作成完了 |
| 6 | [LaunchAgent管理詳細設計書](./launch-agent.md) | F-020 | 中 | 作成完了 |
| 7 | [サウンド再生詳細設計書](./sound-playback.md) | F-021 | 低 | 作成完了 |

### 共通設計

| # | 詳細設計書 | 対象 | 優先度 | ステータス |
|---|-----------|------|--------|-----------|
| 1 | [インフラ設計書（CI/CD）](./infrastructure.md) | GitHub Actions, リリース自動化 | 高 | 作成完了 |
| 2 | [テスト項目書](./test-specification.md) | 単体テスト、統合テスト、E2E、手動テスト | 高 | 作成完了 |
| 3 | [Issue計画書](./issue-plan.md) | GitHub Issue定義、依存関係、マイルストーン | 高 | 作成完了 |

---

## フォルダ構成

```
docs/designs/detailed/pomodoro-timer/
├── README.md                           # このファイル
├── daemon-server.md                    # Daemonサーバー詳細設計
├── cli-client.md                       # CLIクライアント詳細設計
├── notification-system.md              # 通知システム詳細設計
├── menubar-ui.md                       # メニューバーUI詳細設計
├── focus-mode.md                       # フォーカスモード連携詳細設計
├── launch-agent.md                     # LaunchAgent管理詳細設計
├── sound-playback.md                   # サウンド再生詳細設計
├── infrastructure.md                   # CI/CD, リリース
├── test-specification.md               # テスト項目書
└── issue-plan.md                       # GitHub Issue計画書
```

---

## 作成順序（推奨）

1. **Daemonサーバー詳細設計書** - タイマーエンジン、IPC、状態管理の中核
2. **CLIクライアント詳細設計書** - ユーザーインターフェースの中核
3. **通知システム詳細設計書** - objc2実装の技術的検証が必要
4. **インフラ設計書（CI/CD）** - ビルド・テスト・リリースの自動化
5. **メニューバーUI詳細設計書** - tray-icon実装
6. **LaunchAgent管理詳細設計書** - plist生成、launchctl操作
7. **フォーカスモード連携詳細設計書** - Shortcuts.app連携
8. **サウンド再生詳細設計書** - rodio実装

---

## 技術的検証項目

基本設計書で特定された技術的検証が必要な項目：

| 項目 | 検証内容 | 期限 | ステータス |
|------|---------|------|-----------|
| objc2-user-notifications | アクションボタンのイベントハンドリング実装 | 2026-01-10 | 詳細設計完了 |
| Unix Domain Socket | 接続エラー時のリトライロジック | 2026-01-11 | 詳細設計完了 |
| tray-icon | メニューバーアイコンの動的更新 | 2026-01-09 | 詳細設計完了 |
| Shortcuts.app | フォーカスモード制御の動作確認 | 2026-01-08 | 詳細設計完了 |
| rodio | macOSシステムサウンドの再生 | 2026-01-07 | 詳細設計完了 |

---

## 完了したステップ

1. **詳細設計書作成** - 8件の詳細設計書を作成完了
2. **詳細設計書レビュー** - 10/10点で合格
3. **テスト項目書作成** - 150+テストケースを作成
4. **Issue計画書作成** - 15 Issues、依存関係図、マイルストーン定義完了
5. **GitHub Issue登録** - 全15件のIssueをGitHubに登録完了

## GitHub Issues

| # | タイトル | ラベル | 優先度 |
|---|---------|--------|--------|
| [#1](https://github.com/takemo101/agent-testing2/issues/1) | [Setup] Rustプロジェクト初期化 | setup | 高 |
| [#2](https://github.com/takemo101/agent-testing2/issues/2) | [Core] データ型定義 | core | 高 |
| [#3](https://github.com/takemo101/agent-testing2/issues/3) | [Daemon] タイマーエンジン | daemon, core | 高 |
| [#4](https://github.com/takemo101/agent-testing2/issues/4) | [Daemon] IPCサーバー | daemon, ipc | 高 |
| [#5](https://github.com/takemo101/agent-testing2/issues/5) | [CLI] コマンドパーサー | cli | 高 |
| [#6](https://github.com/takemo101/agent-testing2/issues/6) | [macOS] 通知システム | macos, notification | 高 |
| [#7](https://github.com/takemo101/agent-testing2/issues/7) | [macOS] メニューバーUI | macos, ui | 中 |
| [#8](https://github.com/takemo101/agent-testing2/issues/8) | [macOS] サウンド再生 | macos, audio | 低 |
| [#9](https://github.com/takemo101/agent-testing2/issues/9) | [macOS] フォーカスモード | macos, focus | 中 |
| [#10](https://github.com/takemo101/agent-testing2/issues/10) | [macOS] LaunchAgent | macos, launchagent | 中 |
| [#11](https://github.com/takemo101/agent-testing2/issues/11) | [CLI] シェル補完 | cli, ux | 低 |
| [#12](https://github.com/takemo101/agent-testing2/issues/12) | [Test] 統合テスト | test | 高 |
| [#13](https://github.com/takemo101/agent-testing2/issues/13) | [Test] E2Eテスト | test | 中 |
| [#14](https://github.com/takemo101/agent-testing2/issues/14) | [Docs] ドキュメント | documentation | 中 |
| [#15](https://github.com/takemo101/agent-testing2/issues/15) | [Release] リリース | release, ci | 高 |

## 次のステップ

1. **実装開始** - Issue #1（プロジェクト初期化）から順次実装

---

## 関連ドキュメント

- [要件定義書](../../../requirements/REQ-CLI-001_pomodoro-timer.md)
- [基本設計書](../../basic/BASIC-CLI-001_pomodoro-timer.md)
- [技術調査レポート: tokio](../../../research/TECH-BE-001_tokio.md)
- [技術調査レポート: LaunchAgents](../../../research/TECH-INFRA-001_LaunchAgents.md)
- [技術調査レポート: clap](../../../research/TECH-TOOL-001_clap.md)
- [技術調査レポート: objc2](../../../research/TECH-TOOL-002_objc2.md)
- [技術調査レポート: indicatif, rodio, tray-icon](../../../research/TECH-TOOL-003_ui-audio-tray.md)

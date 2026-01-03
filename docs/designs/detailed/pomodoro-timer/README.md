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
| 1 | Daemonサーバー詳細設計書 | F-001, F-003, F-005, F-009 | 高 | 未作成 |
| 2 | CLIクライアント詳細設計書 | F-003, F-006, F-007, F-008 | 高 | 未作成 |
| 3 | 通知システム詳細設計書 | F-002, F-017 | 高 | 未作成 |
| 4 | メニューバーUI詳細設計書 | F-018 | 中 | 未作成 |
| 5 | フォーカスモード連携詳細設計書 | F-019 | 中 | 未作成 |
| 6 | LaunchAgent管理詳細設計書 | F-020 | 中 | 未作成 |
| 7 | サウンド再生詳細設計書 | F-021 | 低 | 未作成 |

### 共通設計

| # | 詳細設計書 | 対象 | 優先度 | ステータス |
|---|-----------|------|--------|-----------|
| 1 | インフラ設計書（CI/CD） | GitHub Actions, リリース自動化 | 高 | 未作成 |

---

## フォルダ構成

```
docs/designs/detailed/pomodoro-timer/
├── README.md                           # このファイル
├── 共通/
│   └── インフラ設計書.md               # CI/CD, リリース
├── daemon-server.md                    # Daemonサーバー詳細設計
├── cli-client.md                       # CLIクライアント詳細設計
├── notification-system.md              # 通知システム詳細設計
├── menubar-ui.md                       # メニューバーUI詳細設計
├── focus-mode.md                       # フォーカスモード連携詳細設計
├── launch-agent.md                     # LaunchAgent管理詳細設計
└── sound-playback.md                   # サウンド再生詳細設計
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
| objc2-user-notifications | アクションボタンのイベントハンドリング実装 | 2026-01-10 | 未着手 |
| Unix Domain Socket | 接続エラー時のリトライロジック | 2026-01-11 | 未着手 |
| tray-icon | メニューバーアイコンの動的更新 | 2026-01-09 | 未着手 |
| Shortcuts.app | フォーカスモード制御の動作確認 | 2026-01-08 | 未着手 |
| rodio | macOSシステムサウンドの再生 | 2026-01-07 | 未着手 |

---

## 関連ドキュメント

- [要件定義書](../../../requirements/REQ-CLI-001_pomodoro-timer.md)
- [基本設計書](../../basic/BASIC-CLI-001_pomodoro-timer.md)
- [技術調査レポート: tokio](../../../research/TECH-BE-001_tokio.md)
- [技術調査レポート: LaunchAgents](../../../research/TECH-INFRA-001_LaunchAgents.md)
- [技術調査レポート: clap](../../../research/TECH-TOOL-001_clap.md)
- [技術調査レポート: objc2](../../../research/TECH-TOOL-002_objc2.md)
- [技術調査レポート: indicatif, rodio, tray-icon](../../../research/TECH-TOOL-003_ui-audio-tray.md)

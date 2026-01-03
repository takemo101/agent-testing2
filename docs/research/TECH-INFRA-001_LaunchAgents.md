# 技術調査レポート: LaunchAgents

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-03 |
| 調査深度 | standard |
| 対象バージョン | N/A (macOS native) |
| 現行バージョン | 新規導入 |

## エグゼクティブサマリー

macOSのLaunchAgentsはログイン時に自動実行されるバックグラウンドプロセスの管理に使用される。Rustからの管理には `plist` クレートを用いたXMLの生成と、`launchctl` コマンドの呼び出しが一般的である。最新のmacOS（10.10+）では `bootstrap` / `bootout` サブコマンドが推奨されているが、互換性のために依然として `load` / `unload` も広く使われている。

## バージョン情報

macOS 12 (Monterey) 以降を対象とする。

## Breaking Changes

### ℹ️ 低影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| `launchctl load/unload` の非推奨化 | サービス管理 | 可能な限り `bootstrap` / `bootout` を使用するが、既存の `load` / `unload` も動作は継続している。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| `bootstrap` / `bootout` | ドメイン（gui/UID）を指定した新しいサービス管理方式 | ユーザーセッションに紐付いた確実なサービス起動 |

## 設計への影響

### 必須対応

- [ ] Plistファイル内のパス（`ProgramArguments`, `StandardOutPath`等）には `~` を使わず、絶対パス（`/Users/...`）を使用する。
- [ ] インストール場所は `~/Library/LaunchAgents/` とする。
- [ ] 冪等性を確保するため、`load` 前に必ず `unload` を試行する。

### 推奨対応

- [ ] `plist` クレートの `serde` 機能を使い、型安全にPlist構造を定義する。
- [ ] `Label` には逆ドメイン形式（例: `com.example.pomodoro`）を使用する。

## マイグレーション手順（該当時）

実装で使用する依存関係：

```toml
[dependencies]
plist = "1.6"
serde = { version = "1.0", features = ["derive"] }
dirs = "5.0"
```

## 参考リンク

- [Apple Launchd Guide](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPMonitoring/Articles/LaunchOnDemand.html)
- [launchctl(1) Manual Page](https://www.manpagez.com/man/1/launchctl/)
- [plist crate documentation](https://docs.rs/plist/latest/plist/)

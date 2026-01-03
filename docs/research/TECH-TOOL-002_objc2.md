# 技術調査レポート: objc2

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-03 |
| 調査深度 | standard |
| 対象バージョン | v0.6.3 (core) / v0.3.2 (frameworks) |
| 現行バージョン | 新規導入 |

## エグゼクティブサマリー

`objc2` はRustからmacOSのObjective-Cランタイムおよびフレームワークにアクセスするための最新かつ安全なライブラリである。`UserNotifications` フレームワークのバインディングを提供しており、アクションボタン（一時停止・停止等）付きのネイティブ通知の実装に最適である。また、Focus Mode（おやすみモード等）の制御については、公式の公開APIが存在しないため、Shortcuts.appを介した外部コマンド実行による代替案が推奨される。

## バージョン情報

| バージョン | リリース日 | サポート状況 |
|-----------|-----------|-------------|
| v0.6.3 | 2025-12-21 | Active (Core) |
| v0.3.2 | 2025-10-04 | Active (Frameworks) |

## Breaking Changes

### ℹ️ 低影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| フレームワーク単位の分割 | 依存関係管理 | `objc2` 本体に加えて、`objc2-foundation`, `objc2-user-notifications` 等、必要なフレームワーク用クレートを個別に導入する。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| `Retained<T>` | 参照カウントに基づいた自動メモリ管理 | `Retain`/`Release` の手動管理を不要にし、メモリ安全性を確保 |
| 自動生成バインディング | Xcode SDKから自動生成された最新のAPI定義 | 最新のmacOS SDK機能へのアクセス |

## 設計への影響

### 必須対応

- [ ] 通知アクションボタンを使用する場合、`UNUserNotificationCenterDelegate` を `define_class!` マクロを使用して実装する。
- [ ] アプリケーションのバイナリを `codesign` で署名する。署名がないと `UserNotifications` APIが正常に動作しない。
- [ ] Focus Mode制御については、`/usr/bin/shortcuts run "Shortcut Name"` を実行するコマンドインターフェースを実装する。

### 推奨対応

- [ ] `mac-notification-sys` は非推奨APIを使用しているため、直接 `objc2-user-notifications` を使用する。

## マイグレーション手順（該当時）

依存関係の追加：

```toml
[dependencies]
objc2 = "0.6"
objc2-foundation = "0.3"
objc2-user-notifications = "0.3"
```

## 参考リンク

- [objc2 GitHub Repository](https://github.com/madsmtm/objc2)
- [objc2-user-notifications Documentation](https://docs.rs/objc2-user-notifications/)
- [Apple UserNotifications Guide](https://developer.apple.com/documentation/usernotifications)

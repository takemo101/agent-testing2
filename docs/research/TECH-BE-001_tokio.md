# 技術調査レポート: tokio

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-03 |
| 調査深度 | standard |
| 対象バージョン | v1.48.0 |
| 現行バージョン | 新規導入 |

## エグゼクティブサマリー

TokioはRustの非同期ランタイムとしてデファクトスタンダードであり、非常に安定している。最新バージョンv1.48.0（2025年10月リリース）ではMSRVが1.71に引き上げられた。CLIツール、特にポモドーロタイマーのようなバックグラウンド処理が必要なツールにおいては、`current_thread`ランタイムの使用が推奨される。

## バージョン情報

| バージョン | リリース日 | サポート状況 |
|-----------|-----------|-------------|
| v1.48.0 | 2025-10-14 | Active (MSRV 1.71) |
| v1.47.3 | 2026-01-03 | Patch Release |
| v1.43.0 | 2025-01-08 | LTS (MSRV 1.70) |

## Breaking Changes

### ⚠️ 中影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| MSRV 1.71への引き上げ (v1.48.0) | コンパイル環境 | `Cargo.toml` の `rust-version` を `"1.71"` 以上に設定する。 |
| Broadcast channelの健全性修正 (v1.44.2) | `Send + !Sync` 型の使用 | v1.44.2以降を使用することで自動的に修正される。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| `SetOnce` (v1.47.0) | `OnceLock` に似た同期プリミティブ | 初期化が一度限りのグローバル状態管理 |
| `biased` join! (v1.46.0) | マクロの実行順序を決定論的に制御 | 優先順位がある非同期タスクの実行 |
| `coop` APIs (v1.47.0) | 協調的スケジューリングの制御 | 長時間のCPU負荷が高いタスクでのハング防止 |

## 設計への影響

### 必須対応

- [ ] `tokio` の機能フラグとして `rt`, `time`, `macros`, `signal` を有効化する。
- [ ] タイマーのドリフトを防ぐため、`tokio::time::interval` の `MissedTickBehavior` を `Skip` に設定する。

### 推奨対応

- [ ] CLIツールとしてのオーバーヘッドを削減するため、`#[tokio::main(flavor = "current_thread")]` を使用する。
- [ ] Ctrl+Cなどのシグナル処理に `tokio::signal::ctrl_c()` と `tokio::select!` を組み合わせて使用する。

## マイグレーション手順（該当時）

新規プロジェクトのため、最新バージョンを導入：

```bash
cargo add tokio --features rt,time,macros,signal
```

## 参考リンク

- [公式ドキュメント](https://docs.rs/tokio/1.48.0/tokio/)
- [CHANGELOG](https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

# 技術調査レポート: indicatif, rodio, tray-icon

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-03 |
| 調査深度 | standard |
| 対象バージョン | indicatif v0.18.3, rodio v0.21.1, tray-icon v0.21.2 |
| 現行バージョン | 新規導入 |

## エグゼクティブサマリー

CLI UI（進捗表示）、オーディオ再生、メニューバーアイコンに関連する主要ライブラリの最新状況を調査した。`indicatif` と `tray-icon` は安定しており、直近1年で大きな破壊的変更はない。一方、`rodio` は2025年7月にメジャーアップデート（v0.21.0）があり、APIの設計が大幅に変更された。新規プロジェクトではこれら最新バージョンを採用し、特に `rodio` の新しいストリーム管理方式に準拠した設計が必要である。

## バージョン情報

| ライブラリ | バージョン | リリース日 | 状況 |
|-----------|-----------|-----------|------|
| indicatif | v0.18.3 | 2025-11-11 | 安定 (MSRV 1.71) |
| rodio | v0.21.1 | 2025-07-14 | **メジャーアップデート済** |
| tray-icon | v0.21.2 | 2025-10-20 | 安定 |

## Breaking Changes (主要な変更点)

### 🚨 高影響: rodio v0.21.0

| 変更内容 | 影響範囲 | 移行/対応方法 |
|---------|---------|-------------|
| ストリームAPIの刷新 | ストリーム初期化 | `OutputStreamHandle` が廃止された。`OutputStreamBuilder` を使用し、`Sink::connect_new(stream.mixer())` で接続する。 |
| サンプル型の統一 | デコード処理 | 全てのソースが内部的に `f32` を使用するようになった。`u16` や `i16` への明示的な変換が不要（または dasp_sample 等での対応）になった。 |
| デコーダーの統合 | ファイル読み込み | Symphoniaがデフォルトデコーダーになり、`Decoder::try_from(file)?` で簡潔にロード可能になった。 |

### ℹ️ 低影響: indicatif v0.18.0

- **MSRVの変更**: Rust 1.71以上が必須となった。
- **機能追加**: `ProgressBar::set_elapsed` が追加され、タイマーの経過時間を手動制御しやすくなった。

## 設計への影響

### 必須対応

- [ ] **rodioの初期化**: 新しい `OutputStreamBuilder` 方式で実装する。
- [ ] **通知音のロード**: `Decoder::try_from(file)` を活用し、`BufReader` の手動ラップを省略してコードを簡略化する。
- [ ] **MSRVの整合性**: `tokio`, `objc2`, `indicatif` すべての要件を考慮し、プロジェクト全体のMSRVを **1.71** に設定する。

### 推奨対応

- [ ] **音量制御**: `rodio` v0.21で追加された `amplify_decibel()` を使用し、デシベル単位での直感的な音量調整機能を検討する。
- [ ] **メニューバー制御**: `tray-icon` の `ns_status_item` メソッド（macOS専用）を必要に応じて使用し、ネイティブな動作を細かく制御する。

## マイグレーション手順（新規導入例）

```toml
[dependencies]
indicatif = "0.18"
rodio = "0.21"
tray-icon = "0.21"
```

オーディオ再生の新しいパターン：
```rust
use rodio::{OutputStreamBuilder, Sink, Decoder};
use std::fs::File;

let stream = OutputStreamBuilder::open_default_stream()?;
let sink = Sink::connect_new(stream.mixer());
let source = Decoder::try_from(File::open("alert.mp3")?)?;
sink.append(source);
```

## 参考リンク

- [rodio v0.21 Upgrade Guide](https://github.com/RustAudio/rodio/blob/master/UPGRADE.md)
- [indicatif documentation](https://docs.rs/indicatif/latest/indicatif/)
- [tray-icon documentation](https://docs.rs/tray-icon/latest/tray_icon/)

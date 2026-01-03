# 利用ガイド

Pomodoro Timer CLI（`pomodoro`）の詳しい使い方と全コマンドのリファレンスです。

## 基本的な考え方

ポモドーロ・テクニックに基づいて、以下のサイクルを繰り返します。

1. **作業セッション** (デフォルト25分)
2. **短い休憩** (デフォルト5分)
3. 4回のセッションごとに **長い休憩** (デフォルト15分)

## コマンドリファレンス

### `start` - タイマーを開始する

新しいポモドーロセッションを開始します。

```bash
pomodoro start [OPTIONS]
```

**オプション:**

| オプション | 短縮 | 説明 | デフォルト |
| :--- | :--- | :--- | :--- |
| `--work <MIN>` | `-w` | 作業時間（1-120分） | 25 |
| `--break-time <MIN>` | `-b` | 短い休憩時間（1-60分） | 5 |
| `--long-break <MIN>` | `-l` | 長い休憩時間（1-60分） | 15 |
| `--task <NAME>` | `-t` | 現在のタスク名 | なし |
| `--auto-cycle` | `-a` | 休憩終了後に自動で次の作業を開始 | false |
| `--focus-mode` | `-f` | macOSの集中モードを自動制御 | false |
| `--no-sound` | | 通知時のサウンドを無効化 | false |

### `pause` - タイマーを一時停止する

実行中のタイマーを一時停止します。

```bash
pomodoro pause
```

### `resume` - タイマーを再開する

一時停止中のタイマーを再開します。

```bash
pomodoro resume
```

### `stop` - タイマーを停止する

現在のセッションを終了し、タイマーをリセットします。

```bash
pomodoro stop
```

### `status` - 現在の状況を表示する

タイマーの残り時間、現在の状態（作業中/休憩中）、完了したサイクル数などを表示します。

```bash
pomodoro status
```

### `install` / `uninstall` - 自動起動の設定

macOSのログイン時にバックグラウンドサービスを自動起動するように設定/解除します。

```bash
pomodoro install
pomodoro uninstall
```

### `completions` - シェル補完の設定

お使いのシェルに合わせて補完スクリプトを生成します。

```bash
# zshの場合
pomodoro completions zsh > /usr/local/share/zsh/site-functions/_pomodoro
```

## 一般的なユースケース

### 1. 標準的なポモドーロを開始する
```bash
pomodoro start --task "コーディング"
```

### 2. 集中時間を長めに設定する（50分集中 / 10分休憩）
```bash
pomodoro start --work 50 --break-time 10 --auto-cycle
```

### 3. 集中モードを自動的に有効にする
```bash
pomodoro start --focus-mode
```
※事前に「集中モード」の設定が必要です。詳細は [フォーカスモード設定ガイド](FOCUS_MODE_SETUP.md) をご覧ください。

## 便利な機能

### メニューバーでの確認
タイマーが動作している間、macOSのメニューバーに残り時間が表示されます。ターミナルを開き直さなくても進捗を確認できます。

### 通知アクション
タイマーが終了するとmacOSの通知が表示されます。通知内のボタンをクリックすることで、ターミナルに戻らずに「休憩開始」や「作業再開」が可能です。

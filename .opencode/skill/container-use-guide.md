# container-use 環境構築ガイド

container-useを使用したクローズドな開発・テスト環境の構築手順です。

## 概要

container-useは、Dockerコンテナ内で開発・テストを行うためのツール群です。

**メリット**:
- ローカル環境を汚さない
- 再現可能な環境
- DB等のサービスを安全にテスト
- チーム間で同一環境を共有

## 基本フロー

```mermaid
flowchart LR
    LIST[環境一覧確認] --> EXISTS{既存環境?}
    EXISTS -->|あり| OPEN[環境を開く]
    EXISTS -->|なし| CREATE[環境を作成]
    CREATE --> CONFIG[環境設定]
    CONFIG --> SERVICE{サービス必要?}
    OPEN --> SERVICE
    SERVICE -->|はい| ADD[サービス追加]
    SERVICE -->|いいえ| WORK
    ADD --> WORK[作業開始]
```

## ツール一覧

| ツール | 用途 |
|--------|------|
| `container-use_environment_list` | 既存環境の一覧取得 |
| `container-use_environment_create` | 新規環境の作成 |
| `container-use_environment_open` | 既存環境を開く |
| `container-use_environment_config` | 環境設定 (base image, setup commands) |
| `container-use_environment_add_service` | サービス追加 (DB, Redis等) |
| `container-use_environment_run_cmd` | コマンド実行 |
| `container-use_environment_file_read` | ファイル読み取り |
| `container-use_environment_file_write` | ファイル書き込み |
| `container-use_environment_file_edit` | ファイル編集 |
| `container-use_environment_file_list` | ディレクトリ一覧 |
| `container-use_environment_checkpoint` | 環境のスナップショット保存 |

## 環境構築手順

### Step 1: 既存環境の確認

```python
container-use_environment_list(
    environment_source="/path/to/repo",
    explanation="Check existing environments for this project"
)
```

### Step 2: 環境の作成または再利用

#### 新規作成の場合

```python
result = container-use_environment_create(
    environment_source="/path/to/repo",
    title="Issue #123 - User Authentication Feature",
    explanation="Create dev environment for auth feature"
)
env_id = result.environment_id
```

#### 既存環境を開く場合

```python
container-use_environment_open(
    environment_source="/path/to/repo",
    environment_id="existing-env-id",
    explanation="Reopen existing environment"
)
```

### Step 3: 環境設定

プロジェクトの技術スタックに応じて設定:

```python
container-use_environment_config(
    environment_source="/path/to/repo",
    environment_id=env_id,
    config={
        "base_image": "node:20-slim",
        "setup_commands": [
            "npm ci",
            "npm run build"
        ],
        "envs": [
            "NODE_ENV=test",
            "LOG_LEVEL=debug"
        ]
    },
    explanation="Configure Node.js environment with dependencies"
)
```

### Step 4: サービス追加 (必要に応じて)

#### PostgreSQL

```python
container-use_environment_add_service(
    environment_source="/path/to/repo",
    environment_id=env_id,
    name="postgres",
    image="postgres:15-alpine",
    envs=[
        "POSTGRES_USER=app",
        "POSTGRES_PASSWORD=password",
        "POSTGRES_DB=testdb"
    ],
    ports=[5432],
    explanation="Add PostgreSQL for database tests"
)
```

#### MySQL

```python
container-use_environment_add_service(
    environment_source="/path/to/repo",
    environment_id=env_id,
    name="mysql",
    image="mysql:8",
    envs=[
        "MYSQL_ROOT_PASSWORD=root",
        "MYSQL_DATABASE=testdb",
        "MYSQL_USER=app",
        "MYSQL_PASSWORD=password"
    ],
    ports=[3306],
    explanation="Add MySQL for database tests"
)
```

#### Redis

```python
container-use_environment_add_service(
    environment_source="/path/to/repo",
    environment_id=env_id,
    name="redis",
    image="redis:7-alpine",
    ports=[6379],
    explanation="Add Redis for caching tests"
)
```

## コマンド実行

### 基本コマンド

```python
container-use_environment_run_cmd(
    environment_source="/path/to/repo",
    environment_id=env_id,
    command="npm test",
    explanation="Run test suite"
)
```

### バックグラウンド実行 (サーバー起動等)

```python
container-use_environment_run_cmd(
    environment_source="/path/to/repo",
    environment_id=env_id,
    command="npm run dev",
    background=True,
    ports=[3000],
    explanation="Start dev server in background"
)
```

### シェル指定

```python
container-use_environment_run_cmd(
    environment_source="/path/to/repo",
    environment_id=env_id,
    command="source .env && npm test",
    shell="bash",
    explanation="Run with bash to source env file"
)
```

## ファイル操作

### ファイル読み取り

```python
container-use_environment_file_read(
    environment_source="/path/to/repo",
    environment_id=env_id,
    target_file="src/index.ts",
    should_read_entire_file=True,
    explanation="Read source file"
)
```

### ファイル書き込み

```python
container-use_environment_file_write(
    environment_source="/path/to/repo",
    environment_id=env_id,
    target_file="src/feature.ts",
    contents="export const feature = () => { ... }",
    explanation="Write new feature file"
)
```

### ファイル編集

```python
container-use_environment_file_edit(
    environment_source="/path/to/repo",
    environment_id=env_id,
    target_file="src/index.ts",
    search_text="old code",
    replace_text="new code",
    explanation="Update import statement"
)
```

## 技術スタック別設定例

### Node.js / TypeScript

```python
config = {
    "base_image": "node:20-slim",
    "setup_commands": [
        "npm ci",
        "npx playwright install chromium --with-deps"
    ],
    "envs": [
        "NODE_ENV=test",
        "DATABASE_URL=postgresql://app:password@postgres:5432/testdb"
    ]
}
```

### Python / FastAPI

```python
config = {
    "base_image": "python:3.11-slim",
    "setup_commands": [
        "pip install --no-cache-dir -r requirements.txt",
        "pip install --no-cache-dir -r requirements-dev.txt"
    ],
    "envs": [
        "PYTHONPATH=/workspace",
        "DATABASE_URL=postgresql://app:password@postgres:5432/testdb"
    ]
}
```

### Go

```python
config = {
    "base_image": "golang:1.21-alpine",
    "setup_commands": [
        "go mod download",
        "go install github.com/golang-migrate/migrate/v4/cmd/migrate@latest"
    ],
    "envs": [
        "CGO_ENABLED=0",
        "DATABASE_URL=postgres://app:password@postgres:5432/testdb?sslmode=disable"
    ]
}
```

### Rust

```python
config = {
    "base_image": "rust:1.74-slim",
    "setup_commands": [
        "cargo fetch",
        "cargo build --release"
    ],
    "envs": [
        "DATABASE_URL=postgres://app:password@postgres:5432/testdb"
    ]
}
```

## DBマイグレーションのテスト

### Flyway (SQL migrations)

```python
# マイグレーション実行
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="flyway -url=jdbc:postgresql://postgres:5432/testdb -user=app -password=password migrate",
    explanation="Run Flyway migrations"
)

# ロールバック
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="flyway -url=jdbc:postgresql://postgres:5432/testdb -user=app -password=password undo",
    explanation="Rollback last migration"
)
```

### Prisma (TypeScript)

```python
# マイグレーション実行
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npx prisma migrate deploy",
    explanation="Run Prisma migrations"
)

# リセット (開発用)
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npx prisma migrate reset --force",
    explanation="Reset database and rerun migrations"
)
```

### SQLAlchemy / Alembic (Python)

```python
# マイグレーション実行
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="alembic upgrade head",
    explanation="Run Alembic migrations"
)

# ロールバック
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="alembic downgrade -1",
    explanation="Rollback one migration"
)
```

## トラブルシューティング

### サービスに接続できない

1. サービス名をホスト名として使用 (例: `postgres`, `redis`)
2. ポートが正しいか確認
3. サービスの起動を待つ

```python
# サービス起動待ち
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="until pg_isready -h postgres -p 5432; do sleep 1; done",
    explanation="Wait for PostgreSQL to be ready"
)
```

### 依存関係のインストールに失敗

1. base imageを確認
2. setup_commandsの順序を確認
3. 必要なシステムパッケージを追加

```python
config = {
    "base_image": "node:20-slim",
    "setup_commands": [
        "apt-get update && apt-get install -y build-essential python3",  # ネイティブモジュール用
        "npm ci"
    ]
}
```

### 環境が重い

1. slimイメージを使用
2. 不要なdevDependenciesを除外
3. マルチステージビルドを検討

## ベストプラクティス

1. **環境の再利用**: 同じIssueの作業には同じ環境を使う
2. **サービス名の統一**: `postgres`, `redis`, `mysql` など分かりやすい名前を使う
3. **環境変数の活用**: 接続情報は環境変数で管理
4. **チェックポイント**: 安定した状態でスナップショットを保存
5. **クリーンアップ**: 不要になった環境は削除

## 環境ID管理 (environments.json)

PRレビュー後の修正作業で環境を再利用するため、環境IDを `.opencode/environments.json` で追跡します。

**ファイル形式**:

```json
{
  "$schema": "./environments.schema.json",
  "environments": [
    {
      "env_id": "abc-123-def",
      "branch": "feature/issue-42-user-auth",
      "issue_number": 42,
      "pr_number": 45,
      "status": "active",
      "created_at": "2026-01-03T10:00:00Z",
      "last_used_at": "2026-01-03T15:30:00Z"
    }
  ]
}
```

### ステータス更新タイミング

| イベント | ステータス変更 | アクション |
|---------|---------------|-----------|
| 環境作成時 | → `active` | 新規エントリ追加 |
| PR作成時 | `active` のまま | `pr_number` を記録 |
| PRマージ時 | → `merged` | 環境は保持（後で削除可） |
| PRクローズ時（マージなし） | → `abandoned` | 環境削除を推奨 |
| 環境手動削除時 | エントリ削除 | environments.jsonから削除 |

> **有効なステータス値**: `active`, `merged`, `abandoned`（スキーマ定義に準拠）

### environments.json 更新手順

**誰が更新するか**: Sisyphus（AIエージェント）が `container-use_environment_*` ツール使用時に自動更新する。

**更新方法**:

```python
# 環境作成時の追加
def add_environment_entry(env_id, branch, issue_number, title):
    with open('.opencode/environments.json', 'r+') as f:
        data = json.load(f)
        data['environments'].append({
            "env_id": env_id,
            "branch": branch,
            "issue_number": issue_number,
            "title": title,
            "status": "active",
            "created_at": datetime.now().isoformat() + "Z",
            "last_used_at": datetime.now().isoformat() + "Z"
        })
        f.seek(0)
        json.dump(data, f, indent=2)
        f.truncate()

# ステータス更新
def update_environment_status(env_id, new_status):
    with open('.opencode/environments.json', 'r+') as f:
        data = json.load(f)
        for env in data['environments']:
            if env['env_id'] == env_id:
                env['status'] = new_status
                env['last_used_at'] = datetime.now().isoformat() + "Z"
                break
        f.seek(0)
        json.dump(data, f, indent=2)
        f.truncate()

# PR番号の記録（PR作成時）
def set_pr_number(env_id, pr_number):
    with open('.opencode/environments.json', 'r+') as f:
        data = json.load(f)
        for env in data['environments']:
            if env['env_id'] == env_id:
                env['pr_number'] = pr_number
                env['last_used_at'] = datetime.now().isoformat() + "Z"
                break
        f.seek(0)
        json.dump(data, f, indent=2)
        f.truncate()
```

**実装時の呼び出しタイミング**:
- `container-use_environment_create` 成功後 → `add_environment_entry()`
- `container-use_environment_open` 成功後 → `last_used_at` を更新
- `gh pr create` 成功後 → `set_pr_number(env_id, pr_number)`
- PR マージ検知時 → `update_environment_status(env_id, "merged")`
- PR クローズ検知時 → `update_environment_status(env_id, "abandoned")`

### 環境再利用フロー（PRレビュー修正時）

1. **environments.json を確認**: 該当PR番号でフィルタ
2. **env_id を取得**: `status: active` のエントリを使用
3. **環境を開く**: `container-use_environment_open` を使用
4. **修正作業を実行**: 既存環境内で作業
5. **last_used_at を更新**: 作業完了時（自動）

```python
# 環境再利用の例
import json

# 1. environments.json を読み込み
with open('.opencode/environments.json') as f:
    data = json.load(f)

# 2. 該当PRの環境を検索
env = next(
    (e for e in data['environments'] 
     if e['pr_number'] == 45 and e['status'] == 'active'),
    None
)

if env:
    # 3. 既存環境を開く
    container-use_environment_open(
        environment_source="/path/to/repo",
        environment_id=env['env_id'],
        explanation="Reopen environment for PR feedback fixes"
    )
else:
    # 環境が見つからない場合は新規作成
    ...
```

### クリーンアップポリシー

| 条件 | 推奨アクション |
|------|---------------|
| PRマージから7日以上経過 | 環境削除 + エントリ削除 |
| PRクローズ（マージなし） | 即時削除推奨 |
| `last_used_at` が30日以上前 | 削除検討 |

**注意**: `environments.json` はローカル環境データのため `.gitignore` に含まれています。
チーム間で共有する必要がある場合は別途管理してください。

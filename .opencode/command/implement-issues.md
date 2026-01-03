# Issue実装コマンド (TDD + container-use)

指定されたGitHub Issueを実装します。
**TDD（テスト駆動開発）を強制**し、品質基準を満たすまでリトライします。
**container-use環境**でクローズドな開発・テストを行います。

## 引数
Issue番号を指定（例: `/implement-issues 123`）

## ワークフロー概要

```mermaid
flowchart TB
    START(Issue着手) --> BRANCH["🌿 ブランチ作成<br/>feature/issue-{N}"]
    BRANCH --> ENV["🐳 container-use環境構築<br/>(from_git_ref: featureブランチ)"]
    ENV --> SERVICE{サービス必要?}
    SERVICE -->|DB等| ADD_SVC[サービス追加]
    SERVICE -->|なし| CHECK_HO
    ADD_SVC --> CHECK_HO
    
    CHECK_HO{申し送り確認}
    CHECK_HO -->|あり| DO_HO[申し送り対応]
    DO_HO --> TDD_RED
    CHECK_HO -->|なし| TDD_RED
    
    subgraph TDD["TDDサイクル (container内)"]
        TDD_RED["🔴 Red: テスト実装"]
        TDD_RED --> TDD_GREEN["🟢 Green: 最小実装"]
        TDD_GREEN --> TDD_REFACTOR["🔵 Refactor: 整理"]
    end
    
    TDD_REFACTOR --> DESIGN_CHECK{設計不備?}
    DESIGN_CHECK -->|あり| REQ_FIX[/"/request-design-fix"/]
    REQ_FIX --> ENV
    
    DESIGN_CHECK -->|なし| REVIEW{品質レビュー}
    REVIEW -->|OK (>=9点)| COMMIT["💾 コミット & プッシュ<br/>(container内)"]
    REVIEW -->|NG| FIX[修正]
    FIX --> TDD_RED
    
    COMMIT --> PR["🔀 PR作成<br/>(container内)"]
    PR --> FINISH(完了)
```

## 実行プロセス

### 0. ブランチ作成 (container-use環境作成前)

Issue着手時に、まず**featureブランチを作成**します。

```python
# ホスト側でブランチ作成 (bashツール使用)
bash("git checkout main && git pull origin main")
bash(f"git checkout -b feature/issue-{issue_id}-{short_description}")
bash(f"git push -u origin feature/issue-{issue_id}-{short_description}")
```

**ブランチ命名規則**:
| プレフィックス | 用途 |
|---------------|------|
| `feature/issue-{N}-*` | 機能追加 |
| `fix/issue-{N}-*` | バグ修正 |
| `refactor/issue-{N}-*` | リファクタリング |

### 1. container-use環境構築

**`from_git_ref`でfeatureブランチを指定**して環境を作成します。

```python
# 環境作成 (featureブランチから)
container-use_environment_create(
    environment_source="/path/to/repo",
    title=f"Issue #{issue_id} - {issue_title}",
    from_git_ref=f"feature/issue-{issue_id}-{short_description}"
)
```

これにより:
- featureブランチのコードがcontainer内にチェックアウトされる
- mainブランチは影響を受けない
- container内での変更はfeatureブランチにコミットされる

#### 1.1 環境設定

```python
container-use_environment_config(
    environment_id=env_id,
    environment_source="/path/to/repo",
    config={
        "base_image": "node:20-slim",
        "setup_commands": [
            "npm ci",
            "npm run build"
        ],
        "envs": [
            "NODE_ENV=test",
            "DATABASE_URL=postgresql://app:password@db:5432/testdb"
        ]
    }
)
```

#### 1.2 サービス追加 (必要に応じて)

```python
# PostgreSQL
container-use_environment_add_service(
    environment_id=env_id,
    environment_source="/path/to/repo",
    name="db",
    image="postgres:15",
    envs=["POSTGRES_USER=app", "POSTGRES_PASSWORD=password", "POSTGRES_DB=testdb"],
    ports=[5432]
)

# Redis (必要な場合)
container-use_environment_add_service(
    environment_id=env_id,
    environment_source="/path/to/repo",
    name="redis",
    image="redis:7-alpine",
    ports=[6379]
)
```

### 2. 申し送り確認 (Handover)

Issueのコメントをスキャンし、未完了の申し送り事項があれば最優先で対応。

### 3. TDD実装 (Red -> Green -> Refactor)

**全てcontainer-use環境内で実行**:

#### 🔴 Red: テスト実装

```python
# テスト実行 (失敗を確認)
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npm test -- --testPathPattern='feature-name'"
)
```

#### 🟢 Green: 最小実装

```python
# ファイル編集
container-use_environment_file_write(
    environment_id=env_id,
    environment_source="/path/to/repo",
    target_file="src/feature.ts",
    contents="// implementation"
)

# テスト実行 (成功を確認)
container-use_environment_run_cmd(...)
```

#### 🔵 Refactor: 整理

```python
# Lint & 型チェック
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npm run lint -- --fix && npm run type-check"
)
```

### 4. DBマイグレーションのテスト (DB関連Issue)

```python
# マイグレーション実行
container-use_environment_run_cmd(command="npx flyway migrate")

# ロールバックテスト
container-use_environment_run_cmd(command="npx flyway undo")

# 再マイグレーション
container-use_environment_run_cmd(command="npx flyway migrate")
```

### 5. 設計不備への対応

設計の矛盾が見つかった場合は `/request-design-fix` を実行。

### 6. 申し送り作成

他領域への影響がある場合は [申し送り処理ガイド](../skill/handover-process.md) に従う。

### 7. 品質レビュー

スコア9点以上で次へ。未達の場合はTDDサイクルに戻る。

### 8. コミット & プッシュ (container内で実行)

```python
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command='''
        git add . && \
        git commit -m "feat: {summary}

Closes #{issue_id}

- {change1}
- {change2}" && \
        git push origin feature/issue-{issue_id}-{description}
    '''
)
```

**コミットメッセージ規則**:
- `feat:` - 新機能
- `fix:` - バグ修正
- `refactor:` - リファクタリング
- `test:` - テスト追加
- `docs:` - ドキュメント

### 9. PR作成 (container内で実行)

```python
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command='''
        gh pr create \
          --title "feat: {title}" \
          --body "## Summary
Closes #{issue_id}

## Changes
- {change1}
- {change2}

## Test Results
{test_log}

## Checklist
- [x] TDDで実装
- [x] 品質レビュー通過
- [x] Lintエラーなし
- [x] 型エラーなし" \
          --base main \
          --head feature/issue-{issue_id}-{description}
    '''
)
```

## 技術スタック別設定

### Node.js/TypeScript

```python
config = {
    "base_image": "node:20-slim",
    "setup_commands": ["npm ci", "npx playwright install chromium --with-deps"],
    "envs": ["NODE_ENV=test"]
}
```

### Python

```python
config = {
    "base_image": "python:3.11-slim",
    "setup_commands": ["pip install -r requirements.txt -r requirements-dev.txt"],
    "envs": ["PYTHONPATH=/workspace"]
}
```

### Go

```python
config = {
    "base_image": "golang:1.21",
    "setup_commands": ["go mod download"],
    "envs": ["CGO_ENABLED=0"]
}
```

## エラーハンドリング

| 状況 | 対応 |
|------|------|
| 3回連続レビュー失敗 | Draft PRを作成して終了 |
| 設計不備 | `/request-design-fix` を実行 |
| 環境構築失敗 | `container-use_environment_config` で設定見直し |
| サービス接続失敗 | ポート・環境変数を確認 |

## Sisyphusへの指示

```python
def implement_issue(issue_id):
    # 0. ブランチ作成 (ホスト側)
    branch_name = create_feature_branch(issue_id)
    
    # 1. Container環境構築 (from_git_ref でブランチ指定)
    env = container_use_environment_create(
        from_git_ref=branch_name
    )
    
    # 2. サービス追加
    if needs_database(issue_id):
        add_database_service(env)
    
    # 3. Handover Check
    resolve_handovers_if_any(issue_id)
        
    # 4. TDD Loop (全てcontainer内)
    while not all_tests_pass:
        run_tests_in_container(env)   # Red
        implement_in_container(env)    # Green
        refactor_in_container(env)     # Refactor
    
    # 5. Design Fix Check
    if design_flaw_detected:
        request_design_fix(issue_id)
        return
        
    # 6. Review
    if review_score < 9:
        continue_tdd_loop()
        
    # 7. Commit & Push & PR (container内)
    commit_and_push_in_container(env)
    create_pr_in_container(env)
```

## 参考

- [container-use環境構築ガイド](../skill/container-use-guide.md)
- [申し送り処理ガイド](../skill/handover-process.md)
- [コード品質ルール](../skill/code-quality-rules.md)

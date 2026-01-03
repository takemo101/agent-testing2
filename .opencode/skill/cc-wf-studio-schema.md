# cc-wf-studio ワークフロー JSON スキーマ仕様

Claude Code Workflow Studio用のワークフローJSONを作成するための完全なスキーマ仕様書です。

## 1. 基本構造

ワークフローJSONは`.vscode/workflows/`ディレクトリに保存されます。

```json
{
  "id": "workflow-unique-id",
  "name": "workflow-name",
  "description": "ワークフローの説明",
  "version": "1.0.0",
  "schemaVersion": "1.1.0",
  "nodes": [],
  "connections": [],
  "createdAt": "2025-01-01T00:00:00.000Z",
  "updatedAt": "2025-01-01T00:00:00.000Z",
  "metadata": {
    "tags": ["tag1", "tag2"],
    "author": "author-name"
  }
}
```

### 必須フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | string | ワークフローの一意識別子 |
| `name` | string | ワークフロー名（1-100文字、英数字・ハイフン・アンダースコア） |
| `version` | string | バージョン番号（セマンティックバージョニング形式: X.Y.Z） |
| `nodes` | array | ノードの配列（最大50ノード） |
| `connections` | array | 接続の配列 |
| `createdAt` | string | 作成日時（ISO 8601形式） |
| `updatedAt` | string | 更新日時（ISO 8601形式） |

### オプションフィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `description` | string | ワークフローの説明 |
| `schemaVersion` | string | スキーマバージョン（"1.0.0", "1.1.0", "1.2.0"） |
| `metadata` | object | メタデータ（tags, author等） |

---

## 2. ノードタイプ一覧

### 2.1 Start ノード

ワークフローの開始点。出力ポートのみ持つ。

```json
{
  "id": "start-node",
  "type": "start",
  "name": "start",
  "position": { "x": 50, "y": 300 },
  "data": {
    "label": "開始"
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 出力 | `out` | source |

---

### 2.2 End ノード

ワークフローの終了点。入力ポートのみ持つ。

```json
{
  "id": "end-node",
  "type": "end",
  "name": "end",
  "position": { "x": 800, "y": 300 },
  "data": {
    "label": "終了"
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `in` | target |

---

### 2.3 Prompt ノード

プロンプトテンプレートを定義。Mustache形式の変数（`{{variableName}}`）をサポート。

```json
{
  "id": "prompt-1",
  "type": "prompt",
  "name": "user-input",
  "position": { "x": 200, "y": 300 },
  "data": {
    "label": "ユーザー入力",
    "prompt": "以下の要件について処理してください:\n\n{{requirements}}",
    "variables": {
      "requirements": ""
    }
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `in` | target |
| 出力 | `out` | source |

**data フィールド:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `label` | string | No | 表示ラベル |
| `prompt` | string | Yes | プロンプトテンプレート |
| `variables` | object | No | 変数のデフォルト値 |

---

### 2.4 SubAgent ノード

AIエージェントタスクを定義。

```json
{
  "id": "agent-1",
  "type": "subAgent",
  "name": "task-executor",
  "position": { "x": 400, "y": 300 },
  "data": {
    "description": "タスクを実行するエージェント",
    "prompt": "以下のタスクを実行してください:\n\n...",
    "tools": "Read, Write, Edit, Bash",
    "model": "sonnet",
    "color": "blue",
    "outputPorts": 1
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力 | `output` | source |

**data フィールド:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `description` | string | Yes | エージェントの説明（1-200文字） |
| `prompt` | string | Yes | システムプロンプト（1-10000文字） |
| `tools` | string | No | 許可ツール（カンマ区切り） |
| `model` | string | No | モデル選択: `sonnet`, `opus`, `haiku`, `inherit` |
| `color` | string | No | 色: `red`, `blue`, `green`, `yellow`, `purple`, `orange`, `pink`, `cyan` |
| `outputPorts` | number | Yes | 出力ポート数（常に1） |

**利用可能なツール:**
- `Read` - ファイル読み取り
- `Write` - ファイル書き込み
- `Edit` - ファイル編集
- `Bash` - シェルコマンド実行
- `Glob` - ファイルパターン検索
- `Grep` - テキスト検索

---

### 2.5 IfElse ノード

2分岐の条件分岐。True/False、Yes/No、Success/Errorなどのパターン用。

```json
{
  "id": "ifelse-1",
  "type": "ifElse",
  "name": "condition-check",
  "position": { "x": 600, "y": 300 },
  "data": {
    "evaluationTarget": "前のステップの結果を評価",
    "branches": [
      {
        "id": "branch-true",
        "label": "成功",
        "condition": "処理が成功した場合"
      },
      {
        "id": "branch-false",
        "label": "失敗",
        "condition": "処理が失敗した場合"
      }
    ],
    "outputPorts": 2
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力1 (True) | `branch-0` | source |
| 出力2 (False) | `branch-1` | source |

**data フィールド:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `evaluationTarget` | string | No | 評価対象の説明 |
| `branches` | array | Yes | 分岐条件（常に2つ） |
| `outputPorts` | number | Yes | 出力ポート数（常に2） |

**branches 配列の各要素:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | No | 分岐ID |
| `label` | string | Yes | 分岐ラベル（1-50文字） |
| `condition` | string | Yes | 条件の説明（1-500文字） |

---

### 2.6 Switch ノード

多分岐（2-N）の条件分岐。

```json
{
  "id": "switch-1",
  "type": "switch",
  "name": "multi-branch",
  "position": { "x": 600, "y": 300 },
  "data": {
    "evaluationTarget": "HTTPステータスコードを評価",
    "branches": [
      {
        "id": "branch-200",
        "label": "成功",
        "condition": "ステータスコードが200"
      },
      {
        "id": "branch-400",
        "label": "クライアントエラー",
        "condition": "ステータスコードが4xx"
      },
      {
        "id": "branch-500",
        "label": "サーバーエラー",
        "condition": "ステータスコードが5xx"
      },
      {
        "id": "branch-default",
        "label": "その他",
        "condition": "上記以外の場合",
        "isDefault": true
      }
    ],
    "outputPorts": 4
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力N | `branch-N` | source（Nは0から開始） |

---

### 2.7 AskUserQuestion ノード

ユーザーに質問し、選択肢に応じて分岐。

```json
{
  "id": "ask-1",
  "type": "askUserQuestion",
  "name": "user-choice",
  "position": { "x": 600, "y": 300 },
  "data": {
    "questionText": "次のアクションを選択してください",
    "options": [
      {
        "id": "option-1",
        "label": "続行",
        "description": "処理を続行します"
      },
      {
        "id": "option-2",
        "label": "中止",
        "description": "処理を中止します"
      }
    ],
    "multiSelect": false,
    "useAiSuggestions": false,
    "outputPorts": 2
  }
}
```

**通常モード（multiSelect: false, useAiSuggestions: false）のポート:**

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力N | `branch-N` | source（Nは0から開始、オプション数分） |

**マルチセレクトまたはAI提案モードのポート:**

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力 | `output` | source（単一出力） |

**data フィールド:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `questionText` | string | Yes | 質問文（1-500文字） |
| `options` | array | Yes | 選択肢（2-4個） |
| `multiSelect` | boolean | No | 複数選択可否（デフォルト: false） |
| `useAiSuggestions` | boolean | No | AI提案使用（デフォルト: false） |
| `outputPorts` | number | Yes | 出力ポート数 |

**options 配列の各要素:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | No | オプションID |
| `label` | string | Yes | ラベル（1-50文字） |
| `description` | string | Yes | 説明（1-200文字） |

---

### 2.8 Skill ノード

Claude Code Skillを参照・実行。

```json
{
  "id": "skill-1",
  "type": "skill",
  "name": "pdf-analyzer",
  "position": { "x": 400, "y": 300 },
  "data": {
    "name": "pdf-analyzer",
    "description": "PDFファイルを解析するスキル",
    "skillPath": "~/.claude/skills/pdf-analyzer/SKILL.md",
    "scope": "personal",
    "allowedTools": "Read, Bash",
    "validationStatus": "valid",
    "outputPorts": 1
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力 | `output` | source |

**data フィールド:**

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | Yes | スキル名（小文字、ハイフン可、1-64文字） |
| `description` | string | Yes | 説明（1-1024文字） |
| `skillPath` | string | Yes | SKILL.mdファイルパス |
| `scope` | string | Yes | スコープ: `personal`, `project` |
| `allowedTools` | string | No | 許可ツール |
| `validationStatus` | string | Yes | 検証状態: `valid`, `missing`, `invalid` |
| `outputPorts` | number | Yes | 出力ポート数（常に1） |

---

### 2.9 MCP ノード

Model Context Protocol (MCP) ツールを実行。

```json
{
  "id": "mcp-1",
  "type": "mcp",
  "name": "playwright-navigate",
  "position": { "x": 400, "y": 300 },
  "data": {
    "serverId": "playwright",
    "toolName": "navigate",
    "toolDescription": "指定URLにブラウザで移動",
    "parameters": [
      {
        "name": "url",
        "type": "string",
        "description": "移動先URL",
        "required": true
      }
    ],
    "parameterValues": {
      "url": "https://example.com"
    },
    "validationStatus": "valid",
    "outputPorts": 1
  }
}
```

| ポート | ID | 方向 |
|--------|-----|------|
| 入力 | `input` | target |
| 出力 | `output` | source |

---

## 3. 接続（Connections）

ノード間の接続を定義します。

```json
{
  "id": "conn-unique-id",
  "from": "source-node-id",
  "to": "target-node-id",
  "fromPort": "output-port-id",
  "toPort": "input-port-id",
  "condition": "オプション: 分岐条件ラベル"
}
```

### 接続フィールド

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `id` | string | Yes | 接続の一意識別子 |
| `from` | string | Yes | 接続元ノードID |
| `to` | string | Yes | 接続先ノードID |
| `fromPort` | string | Yes | 接続元ポートID |
| `toPort` | string | Yes | 接続先ポートID |
| `condition` | string | No | 条件ラベル（分岐ノードからの接続時） |

---

## 4. ポートID早見表

| ノードタイプ | 入力ポートID | 出力ポートID |
|-------------|-------------|-------------|
| `start` | なし | `out` |
| `end` | `in` | なし |
| `prompt` | `in` | `out` |
| `subAgent` | `input` | `output` |
| `ifElse` | `input` | `branch-0`, `branch-1` |
| `switch` | `input` | `branch-0`, `branch-1`, `branch-2`, ... |
| `askUserQuestion` (通常) | `input` | `branch-0`, `branch-1`, ... |
| `askUserQuestion` (multiSelect/AI) | `input` | `output` |
| `skill` | `input` | `output` |
| `mcp` | `input` | `output` |

---

## 5. 検証ルール

### ワークフロー全体

- ノード数: 最大50
- ワークフロー名: 1-100文字
- バージョン形式: `X.Y.Z`（セマンティックバージョニング）

### ノード

- ノード名: 1-50文字、`/^[a-zA-Z0-9_-]+$/`
- 位置: `{ x: number, y: number }`

### SubAgent

- description: 1-200文字
- prompt: 1-10000文字
- outputPorts: 1（固定）

### IfElse

- branches: 2つ固定
- outputPorts: 2（固定）

### Switch

- branches: 2-10個
- 最後のbranchに`isDefault: true`を設定可能

### AskUserQuestion

- options: 2-4個
- questionText: 1-500文字
- オプションラベル: 1-50文字
- オプション説明: 1-200文字

### Skill

- name: 1-64文字、`/^[a-z0-9-]+$/`（小文字、数字、ハイフンのみ）

---

## 6. サンプルワークフロー

### 最小構成（Start → SubAgent → End）

```json
{
  "id": "minimal-workflow",
  "name": "minimal-workflow",
  "description": "最小構成のワークフロー",
  "version": "1.0.0",
  "schemaVersion": "1.1.0",
  "nodes": [
    {
      "id": "start",
      "type": "start",
      "name": "start",
      "position": { "x": 50, "y": 200 },
      "data": { "label": "開始" }
    },
    {
      "id": "agent",
      "type": "subAgent",
      "name": "main-agent",
      "position": { "x": 250, "y": 200 },
      "data": {
        "description": "メインタスクを実行",
        "prompt": "与えられたタスクを実行してください。",
        "tools": "Read, Write",
        "model": "sonnet",
        "outputPorts": 1
      }
    },
    {
      "id": "end",
      "type": "end",
      "name": "end",
      "position": { "x": 450, "y": 200 },
      "data": { "label": "終了" }
    }
  ],
  "connections": [
    {
      "id": "conn-1",
      "from": "start",
      "to": "agent",
      "fromPort": "out",
      "toPort": "input"
    },
    {
      "id": "conn-2",
      "from": "agent",
      "to": "end",
      "fromPort": "output",
      "toPort": "in"
    }
  ],
  "createdAt": "2025-01-01T00:00:00.000Z",
  "updatedAt": "2025-01-01T00:00:00.000Z"
}
```

### 条件分岐あり

```json
{
  "id": "branching-workflow",
  "name": "branching-workflow",
  "version": "1.0.0",
  "schemaVersion": "1.1.0",
  "nodes": [
    {
      "id": "start",
      "type": "start",
      "name": "start",
      "position": { "x": 50, "y": 200 },
      "data": { "label": "開始" }
    },
    {
      "id": "checker",
      "type": "subAgent",
      "name": "checker",
      "position": { "x": 200, "y": 200 },
      "data": {
        "description": "条件をチェック",
        "prompt": "条件を確認し、SUCCESS: true/false で結果を出力してください。",
        "outputPorts": 1
      }
    },
    {
      "id": "branch",
      "type": "ifElse",
      "name": "result-check",
      "position": { "x": 400, "y": 200 },
      "data": {
        "evaluationTarget": "チェック結果",
        "branches": [
          { "id": "b-success", "label": "成功", "condition": "SUCCESS: true" },
          { "id": "b-failure", "label": "失敗", "condition": "SUCCESS: false" }
        ],
        "outputPorts": 2
      }
    },
    {
      "id": "success-handler",
      "type": "subAgent",
      "name": "success-handler",
      "position": { "x": 600, "y": 100 },
      "data": {
        "description": "成功時の処理",
        "prompt": "成功処理を実行します。",
        "outputPorts": 1
      }
    },
    {
      "id": "failure-handler",
      "type": "subAgent",
      "name": "failure-handler",
      "position": { "x": 600, "y": 300 },
      "data": {
        "description": "失敗時の処理",
        "prompt": "失敗処理を実行します。",
        "outputPorts": 1
      }
    },
    {
      "id": "end",
      "type": "end",
      "name": "end",
      "position": { "x": 800, "y": 200 },
      "data": { "label": "終了" }
    }
  ],
  "connections": [
    { "id": "c1", "from": "start", "to": "checker", "fromPort": "out", "toPort": "input" },
    { "id": "c2", "from": "checker", "to": "branch", "fromPort": "output", "toPort": "input" },
    { "id": "c3", "from": "branch", "to": "success-handler", "fromPort": "branch-0", "toPort": "input", "condition": "成功" },
    { "id": "c4", "from": "branch", "to": "failure-handler", "fromPort": "branch-1", "toPort": "input", "condition": "失敗" },
    { "id": "c5", "from": "success-handler", "to": "end", "fromPort": "output", "toPort": "in" },
    { "id": "c6", "from": "failure-handler", "to": "end", "fromPort": "output", "toPort": "in" }
  ],
  "createdAt": "2025-01-01T00:00:00.000Z",
  "updatedAt": "2025-01-01T00:00:00.000Z"
}
```

---

## 7. よくある間違いと対処法

### ポートIDの間違い

**誤り:**
```json
{ "fromPort": "output-0", "toPort": "input-0" }
```

**正解:**
```json
{ "fromPort": "output", "toPort": "input" }
```

### Start/Endノードのポート

**誤り:**
```json
{ "from": "start", "fromPort": "output" }
{ "to": "end", "toPort": "input" }
```

**正解:**
```json
{ "from": "start", "fromPort": "out" }
{ "to": "end", "toPort": "in" }
```

### IfElseの分岐ポート

**誤り:**
```json
{ "fromPort": "true" }
{ "fromPort": "false" }
```

**正解:**
```json
{ "fromPort": "branch-0" }
{ "fromPort": "branch-1" }
```

### type名の大文字小文字

**誤り:**
```json
{ "type": "subagent" }
{ "type": "ifelse" }
{ "type": "askuserquestion" }
```

**正解:**
```json
{ "type": "subAgent" }
{ "type": "ifElse" }
{ "type": "askUserQuestion" }
```

---

## 8. 保存場所

ワークフローJSONは以下のパスに保存:

```
.vscode/workflows/{workflow-name}.json
```

エクスポート時に生成されるファイル:

```
.claude/agents/{agent-name}.md
.claude/commands/{workflow-name}.md
```

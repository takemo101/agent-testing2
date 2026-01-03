# コード品質ルール

本ドキュメントは、実装時に遵守すべきコード品質ルールを定義します。
すべてのエージェントはこのルールに従って実装を行います。

---

## 1. 500行ルール

### 1.1 ルール定義

**1ファイルあたり500行を上限**とします。
これはクリーンアーキテクチャ/オニオンアーキテクチャの単一責任の原則に基づきます。

| 条件 | 対応 |
|------|------|
| 500行以下 | OK |
| 500行超過 | **自動分割を実行** |

### 1.2 500行を超えた場合の対応

1. 実装時にエージェントが自動で分割を実行
2. 分割後に品質チェックを再実行
3. すべて500行以下になるまで繰り返し
4. 分割が技術的に困難な場合のみTODOに記録

### 1.3 分割の指針

| レイヤー | 分割方針 |
|---------|---------|
| Domain | Entity → Value Object抽出、Domain Service分離 |
| Application | UseCase分割、Query/Command分離（CQRS） |
| Infrastructure | Repository実装分割、外部サービスAdapter分離 |
| Presentation | Controller分割、DTO/Transformer分離 |
| Frontend | コンポーネント分割、カスタムHook抽出 |

### 1.4 例外

以下の場合は500行超過を許容：

- 自動生成ファイル（マイグレーション、型定義等）
- 設定ファイル
- テストファイル（ただし1000行を上限とする）

---

## 2. 固定アーキテクチャ

### 2.1 バックエンド

以下のアーキテクチャを**必須**とします。

| アーキテクチャ/手法 | 説明 |
|-------------------|------|
| オニオンアーキテクチャ | 依存関係を内側（ドメイン）に向ける層構造 |
| クリーンアーキテクチャ | ビジネスロジックをフレームワーク・DBから独立 |
| TDD（テスト駆動開発） | テストを先に書いてから実装 |

#### 層構造（内側から外側）

```
Domain Layer（最内層）
  └── Entities, Value Objects, Domain Services, Repository Interfaces
      │
Application Layer
  └── Use Cases, Application Services, DTOs
      │
Interface Adapters Layer
  └── Controllers, Gateways, Presenters
      │
Infrastructure Layer（最外層）
  └── Frameworks, DB, External Services
```

#### 依存関係のルール

- **Domain層は他の層に依存しない**（純粋なビジネスロジック）
- Application層はDomain層のみに依存
- Infrastructure層・Interfaces層はApplication層・Domain層に依存
- **外側の層から内側の層への依存のみ許可**（逆は禁止）

#### ディレクトリ構成

```
backend/
├── src/
│   ├── Domain/                 # ドメイン層（最内層）
│   │   ├── Entities/
│   │   ├── ValueObjects/
│   │   ├── Repositories/       # インターフェースのみ
│   │   └── Services/
│   │
│   ├── Application/            # アプリケーション層
│   │   ├── UseCases/
│   │   ├── DTOs/
│   │   └── Services/
│   │
│   ├── Infrastructure/         # インフラストラクチャ層
│   │   ├── Persistence/        # Repository実装
│   │   └── ExternalServices/
│   │
│   └── Interfaces/             # インターフェースアダプター層
│       └── Http/
│           ├── Controllers/
│           ├── Requests/
│           └── Responses/
│
└── tests/
    ├── Unit/
    ├── Integration/
    └── E2E/
```

### 2.2 フロントエンド

以下のアーキテクチャを**必須**とします。

| アーキテクチャ | 説明 |
|--------------|------|
| Atomic Design | UIコンポーネントを5階層で分類 |
| MVVM | Model-View-ViewModelパターン |

#### Atomic Designの5階層

| 階層 | 説明 | 例 |
|-----|------|-----|
| Atoms | 最小単位のUI要素 | Button, Input, Label, Icon |
| Molecules | Atomsを組み合わせた機能単位 | SearchBox, FormField, Card |
| Organisms | 独立したUIセクション | Header, Footer, UserList |
| Templates | ページのレイアウト構造 | DefaultLayout, DashboardLayout |
| Pages | 実際のページコンポーネント | HomePage, UserDetailPage |

#### MVVMパターン

| 層 | 責務 | 実装例 |
|----|-----|-------|
| Model | データ・ビジネスロジック | API Client, Store, Repository |
| ViewModel | 状態管理・UIロジック | Custom Hooks, Store Slice |
| View | UI表示 | React/Vue Component |

#### ディレクトリ構成

```
frontend/
├── src/
│   ├── components/             # Atomic Design
│   │   ├── atoms/
│   │   ├── molecules/
│   │   ├── organisms/
│   │   └── templates/
│   │
│   ├── pages/                  # ページコンポーネント
│   │
│   ├── viewmodels/             # ViewModel層
│   │   ├── hooks/
│   │   └── stores/
│   │
│   ├── models/                 # Model層
│   │   ├── api/
│   │   ├── repositories/
│   │   └── types/
│   │
│   └── utils/
│
└── tests/
```

---

## 3. 設計原則

### 3.1 SOLID原則

| 原則 | 説明 | チェック項目 |
|------|------|-------------|
| **S**ingle Responsibility | 単一責任 | 各クラス・関数は1つの責務のみ |
| **O**pen/Closed | 開放閉鎖 | 拡張に開き、修正に閉じている |
| **L**iskov Substitution | リスコフ置換 | サブタイプは親を代替可能 |
| **I**nterface Segregation | インターフェース分離 | 必要なインターフェースのみ実装 |
| **D**ependency Inversion | 依存性逆転 | 抽象に依存、具象に依存しない |

### 3.2 その他の原則

| 原則 | 説明 |
|------|------|
| DRY | Don't Repeat Yourself（重複を避ける） |
| YAGNI | You Aren't Gonna Need It（必要になるまで作らない） |
| KISS | Keep It Simple, Stupid（シンプルに保つ） |

---

## 4. 命名規則

### 4.1 共通

| 対象 | 規則 | 例 |
|------|------|-----|
| ファイル名 | kebab-case | `user-repository.ts` |
| ディレクトリ名 | kebab-case | `use-cases/` |
| 定数 | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT` |

### 4.2 バックエンド

| 対象 | 規則 | 例 |
|------|------|-----|
| クラス名 | PascalCase | `UserRepository` |
| メソッド名 | camelCase | `findById()` |
| 変数名 | camelCase | `userName` |
| テーブル名 | snake_case, 複数形 | `users` |
| カラム名 | snake_case | `created_at` |

### 4.3 フロントエンド

| 対象 | 規則 | 例 |
|------|------|-----|
| コンポーネント | PascalCase | `UserCard.tsx` |
| カスタムHook | camelCase, use接頭辞 | `useAuth()` |
| 変数名 | camelCase | `isLoading` |
| イベントハンドラ | handle接頭辞 | `handleClick` |

---

## 5. テストカバレッジ要件

### 5.1 カバレッジ閾値

| 対象 | 閾値 |
|------|------|
| 新規コード | **80%以上** |
| 全体 | 70%以上（推奨） |

### 5.2 テストの種類

| 種類 | 対象 | 優先度 |
|------|------|--------|
| Unit Test | ドメインロジック、ユーティリティ | 高 |
| Integration Test | API、DB連携 | 高 |
| E2E Test | ユーザーシナリオ | 中 |

---

## 6. コードフォーマット

### 6.1 必須ツール

| 言語/フレームワーク | Linter | Formatter |
|-------------------|--------|-----------|
| TypeScript | ESLint | Prettier |
| JavaScript | ESLint | Prettier |
| PHP | PHP_CodeSniffer | PHP-CS-Fixer |
| Python | Ruff / Flake8 | Black |
| Go | golangci-lint | gofmt |

### 6.2 コミット前チェック

以下のチェックをコミット前に実行：

1. Lint（エラー0件）
2. Format（差分0件）
3. Type Check（エラー0件）
4. Unit Test（全件パス）

---

## 7. 禁止事項

### 7.1 絶対禁止

| 禁止事項 | 理由 |
|---------|------|
| `any` 型の使用（TypeScript） | 型安全性の破壊 |
| `@ts-ignore` / `@ts-expect-error` | 型エラーの隠蔽 |
| 空のcatchブロック `catch(e) {}` | エラーの握りつぶし |
| 機密情報のハードコード | セキュリティリスク |
| `console.log` の本番コード残存 | デバッグコードの漏洩 |
| テスト削除による「修正」 | 品質の低下 |

### 7.2 原則禁止（例外要申請）

| 禁止事項 | 例外条件 |
|---------|---------|
| 新規依存パッケージの追加 | 既存で代替不可の場合のみ |
| グローバル状態の使用 | 明確な理由がある場合のみ |
| 直接DOM操作（React/Vue） | パフォーマンス上必要な場合のみ |

---

## 変更履歴

| 日付 | バージョン | 変更内容 |
|:---|:---|:---|
| 2026-01-02 | 1.0.0 | 初版作成（ai-frameworkからの取り込み） |

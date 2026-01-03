# 基本設計・完全ワークフロー

要件定義書を入力として基本設計書を作成し、スペシャリストによるレビュー（合格基準9点）を通過するまで修正を繰り返す完全自動化ワークフローです。

## 入力
$ARGUMENTS (要件定義書のパスなど)

## 前提条件
- 要件定義書が8点以上でレビュー合格済みであること
- 要件定義書に技術スタックが定義されていること（未定義の場合は未解決課題として扱う）
- **推奨**: `/tech-catchup-workflow` で技術調査レポートが作成済みであること
  - 技術調査レポートがある場合、Phase 0.5のヒアリングで参照される
  - 未実施でもワークフローは続行可能（ただし最新情報の把握漏れリスクあり）

## サーキットブレーカー
- **最大リトライ回数**: **3回**
- **ギブアップ条件**: 3回修正しても9点に届かない場合、現状ファイルに警告マークを付与して終了
- **スコア悪化検知**: 前回より低下した場合、即座に中断

---

## 実行プロセス

### Phase 0: 技術スタック完全性チェック【必須】

要件定義書の技術スタックを検証する。

**チェック項目:**

| レイヤー | 必須/推奨 | 未定義時のアクション |
|---------|----------|---------------------|
| フロントエンド | 必須 | **→ Phase 0.5 ヒアリング実施** |
| バックエンド | 必須 | **→ Phase 0.5 ヒアリング実施** |
| データベース | 必須 | **→ Phase 0.5 ヒアリング実施** |
| 認証方式 | 必須 | **→ Phase 0.5 ヒアリング実施** |
| UIライブラリ | 推奨 | ヒアリングで確認 |
| 状態管理 | 推奨 | ヒアリングで確認 |
| ORM | 推奨 | ヒアリングで確認 |
| インフラ/クラウド | 推奨 | ヒアリングで確認 |
| CI/CD | 推奨 | ヒアリングで確認 |
| テスト | 推奨 | ヒアリングで確認 |
| モニタリング | 推奨 | ヒアリングで確認 |

---

### Phase 0.5: 技術スタック選定ヒアリング【未定義時必須】

**トリガー**: Phase 0で必須レイヤーが1つでも未定義の場合

`basic-design-writer` の**技術スタック選定ヒアリングフェーズ**を実行：

1. **技術調査レポート確認**: `docs/research/TECH-*.md` が存在する場合、内容を参照
   - Breaking Changes、非推奨機能を考慮
   - 新機能の活用可能性を検討
2. **要件分析**: 要件定義書からパフォーマンス/セキュリティ/可用性要件を抽出
3. **Web検索**: ドメイン・規模に適した技術スタックのベストプラクティスを調査
   - 技術調査レポートがない技術については重点的に調査
4. **提案形式ヒアリング**: 各レイヤーについて選択肢と推奨を提示し、ユーザーに確認
   - 技術調査レポートの知見を反映した提案を行う
5. **選定確定**: ユーザー回答を元に技術スタックを確定

**ヒアリング対象（11レイヤー）:**

| # | レイヤー | 例 |
|---|---------|-----|
| 1 | フロントエンド | Next.js / Nuxt.js / SvelteKit |
| 2 | UIライブラリ | shadcn/ui / MUI / Chakra UI |
| 3 | 状態管理 | TanStack Query+Zustand / Redux |
| 4 | バックエンド | Node.js / Go / Python |
| 5 | データベース | PostgreSQL / MySQL / MongoDB |
| 6 | ORM | Prisma / Drizzle / TypeORM |
| 7 | 認証 | NextAuth.js / Clerk / 自前JWT |
| 8 | インフラ | Vercel+Supabase / AWS / GCP |
| 9 | CI/CD | GitHub Actions / GitLab CI |
| 10 | テスト | Vitest+Playwright |
| 11 | モニタリング | Sentry / Datadog |

**スキップ条件:**
- 要件定義書で全レイヤー定義済み
- ユーザーから「技術スタック確定済み」と明示的指示
- 既存コードベースがあり技術スタック自明

> **重要**: ヒアリング後も未確定の項目は「要選定（I-XXX）」と明記すること。

**仮定の明示ルール:**

基本設計書で仮定を置く場合は以下のフォーマットを使用：
- 「要選定（I-XXX）」と記載し、未解決課題IDを付与
- 仮定で進める場合は「【仮定】」プレフィックスを使用
- 例: 「【仮定】React/Next.js（I-007で確定予定）」

---

### Phase 1: コンテキスト解析 & ドラフト作成 (basic-design-writer)

1. **要件解析**: 要件定義書から機能一覧、非機能要件、技術制約を抽出
2. **技術スタック検証**: Phase 0の結果を反映
3. **ドラフト作成**: `docs/designs/basic/BASIC-[カテゴリ]-[連番]_[機能名].md` を作成

**ドラフトに含めるべき内容:**
- システムアーキテクチャ図
- 機能一覧（機能ID付き）
- **画面一覧**（画面ID付き）← 詳細設計で使用
- API一覧
- データモデル概要
- 技術スタック（未選定項目は「要選定」と明記）

---

### Phase 2: 品質保証ループ (basic-design-reviewer)

1. **レビュー**: `basic-design-reviewer` エージェントがレビュー
   - 要件との整合性
   - アーキテクチャの妥当性
   - **技術スタックの網羅性**（追加チェック項目）
   - **仮定の明示**（未選定技術は「要選定」と明記されているか）
   - **要件との整合性**（要件定義書の技術制約と一致しているか）
2. **判定**:
   - **スコア >= 9**: 合格。Phase 3へ
   - **スコア < 9**: 修正ループ（最大3回）

**レビュー観点チェックリスト:**
```
【技術スタック完全性】★強化項目
□ 全レイヤー（FE/BE/DB/インフラ）の技術が定義されているか
□ 技術スタック選定ヒアリングが実施されたか（未定義時）
□ 各技術の選定理由が明記されているか（要件との適合性）
□ 未選定の技術は「要選定（I-XXX）」と明記されているか
□ 要件定義書の技術制約と一致しているか
□ 仮定を置いている場合「【仮定】」プレフィックスがあるか
□ UIライブラリ・状態管理・ORM・テスト等の推奨レイヤーも考慮されているか

【アーキテクチャ】
□ システム構成図が明確か
□ コンポーネント間の依存関係が適切か
□ スケーラビリティが考慮されているか
□ 選定技術がアーキテクチャ図に反映されているか

【機能・画面一覧】
□ 全機能にIDが付与されているか
□ 全画面にIDが付与されているか（詳細設計で使用）
□ 優先度・フェーズが設定されているか
```

---

### Phase 3: 詳細設計準備

1. **機能抽出**: 基本設計書の機能一覧から、詳細設計が必要な単位を特定
2. **フォルダ作成**: `docs/designs/detailed/{機能名}/` などのフォルダ構造を自動生成
3. **リンク設定**: 基本設計書に詳細設計へのリンクを追加

**フォルダ構成:**
```
docs/designs/detailed/{親機能名}/
├── README.md           # 概要・インデックス
├── {サブ機能1}/
├── {サブ機能2}/
└── 共通/
    ├── データベース設計書.md
    ├── インフラ設計書.md
    └── セキュリティ設計書.md
```

---

## エラーハンドリング & リカバリ

### サーキットブレーカー発動時

| 状況 | 対処法 |
|------|--------|
| 3回修正しても9点未満 | 現状ファイルに `<!-- ⚠️ REVIEW_FAILED: 手動修正が必要 -->` マークを付与。問題点サマリーを出力 |
| スコア悪化を検知 | 即座に中断。前回の修正内容をロールバック検討 |
| 技術スタック未定義 | 未解決課題として記録し、続行。基本設計書に「要選定」と明記 |

### 手動リカバリ手順

```
1. 問題点サマリーを確認
2. 指摘された箇所を手動で修正
3. 以下のコマンドで再開:
   /basic-design-workflow "要件定義書パス" --resume-from=phase2
```

---

## 完了チェックリスト

```
【基本設計書完了チェック】
□ スコア9点以上で合格
□ 技術スタック選定ヒアリング実施済み（未定義レイヤーがあった場合）
□ 技術スタックが全レイヤーで定義（または「要選定」明記）
□ 各技術の選定理由が記載されている
□ 機能一覧に全機能IDが付与
□ 画面一覧に全画面IDが付与
□ 詳細設計フォルダが作成済み
□ 基本設計書に詳細設計へのリンクが設定済み
```

---

## Sisyphusへの指示

```python
def basic_design_workflow(req_path):
    # Phase 0: 技術スタック完全性チェック
    tech_stack = validate_tech_stack(req_path)
    missing_layers = []
    for layer in ['frontend', 'backend', 'database', 'auth']:
        if not tech_stack.get(layer):
            missing_layers.append(layer)
    
    # Phase 0.5: 技術スタック選定ヒアリング（未定義時）
    if missing_layers:
        # Web検索でベストプラクティス調査
        best_practices = web_search(f"{domain} tech stack best practices 2026")
        
        # 提案形式でユーザーにヒアリング
        tech_proposal = basic_writer.generate_tech_stack_proposal(
            req_path, 
            missing_layers, 
            best_practices
        )
        # ユーザー回答を待機
        user_selections = await_user_response(tech_proposal)
        
        # 選定結果を反映
        tech_stack = merge_tech_stack(tech_stack, user_selections)
        
        # まだ未確定の項目は I-XXX として記録
        unresolved_issues = []
        for layer in missing_layers:
            if not tech_stack.get(layer):
                issue_id = generate_issue_id()
                unresolved_issues.append(f"I-{issue_id}: {layer}技術選定")
    else:
        unresolved_issues = []
    
    # Phase 1: ドラフト作成（確定した技術スタックを反映）
    basic_path = basic_writer.create_draft(req_path, tech_stack, unresolved_issues)
    
    # Phase 2: 品質保証ループ
    history = []
    for i in range(4):
        score, feedback = basic_reviewer.review(basic_path)
        
        if score >= 9:
            # Phase 3: 詳細設計準備
            prepare_detailed_design_folders(basic_path)
            return success(basic_path, score)
            
        if i > 0 and score < history[-1]:
            mark_as_failed(basic_path, "Score degraded")
            return fail("Score degraded")
            
        if i == 3:
            mark_as_failed(basic_path, "Max retries reached")
            return fail("Max retries reached")
            
        history.append(score)
        basic_writer.fix(basic_path, feedback)

    return fail("Unknown error")
```

---

## 関連ドキュメント

- 前工程: `/req-workflow`
- **推奨前工程**: `/tech-catchup-workflow`（技術キャッチアップ）
- 次工程: `/detailed-design-workflow`
- 技術スタック定義: `@design-document-types`
- 技術調査レポート: `docs/research/TECH-*.md`

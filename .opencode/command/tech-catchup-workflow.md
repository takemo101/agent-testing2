---
model: google/antigravity-gemini-3-flash
description: 対象技術の最新情報を調査し、技術調査レポートを作成する
---

# 技術キャッチアップ・完全ワークフロー

対象技術に関する最新情報を調査し、技術調査レポートを作成する自動化ワークフローです。
基本設計フェーズ前に実行することで、技術選定の精度向上と設計ミスの予防を図ります。

> **使用モデル**: Gemini 3 Flash（コスト効率重視）

## 入力

$ARGUMENTS（対象技術リスト、調査深度など）

### 入力形式

以下の形式で入力を受け付けます：

```
技術: [調査対象技術のリスト（カンマ区切り）]
深度: [quick/standard/deep]（省略時: standard）
要件: [要件定義書のパス（任意）]
目的: [調査の目的・背景（任意）]
```

**入力例:**
```
技術: Next.js 15, Prisma, TanStack Query v5
深度: standard
要件: docs/requirements/REQ-FT-001_ECサイト.md
目的: 半年ぶりの技術更新、Breaking Changes確認
```

**簡易入力（技術名のみ）:**
```
React 19, Bun
```

---

## 調査深度の定義

| 深度 | 調査範囲 | 所要時間目安 | 成果物 |
|------|---------|------------|--------|
| **quick** | 最新バージョン・Breaking Changesのみ | 5-10分 | 簡易レポート |
| **standard** | 上記 + 新機能・非推奨・マイグレーション | 15-30分 | 標準レポート |
| **deep** | 上記 + ベストプラクティス・エコシステム動向・競合比較 | 30-60分 | 詳細レポート |

---

## 実行プロセス

### Phase 1: 調査対象の特定と優先度付け

**実行内容:**
1. 入力から調査対象技術を抽出
2. 要件定義書が指定されている場合、追加の技術キーワードを抽出
3. 既存プロジェクトの `package.json` / `Cargo.toml` 等から現行バージョンを確認
4. 調査優先度を決定

**優先度判定基準:**

| 優先度 | 条件 | 例 |
|--------|------|-----|
| 高 | メジャーバージョン差あり | Next.js 13 → 15 |
| 高 | 未使用技術（新規導入） | 初めてのRust |
| 中 | マイナーバージョン差あり | React 18.2 → 18.3 |
| 中 | 1年以上触れていない | 久しぶりのDocker |
| 低 | 現行最新、日常的に使用 | 毎日使うTypeScript |

**スキップ条件:**
- 全技術が「低」優先度 かつ ユーザーが明示的にスキップを許可
- 直近30日以内に同一技術の調査レポートが存在

---

### Phase 2: 最新情報収集

**Gemini 3 Flash + Web検索ツールを活用して以下を調査:**

> **モデル選定理由**: 
> - Web検索との統合が優秀
> - 大量ドキュメントの高速処理
> - コスト効率（Claude比 約1/10）

1. **公式ドキュメント確認**
   - 最新バージョン番号とリリース日
   - Changelog / Release Notes
   - マイグレーションガイド

2. **Breaking Changes特定**
   - API変更
   - 非推奨（Deprecated）機能
   - 削除された機能
   - 挙動変更

3. **新機能調査**（standard以上）
   - 注目の新機能
   - パフォーマンス改善
   - DX向上ポイント

4. **エコシステム調査**（deep のみ）
   - 関連ライブラリの対応状況
   - コミュニティの動向
   - 競合技術との比較

**調査ソース:**
- 公式ドキュメント（最優先）
- GitHub Releases / Changelog
- 公式ブログ
- RFC / Proposal（deep のみ）

---

### Phase 3: 技術調査レポート作成

**出力先:** `docs/research/TECH-[カテゴリ]-[連番]_[技術名].md`

**カテゴリ定義:**

| カテゴリ | 対象 |
|---------|------|
| FE | フロントエンドフレームワーク・ライブラリ |
| BE | バックエンドフレームワーク・ライブラリ |
| DB | データベース・ORM |
| INFRA | インフラ・DevOps・コンテナ |
| LANG | プログラミング言語・ランタイム |
| TOOL | 開発ツール・ビルドツール |

**レポート構成:**

```markdown
# 技術調査レポート: [技術名]

| 項目 | 内容 |
|------|------|
| 調査日 | YYYY-MM-DD |
| 調査深度 | quick / standard / deep |
| 対象バージョン | vX.Y.Z（調査時最新） |
| 現行バージョン | vA.B.C（プロジェクト使用中） |

## エグゼクティブサマリー

[1-3行で調査結果の要点]

## バージョン情報

| バージョン | リリース日 | サポート状況 |
|-----------|-----------|-------------|
| vX.Y.Z | YYYY-MM-DD | Active / LTS / EOL |

## Breaking Changes

### 🚨 高影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| ... | ... | ... |

### ⚠️ 中影響

...

### ℹ️ 低影響

...

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| ... | ... | ... |

## 非推奨・削除予定

| 機能 | 状況 | 代替手段 | 削除予定 |
|------|------|---------|---------|
| ... | Deprecated | ... | vX.Y.Z |

## 設計への影響

### 必須対応

- [ ] [対応事項1]
- [ ] [対応事項2]

### 推奨対応

- [ ] [対応事項]

### 検討事項

- [検討が必要な事項]

## マイグレーション手順（該当時）

```bash
# 手順例
npm install package@latest
npx migrate-tool
```

## 参考リンク

- [公式ドキュメント](URL)
- [マイグレーションガイド](URL)
- [Changelog](URL)

---

**関連ドキュメント:**
- 要件定義書: [リンク]
- 基本設計書: [リンク]（作成後追記）
```

---

### Phase 4: 基本設計への引き継ぎ

**実行内容:**

1. **技術調査レポートの要約生成**
   - 設計に影響する重要ポイントを抽出
   - 未解決課題（I-XXX）として記録

2. **基本設計ワークフローへの連携準備**
   - 技術調査レポートへのリンクを準備
   - 技術スタック選定ヒアリングへのインプット整理

3. **次ステップの案内**

---

## サーキットブレーカー

| 条件 | アクション |
|------|----------|
| 公式ドキュメントにアクセス不可 | 代替ソース使用 + 警告表示 |
| 情報が古い可能性あり | `⚠️ 要確認` マークを付与 |
| 調査時間超過（deep: 90分以上） | 中間レポート出力して終了 |

---

## 最終出力

```markdown
## 技術キャッチアップ完了報告

### 実行サマリー

| 項目 | 内容 |
|------|------|
| 調査技術数 | X件 |
| 調査深度 | standard |
| 所要時間 | XX分 |

### 調査結果一覧

| 技術 | 現行 | 最新 | 影響度 | レポート |
|------|------|------|--------|---------|
| Next.js | 14.0.0 | 15.1.0 | 🔴 高 | [リンク] |
| Prisma | 5.0.0 | 5.5.0 | 🟡 中 | [リンク] |
| TanStack Query | v4 | v5 | 🔴 高 | [リンク] |

### 設計への影響サマリー

#### 🚨 必須対応（基本設計前に決定必要）

1. **Next.js 15**: App Router完全移行が必要。Pages Router非推奨
2. **TanStack Query v5**: API大幅変更。既存コード要修正

#### ⚠️ 注意事項

- Prisma 5.5: 新しいクエリエンジン。パフォーマンステスト推奨

#### 未解決課題

| ID | 課題 | 対応方針 |
|----|------|---------|
| I-XXX | Next.js 14→15移行戦略 | 基本設計で決定 |

### 成果物

- docs/research/TECH-FE-001_Next.js.md
- docs/research/TECH-BE-002_Prisma.md
- docs/research/TECH-FE-003_TanStack-Query.md

### 次のステップ

1. 技術調査レポートの確認
2. 不明点があればユーザーに確認
3. `/basic-design-workflow` の実行（技術調査レポートを参照）
```

---

## エラーハンドリング

| 状況 | 対処法 |
|------|--------|
| 技術名が曖昧 | 候補を提示してユーザーに確認 |
| 公式ドキュメントなし | GitHub / npm / crates.io から情報収集 |
| バージョン情報取得失敗 | 手動入力を依頼 |
| 深度「deep」で時間超過 | 中間成果を保存して続行オプション提示 |

---

## Sisyphusへの指示

```python
def tech_catchup_workflow(input_args):
    # Phase 1: 調査対象の特定
    technologies = parse_technologies(input_args)
    req_path = input_args.get('requirements')
    depth = input_args.get('depth', 'standard')
    
    if req_path:
        # 要件定義書から追加の技術キーワード抽出
        additional_techs = extract_tech_keywords(req_path)
        technologies.extend(additional_techs)
    
    # 既存プロジェクトのバージョン確認
    current_versions = detect_current_versions()
    
    # 優先度付け
    prioritized = prioritize_technologies(technologies, current_versions)
    
    # スキップ判定
    if all(t.priority == 'low' for t in prioritized):
        if await confirm_skip():
            return skip_with_summary()
    
    # Phase 2: 最新情報収集（Gemini 3 Flash + Web検索）
    reports = []
    for tech in prioritized:
        # 並列実行可能
        # 使用ツール: websearch_exa, context7, webfetch
        research = gemini_flash.research(
            technology=tech.name,
            depth=depth,
            tools=['websearch_exa', 'context7_query-docs', 'webfetch'],
            focus=[
                'latest_version',
                'breaking_changes',
                'new_features' if depth != 'quick' else None,
                'ecosystem' if depth == 'deep' else None
            ]
        )
        reports.append(research)
    
    # Phase 3: レポート作成
    report_paths = []
    for tech, research in zip(prioritized, reports):
        category = determine_category(tech)
        next_id = get_next_report_id(category)
        path = f"docs/research/TECH-{category}-{next_id}_{tech.name}.md"
        
        create_report(path, tech, research, depth)
        report_paths.append(path)
    
    # Phase 4: 基本設計への引き継ぎ
    impact_summary = generate_impact_summary(reports)
    unresolved_issues = extract_unresolved_issues(reports)
    
    # 次ステップへの連携情報を準備
    handoff = {
        'reports': report_paths,
        'impact_summary': impact_summary,
        'unresolved_issues': unresolved_issues
    }
    
    return success(handoff)
```

---

## 関連ドキュメント

- 前工程: `/req-workflow`（要件定義）
- 次工程: `/basic-design-workflow`（基本設計）
- 参照: `.opencode/README.md`（ワークフロー全体図）

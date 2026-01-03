# {機能名} コンテナ設計書

## 1. コンテナ構成

### 1.1 サービス一覧
| サービス名 | ベースイメージ | ポート | 用途 |
|-----------|--------------|--------|------|
| api | node:20-alpine | 8080 | APIサーバー |
| worker | node:20-alpine | - | バックグラウンドジョブ |

## 2. Dockerfile設計

### 2.1 APIサービス
```dockerfile
# ビルドステージ
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

# 実行ステージ
FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
EXPOSE 8080
USER node
CMD ["node", "dist/main.js"]
```

## 3. ECS Task Definition

### 3.1 リソース設定
| パラメータ | 開発環境 | 本番環境 |
|-----------|---------|---------|
| CPU | 256 (0.25 vCPU) | 512 (0.5 vCPU) |
| メモリ | 512 MB | 1024 MB |
| 最小タスク数 | 1 | 2 |
| 最大タスク数 | 2 | 10 |

### 3.2 ヘルスチェック
| パラメータ | 値 |
|-----------|-----|
| パス | /health |
| 間隔 | 30秒 |
| タイムアウト | 5秒 |
| 正常閾値 | 2 |
| 異常閾値 | 3 |

## 4. 環境変数・シークレット

### 4.1 環境変数
| 変数名 | 説明 | ソース |
|--------|------|--------|
| NODE_ENV | 実行環境 | タスク定義 |
| PORT | リッスンポート | タスク定義 |
| LOG_LEVEL | ログレベル | タスク定義 |

### 4.2 シークレット（Secrets Manager）
| 変数名 | シークレット名 |
|--------|---------------|
| DATABASE_URL | {project}/db-credentials |
| JWT_SECRET | {project}/jwt-secret |

---

## 変更履歴

| 日付 | バージョン | 変更内容 | 担当者 |
|:---|:---|:---|:---|
| YYYY-MM-DD | 1.0.0 | 初版作成 | - |

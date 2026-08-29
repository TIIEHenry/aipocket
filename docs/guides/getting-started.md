---
title: "快速开始"
type: guide
status: current
updated: 2026-08-18
summary: "Docker 与本地开发的最短路径；生产细节见部署指南"
---

# 快速开始

仅用于**已获授权**的安全研究。免责声明见根 [README.md](../../README.md)。

## Docker（推荐）

```bash
cp .env.example .env
# 编辑 .env：设置 WEB_PASSWORD、WEB_JWT_SECRET，至少配置 FOFA_KEYS 或 SHODAN_KEYS 之一

docker compose up -d --build

# 前端: http://localhost:3080
# API:  http://localhost:8000
```

可选定时扫描：

```bash
docker compose --profile watch up -d backend-watch
```

Compose 会覆盖容器内的 `DATABASE_URL` 与 `DEDUP_REDIS_URL`，指向捆绑的 PostgreSQL / Redis。数据在 `/data/aipocket`。禁止 `docker compose down -v`。

## 本地开发

```bash
cp .env.example .env
cargo test --workspace
cargo run -p aipocket -- serve --host 127.0.0.1 --port 8000

# 另一个终端
cd frontend
pnpm install
pnpm dev
```

本地需要本机 PostgreSQL / Redis，或把 `.env` 指到已有实例。Vite 开发服务器会把 `/api` 代理到后端。

下一步：[配置](configuration.md) · [部署](deploy.md) · [Web UI](web-ui.md)

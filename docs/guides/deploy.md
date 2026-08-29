---
title: "部署"
type: guide
status: current
updated: 2026-08-18
summary: "生产 Docker、数据持久化约束、以及可复现的本机编译方案"
---

# 部署

上游推荐 Docker。本机绝对路径、口令放 `docs/local/`（不入库）。

## Docker 生产

```bash
cp .env.example .env
# 设置 WEB_PASSWORD、WEB_JWT_SECRET；生产把 WEB_CORS_ORIGINS 设为真实前端源

docker compose -f docker-compose.yml up -d --build
```

| 入口 | 默认 |
|------|------|
| Web UI | `http://<host>:3080`（frontend 容器 Nginx） |
| API | `http://<host>:8000` |

更新应用（保留数据）：

```bash
docker compose build backend frontend && docker compose up -d backend frontend
```

### 持久化（硬约束）

| 路径 / 服务 | 用途 |
|-------------|------|
| `/data/aipocket` | 后端结果目录 |
| `/data/aipocket/pg` | PostgreSQL 数据 |
| Redis volume | 去重缓存 |

禁止：`docker compose down -v`、删除数据目录、flush Redis、改生产库名/端口以「换新」。Schema 变更必须幂等可加。

## 本机编译（可选）

在无法或不愿拉 Docker Hub 镜像时，可本机编译：

1. Rust（见 `rust-toolchain.toml`）、Node、pnpm、本机 PostgreSQL 16、Redis 7。
2. `cargo build --release --locked -p aipocket`
3. `cd frontend && pnpm install --frozen-lockfile && pnpm build`
4. 用 `.env` 指向本机 `DATABASE_URL` / `DEDUP_REDIS_URL`
5. `./target/release/aipocket serve --host 127.0.0.1 --port 8000`
6. 前端用 `pnpm preview` 或 Nginx，把 `/api` 反代到 8000

实例口令、启停脚本、本机端口记录写在 `docs/local/`，不要提交。

## 相关文档

- [快速开始](getting-started.md)
- [配置](configuration.md)
- [日常运维](operations.md)
- [存储](../systems/storage.md)

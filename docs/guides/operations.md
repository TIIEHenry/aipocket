---
title: "日常运维"
type: guide
status: current
updated: 2026-08-18
summary: "备份恢复、定时扫描、日志与常见故障"
---

# 日常运维

## 备份与恢复

PostgreSQL 是结果真源：

```bash
docker compose exec -T postgres pg_dump -U aipocket aipocket > backup.sql
docker compose exec -T postgres psql -U aipocket -d aipocket < backup.sql
```

- Redis 只是去重缓存 **和** scan lease。丢缓存会重复劳动；lease 残留会让下一轮报 `scan already running`。不要 flush Redis「清理环境」。僵锁：确认没有 scan 进程后删 `aipocket:scan:lock`，或等 TTL。

## 定时扫描

```bash
docker compose --profile watch up -d backend-watch
```

或 `aipocket watch`。间隔见 `SCHEDULER_INTERVAL`。同一时刻只应有一个 scan：Redis scan lease（`SCAN_LOCK_TTL`）防止重叠。

## 日志

- Docker：`docker compose logs -f backend`
- 扫描进行中：Web「执行扫描」页走 SSE `/api/scan/logs/stream`；结束后全文写入 `runs.log` 列
- 本机进程：把 stdout 重定向到自己的日志文件（记录在 `docs/local/`，不入库）

## 常见问题

| 现象 | 处理 |
|------|------|
| 前端能开、API 连不上 | 确认 8000 在听；前端 `/api` 是否代理/反代到后端 |
| GitHub 显示 disabled / no tokens | `.env` 有 `GITHUB_TOKENS` 后必须重启后端 |
| 改设置不生效 | Web 保存写回 `.env`；敏感字段不要把 UI 打码 `****` 写回去 |
| 扫描卡住 / 第二实例起不来 | 看 Redis `aipocket:scan:lock`；确认没有第二个 `watch`/`scan` |
| 磁盘涨 | spill 表与 `request_ledger` 随 run 增长；删 run 走 API/`DELETE /api/runs/{id}`，不要手改 volume |

## 相关文档

- [部署](deploy.md)
- [存储](../systems/storage.md)
- [Web UI](web-ui.md)

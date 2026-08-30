---
title: "Liveness/readiness 分离；敏感操作审计不落明文"
type: decision
status: accepted
updated: 2026-08-30
created: 2026-08-30
summary: "/api/health 保活；/api/ready 探 PG；reveal/chat/export/restart 写 audit_events（仅打码与元数据）"
---

# ADR-004：readiness 与敏感操作审计

## 背景

`GET /api/health` 恒返回 `{ok:true}`，编排器无法区分「进程活着」和「PostgreSQL 已挂」。`run_log` 在 async handler 里对 tokio `RwLock` 调用 `blocking_read()`，写锁占用时会卡住 worker。reveal / chat / export / restart 与普通列表共用同一把 JWT，没有留下「谁对哪把打码 key 做了什么」的记录。

## 决策

1. **`GET /api/health` 不变**：未鉴权 liveness `{ok:true}`。
2. **新增 `GET /api/ready`（未鉴权）**：探测依赖。
   - `postgres`：`DATABASE_URL` 空 → `"skipped"`；已配置则 `SELECT 1`，失败 → `"error"`。
   - `redis`：`DEDUP_ENABLED=false` → `"skipped"`；否则 `PING`，失败 → `"error"`。
   - HTTP **503 仅当 postgres 已配置且探测失败**（结果无法落库）。Redis 失败保持 200 且 `degraded: true`，与现有「Redis 不可达则不去重、继续扫」一致。
3. **敏感操作审计**：`reveal`、`high_value_reveal`、`chat`、`export`、`restart` 在成功路径（restart 在 `exit` 之前）写入：
   - 始终 `tracing::info!(action, client, ...)`
   - 有 PG 时插入 `audit_events`（`IF NOT EXISTS` 加表，可加不可减）
   - 无 PG 或写入失败：只留 tracing，**不阻断**原操作
4. **禁止明文密钥入审计**。`detail` JSON 只含打码 key、`run_id`、`result_id`、`model`、`dataset`、`format`、导出条数等。
5. **`run_log` 的 JSONL 回退**改用 `.read().await`，禁止在 async 路径 `blocking_read`。
6. **`GET /api/audit?limit=`**（需 JWT）返回最近事件，默认 100、上限 500。单用户模型下 `client` 与登录节流相同：`X-Forwarded-For` 第一段，否则 `"unknown"`。

## 备选

- **健康检查混在 `/api/health`**：破坏现有探活契约。否决。
- **Redis 失败也 503**：与 fail-open 去重策略矛盾。否决。
- **审计必须成功否则拒绝 reveal**：无 PG 部署会失去 reveal。否决。

## 关联

- 实施方案：[dev/plans/readiness-and-audit.md](../plans/readiness-and-audit.md)
- [Web 鉴权](../../docs/systems/auth.md) · [存储](../../docs/systems/storage.md) · [安全边界](../../docs/guides/security.md)

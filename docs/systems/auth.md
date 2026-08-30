---
title: "Web 鉴权"
type: architecture
status: current
updated: 2026-08-30
summary: "全局密码换 JWT、登录节流、client_key、未鉴权就绪探测与敏感操作审计"
---

# Web 鉴权

实现：`crates/aipocket-api/src/auth.rs`。

## 模型

- 没有多用户。`WEB_PASSWORD` 是唯一口令；比较用恒定时间 `ct_eq`。
- `POST /api/auth/login` → JWT HS256（`WEB_JWT_SECRET`），`exp = now + WEB_TOKEN_TTL`。
- 其余 `/api/*`（除 `/api/health`、`/api/ready` 与 login）用 `Authorization: Bearer <token>`。
- `POST /api/auth/logout` 只让客户端丢 token；服务端无黑名单，过期前旧 JWT 仍有效。

## `client_key`

`auth::client_key(&HeaderMap)`：取 `X-Forwarded-For` 第一段，否则 `"unknown"`。登录失败节流与 `audit_events.client` 共用。

## 审计

`reveal` / `high_value_reveal` / `chat` / `export` / `restart` 在成功路径写入 `audit_events`（无 PG 则只打 tracing，不阻断操作）。列表见 [Web API](../reference/api.md)；决策见 [ADR-004](../../dev/decisions/004-readiness-and-audit.md)。

## 登录节流

按客户端键计数：`X-Forwarded-For` 第一个 IP，否则 `"unknown"`。

- 窗口 300 秒
- 同一键 ≥ 10 次失败 → `429 rate_limited`
- 成功登录清除该键记录

生产必须让反代正确传 `X-Forwarded-For`，否则所有人挤在 `unknown` 桶里。

## SSE

`EventSource` 不能带 Authorization。扫描日志流使用 `?token=`，handler 里走同一套 `verify`。不要把 JWT 写进可提交的日志或截图。

## 设置页

`GET /api/settings` 把密钥列表打码。`PUT` 时字段值含 `****` 的密钥 **不会写回** `.env`（防止把 UI 马赛克存成真密钥）。改完 Settings 后进程内 `RwLock<Settings>` 更新；部分发现源客户端在下次扫描才重建。拿不准就重启 `serve`。

## CORS

`WEB_CORS_ORIGINS`：开发可用 `*`；生产设为前端真实 origin。见 [配置](../guides/configuration.md)。

## 相关文档

- [Web API](../reference/api.md)
- [安全边界](../guides/security.md)

---
title: "配置"
type: guide
status: current
updated: 2026-08-19
summary: "解释 .env 分组与后果；变量名和默认值以 .env.example 与 Settings 为准"
---

# 配置

**真源**是 [`.env.example`](../../.env.example) 与 `aipocket_core::Settings`。本文不抄全表，只说明分组和改错的后果。

所有进程从项目根 `.env` 加载。Web 设置页保存会写回 `.env`；改完后通常要重启 `aipocket serve`。

## 必填（Web）

| 变量 | 后果 |
|------|------|
| `WEB_PASSWORD` | 单一全局登录密码；未设则服务起不来 |
| `WEB_JWT_SECRET` | JWT HMAC 密钥；用足够长的随机串 |

生产把 `WEB_CORS_ORIGINS` 从 `*` 改成真实前端 origin。

## 发现源（至少配 FOFA 或 Shodan 之一）

- `FOFA_KEYS` / `FOFA_BASE_URL`：逗号分隔多 key 轮询。代理（fofoapi）返回「账号无效」表示 Base URL 仍是 `fofa.info`；「key 不存在」表示接口对了但 key 填错。
- `FOFA_QUERY_BUDGET` 只限制**增量**模式的查询表达式条数；每条再翻 `FOFA_MAX_PAGES` 页，每页算一次 API 调用。代理日配额紧时优先加 budget、减 max pages，不要对 `body="sk-"` 这类宽查询深翻页。
- `SHODAN_KEYS`：同样轮询；分页会消耗 query credit。
- `GITHUB_HUNTER_ENABLED` + `GITHUB_TOKENS`：GitHub 制品狩猎。无 token 或无 `DATABASE_URL` 时 GitHub 路径 fail closed。

增量扫描的 query 条数由 `FOFA_QUERY_BUDGET` / `SHODAN_QUERY_BUDGET` / GitHub shard budget 限制；`mode=full` 执行 pack 内全部查询，并**关闭** Redis 跨 run 去重、强制重新验证与重新查余额（`ScanPolicy`）。日常用 incremental；只有要补历史空洞时用 full。

## 持久化

- `DATABASE_URL` 为空：仅 JSONL（`RESULTS_DIR`）。
- 设置后 PostgreSQL 是真源。Compose 会注入捆绑实例 URL。
- `DEDUP_ENABLED` + `DEDUP_REDIS_URL`：跨 run 去重 **以及** 全局 scan lease。Redis 不可达时不去重、也不拿锁，扫描仍继续；此时不要并行起第二个 `scan`/`watch`。
- 去重 TTL 与 key 格式见 [存储](../systems/storage.md)，不要改 Redis 前缀。
- `PG_DUAL_WRITE=true` 仅用于迁移核对，默认 false。

## 探测安全门（默认安全）

`.env.example` 默认 **L0 被动读**。`INTRUSIVE_CHECKS`、`PROBE_MAX_RISK`、`AUTHORIZED_PROBE_SCOPE` 必须同时满足才会跑 L1+。未授权的全网扫描不要打开这些开关。详见 [安全边界](security.md)。

## 其它分组

| 分组 | 典型变量 | 说明 |
|------|----------|------|
| 验证并发 | `VALIDATE_CONCURRENCY`、`PROBER_CONCURRENCY` | 小 VPS 先降并发，不要关数据源 |
| 调度 | `SCHEDULER_ENABLED`、`SCHEDULER_INTERVAL` | `aipocket watch` / compose profile `watch` |
| GPT | `GPT_BASE_URL`、`GPT_KEY` | 留空则跳过 LLM 提取 |
| CVE | `TAVILY_KEY` | `aipocket cve-sync` |

完整字段与注释只维护在 `.env.example`。

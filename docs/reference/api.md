---
title: "Web API"
type: reference
status: current
updated: 2026-08-19
summary: "扫描启动体、SSE、密钥类请求形状；路径以 routes.rs 为准，字段以前端 api.ts 为对照"
---

# Web API

路径真源：`crates/aipocket-api/src/routes.rs`。前端对照：`frontend/src/lib/api.ts`。第二期再接 OpenAPI。

鉴权见 [Web 鉴权](../systems/auth.md)。除 `/api/health` 与 login 外均需 JWT。

## 登录

`POST /api/auth/login` body：`{ "password" }`。

成功：`{ "token", "token_type": "bearer", "expires_in" }`（秒，来自 `WEB_TOKEN_TTL`）。

## 启动扫描

`POST /api/scan/start`

```json
{
  "source": "all",
  "sources": [],
  "mode": "incremental",
  "github_pack_ids": ["anthropic"],
  "manual_enrich": ["fofa"],
  "resume_run_id": ""
}
```

| 字段 | 默认 | 说明 |
|------|------|------|
| `source` | `"all"` | `all` / `fofa` / `shodan` / `github` / `manual` |
| `sources` | `[]` | 非空时优先于 `source`（可多选） |
| `mode` | incremental | `incremental` \| `full` |
| `github_pack_ids` | `[]` | 空 = 全部 pack；见 [packs](../../crates/aipocket-discovery/docs/packs.md) |
| `manual_enrich` | `[]` | 仅 `manual`：`fofa` 和/或 `shodan` 做主机名富化 |
| `resume_run_id` | `""` | 非空则从该 run 的 spill 续跑（需 PG） |

成功立即返回当前 `ScanStatus`（`state: running`）。若已有 running/stopping，启动失败。源如何挂载见 [扫描流水线](../systems/scan-pipeline.md)。

`GET /api/scan/status` 同形：`state`、`source`、`mode`、`run_id`、`phase`、`progress`、`error`、`log_seq`。

`POST /api/scan/stop` 取消 `CancellationToken`。

## SSE 日志

`GET /api/scan/logs/stream?since=<seq>&token=<jwt>`

浏览器 `EventSource` 不能设 Header，因此 **token 走 query**。事件：

- `event: log`
- `id:` 行号 `seq`
- `data:` 一行文本

先 replay `since` 之后的缓冲，再订 live；用 replay 高水位滤掉重复。15s keep-alive。轮询备用：`GET /api/scan/logs?since=`。

## 密钥类

列表 `GET /api/keys/{kind}`、`GET /api/high-value`：**apikey 打码**。

| 方法 | 路径 | body 要点 | 风险 |
|------|------|-----------|------|
| POST | `/api/key/reveal` | `run_id` + `kind` + (`index` 或打码 `masked`) | 返回明文 |
| POST | `/api/high-value/reveal` | 指向高价值行 | 返回明文 |
| POST | `/api/key/balance` | `apikey`（可用打码+result_id）、可选 `apiurl` | 可能打目标配额 |
| POST | `/api/keys/balance` | 批量；并发受 `BALANCE_BATCH_CONCURRENCY` 限制 | 同上 |
| POST | `/api/key/models` | 同源 | 拉 `/v1/models` 一类 |
| POST | `/api/key/chat` | **必须** `model` | **消耗目标额度** |
| POST | `/api/keys/status` | `{ "result_ids", "status" }` | 受 ValidationState 迁移表约束 |
| POST | `/api/export` | `dataset`: selected/run/high-value/all；`format`: json/csv/sub2api | 导出含明文，按 dataset |

`kind`：`valid` \| `suspicious`；UI 还有 `unavailable`。

## 其它资源

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/runs` | 按日分组的历史 |
| GET | `/api/runs/{id}/{kind}` | 单次结果 |
| GET | `/api/runs/{id}/log` | 该 run 落库日志 |
| DELETE | `/api/runs/{id}` | 级联删 results |
| GET/POST | `/api/cve`、`/sync`、`/add` | CVE |
| GET/POST/PATCH/DELETE | `/api/honeypot` | 另有 `/bulk-delete` |
| GET/POST/DELETE | `/api/manual-targets` | 另有 bulk-delete |
| GET/PUT | `/api/settings` | GET 打码密钥；PUT 忽略含 `****` 的密钥字段，写回 `.env` |
| POST | `/api/settings/check/{fofa,shodan,github}` | 连通性 |
| POST | `/api/system/restart` | 约 100ms 后 `exit(75)`。Compose 靠 `restart: unless-stopped` 拉起；本机 `start-all.sh` 用 `scripts/run-backend.sh` 循环拉起。 |

加路由：[add-endpoint.md](../guides/add-endpoint.md)。

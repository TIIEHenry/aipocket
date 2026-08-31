---
title: "验证与高价值"
type: architecture
status: current
updated: 2026-08-31
summary: "ValidationState 迁移、results.kind、high_value_keys 条件，以及 proxy_sub 机场订阅"
---

# 验证与高价值

实现：`ValidationState`（`aipocket-core`）、`Validator`（`aipocket-prober`）、`high_value_record`（`aipocket-services`）。

## 状态机

```
Candidate
  → Authenticated | Rejected | Transient | NoAuthEndpoint
Authenticated
  → FinalVerified | Suspicious | Rejected
Transient
  → Authenticated | Rejected
Suspicious
  → FinalVerified | Rejected
```

不允许的迁移被 `can_transition` 拒绝。语义：

| 状态 | 含义 |
|------|------|
| Candidate | 已提取、尚未确认鉴权 |
| Authenticated | 目标承认这把 key，尚未到终态 |
| FinalVerified | 终态有效 |
| Suspicious | 隔离：像有效但不干净（例如无鉴权端点误报相邻） |
| Rejected | 确定无效 |
| Transient | 超时/限速等，增量模式可稍后重试 |
| NoAuthEndpoint | 该 URL 看起来不需要鉴权，不当成泄露密钥 |

落库 `results.kind`：`valid`（FinalVerified）、`suspicious`、`rejected`（含 Transient 收口后的失败）。列表 API 的 `unavailable` 是 UI 对非 valid/suspicious 的归类。

Web `POST /api/keys/status` 按 `ValidationState::can_transition` 迁移，不能从 valid 直接改成任意字符串。

## 高价值入库

`high_value_keys` 按 **明文 apikey** UPSERT。`high_value_record` 返回 `None` 则不写入。必须同时满足：

1. `validation_state == final_verified`
2. 非 `suspicious`
3. `status_code == 200`
4. **不是** Google 直连 key（`is_google_direct`）
5. 下列之一：
   - key 前缀 `sk-proj-` / `sk-admin-` / `sk-svcacct-` / `sk-ant-admin`
   - 或 `sk-ant-` 且（`tier == org:admin` 或已验证模型命中一组 Claude 高阶型号）

列表接口打码；明文只走 reveal。过期探测（DeepSeek / Cursor 明确 401/403）可从高价值集移除，见 API `key_state: expired`。Cursor 官方 key 是 `crsr_`：扫描时对 `https://api.cursor.com/v1/me` 验身份，200 且有 `apiKeyName`/`userEmail` 为能用，401/403 为不能用。`crsr_` 不进高价值列表，验通后在全部密钥里。

火山方舟 key 前缀 `ark-`：方舟没有公开 `/models`，专项验证走最小化 `chat/completions` 探活（Coding Plan 基座用 `ark-code-latest` 别名），200 且有 `choices`/`usage` 为能用；余额无 API-key 可查的端点，按验证结果记 liveness。

FOFA / Shodan API key 没有独特前缀，必须靠上下文关键字提取（`FOFA_API_KEY=` / JSON `"FOFA_API_KEY":` 等），`product` 标成 `fofa`/`shodan` 并路由到官方接口。是否保留已有 URL 看 **host**（`fofoapi.com` / `fofa.info` / `shodan.io`），query/path 里出现这些域名不算。验证走 `GET /api/v1/info/my?key=`：与发现客户端同一套 error 判定（bool `true`、`"true"`/`"1"`、数字 `1` 为失败），成功还需非空 `email`/`username` 或**数值** `fcoin`（`null` 不算）。Shodan 走 `GET /api-info?key=`（plan/query_credits）。泄漏页 URL 不得当成验证端点。32 位 hex 仅在 `product` 或官方 host 为 FOFA/Shodan 时豁免 `blocked_key_format`。这两种 key 不进高价值列表。

## 机场订阅（`credential_kind = proxy_sub`）

与 AI key 共用 `Credential` / `ValidationResult`，但语义不同：

| 字段 | 含义 |
|------|------|
| `credential.apiurl` | **完整** Clash 订阅 URL（含 token 或 `/link/`、`/sub/` 路径） |
| `credential.apikey` | 提取到的 token 片段（列表与 reveal 均可能打码） |
| `credential_kind` | 固定 `proxy_sub` |

验证走 `ProtocolFamily::ClashSubscription`：`GET` 订阅 URL（Clash UA）、base64/YAML/URI 解析，**不**调用 `/v1/models`。`PROXY_SUB_VALIDATE_ENABLED=false` 时仅标记 `candidate`，不 GET。

`high_value_record` **永不**收录 `proxy_sub`。Scanner 跳过 balance、GPT recheck。验证失败仍写入 `results`（`kind=unavailable`），完整 `apiurl` 保留供 reveal。

列表 API 对 `proxy_sub` 打码 `apiurl` token、`raw_context`、`response_snippet`。明文仅经 `POST /api/key/reveal`（优先 `result_id`）或 `format=subscription-url` 导出。

## 相关文档

- [扫描流水线](scan-pipeline.md)
- [Web API](../reference/api.md)
- [术语表](../glossary.md)

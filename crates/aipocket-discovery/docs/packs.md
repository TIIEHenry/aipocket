---
title: "Provider Packs"
type: architecture
status: current
updated: 2026-08-31
summary: "按产品组织的 FOFA/Shodan/GitHub 查询包；AI PACKS 与 PROXY_PACKS 分离"
---

# Provider Packs

查询字符串的真源是 `packs.rs` 的 `PACKS`（AI）与 `proxy_packs.rs` 的 `PROXY_PACKS`（机场），以及 `legacy_queries.rs`。本文只说明怎么组合，不抄查询原文。

## Proxy pack（`PROXY_PACKS`）

与 AI `PACKS` **分离**。仅当 `PROXY_SUB_ENABLED=true`（默认 false）时由 `assemble_sources` 追加查询与 GitHub terms。

| Pack id | 面板 / 场景 |
|---------|-------------|
| `proxy_v2board` | V2Board `/api/v1/client/subscribe?token=` |
| `proxy_sspanel` | SSPanel `/link/{token}`、`SUBSCRIBE_URL` |
| `proxy_marzban` | Marzban `/sub/{token}` |
| `proxy_3xui` | 3x-ui / x-ui 订阅路径 |
| `proxy_clash_env` | `CLASH_SUB_URL` / `SUBSCRIBE_URL` 环境变量泄露 |
| `proxy_subconverter` | subconverter 相关页面与配置 |

**实验性 pack**（`PROXY_SUB_EXTRA_PACKS`，设置页 / 扫描页「待取证 pack」勾选）：`proxy_nezha`、`proxy_wings`、`proxy_mqpanel`、`proxy_node_uri`。与基础 `PROXY_PACKS` 合并选用，未知 id 忽略。

`PROXY_SUB_FOFA_BODY=true` 且 `PROXY_SUB_ENABLED` 时，FOFA 请求 `fields` 追加 `body`（配额更高；列表仍走 `mask_record`）。

`github_pack_ids` 空 / `all` **仅选 AI pack**；proxy 不由 GitHub pack 多选控制，而由 `PROXY_SUB_ENABLED` 开关。

导出格式 `sub2api` 与发现产品 id `sub2api_panel` 不是同一个名字；CPA 对应 `cliproxyapi`。中转面板查询在 `legacy_queries` 的 `PRODUCT_QUERIES`。Cursor（`api.cursor.com` + `crsr_`）与火山方舟（`ark.*.volces.com` + `ark-`）用 `domain_key_pack!`；增量 FOFA 的 DIRECT 列表也包含这两类表达式，不勾 pack 也会搜。`fofa_leak` / `shodan_leak` 搜环境变量名（`FOFA_API_KEY` 等），`query_priority` 为 850，保证默认增量预算能排到。

## 形状

```text
ProviderPack { id, fofa_queries[], shodan_queries[], github_terms[] }
```

当前 pack id（与前端 `GitHubPackId` 对齐，另加 UI 的 `all`）：

`openai` `anthropic` `gemini` `xai` `qoder` `kiro` `aws_bedrock` `cursor` `windsurf` `azure_openai` `cohere` `deepseek` `fireworks` `glm` `kimi` `longcat` `minimax` `qwen` `replicate` `together` `volcengine_ark` `fofa_leak` `shodan_leak`

CLI `aipocket queries` 打印每个 pack 在三源上的条数。

## 如何被选用

1. Web `github_pack_ids` 为空或含 `all` → 使用全部 pack。
2. 否则只取列出的 id。
3. FOFA 查询 = `legacy_queries::fofa_queries()` ∪ 所选 pack 的 `fofa_queries`（去重）。
4. Shodan = 所选 pack ∪ `legacy_queries::shodan_product_queries()`。
5. GitHub terms 只来自所选 pack；无 token 或无 PG 时根本不挂 `GithubSource`。

增量模式再按 `FOFA_QUERY_BUDGET` / `SHODAN_QUERY_BUDGET` / GitHub commit·code budget 截断。`QUERY_EXPLORATION_RATIO` 留给低收益查询做探索，避免永远只跑头部表达式。

## 加一个 pack

1. 在 `PACKS` 增加唯一 `id`，三源都给至少一条（测试 `pack_ids_are_unique_and_new_families_cover_every_source` 对部分新家族强制三源非空）。
2. 如需出现在 GitHub 狩猎 UI，把 id 加进 `frontend/src/lib/api.ts` 的 `GitHubPackId`。
3. 不要把密钥样例写进文档；查询里用公开的变量名/域名即可。

## 相关文档

- [扫描流水线](../../../docs/systems/scan-pipeline.md)
- [`crates/aipocket-discovery/README.md`](../README.md)

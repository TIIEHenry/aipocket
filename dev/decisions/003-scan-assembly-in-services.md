---
title: "扫描装配落在 aipocket-services，api 与 CLI 共用"
type: decision
status: accepted
updated: 2026-08-29
created: 2026-08-29
summary: "发现源拼装移入 services 的 assemble_sources；被跳过的源经 SkippedSource 结构化回传"
---

# ADR-003：扫描装配落在 services，api/CLI 共用

## 背景

一次 scan 的「发现源拼装」目前有两份实现：

- Web：`crates/aipocket-api/src/routes.rs` 的 `scan_start`（选 pack、组 query、建 `FofaSource` / `ShodanSource` / `GithubSource` / `ManualSource` / `ManualEnrichSource`）。
- CLI：`crates/aipocket/src/main.rs` 的 `run_scan`（再拼一遍）。

两份已经漂移：CLI 不支持 `manual_enrich`、不做 pack 过滤（永远全量 registry）、shodan query 组合与 Web 不一致。这违反 [AGENTS.md](../../AGENTS.md) 的「业务编排放 services，不放 handler」与「单一真源」。

同时，GitHub 源在缺 token 或未启用 PostgreSQL 时按 [安全边界](../../docs/guides/security.md) fail closed **静默不挂**，请求仍返回 `running`，用户以为在扫 GitHub。跳过原因既没进日志也没进 `ScanStatus`。

## 决策

1. 新增 `aipocket_services::assemble_sources`（同步纯函数），作为发现源拼装的**唯一真源**。api 的 `scan_start` 与 CLI 的 `run_scan` 都改为调用它。
2. 装配结果用 `ScanPlan { sources, skipped }` 返回；`skipped` 是 `Vec<SkippedSource>`。
3. `SkippedSource { source, reason }` 定义在 `aipocket-core`（core 不依赖 services），供 core / services / api / 前端共用。
4. `ScanStatus` 增加 `skipped_sources: Vec<SkippedSource>`（可加不可减、`#[serde(default)]`），Web 在扫描台展示；CLI 用 `tracing::warn` 逐条打印。
5. query / pack 组合下沉到 `aipocket-discovery::compose_queries`，被 services 装配器消费。api 不再直接 `new` 各发现源。

**跳过判定（fail closed，结构化可见）**：

| 源 | 挂上条件 | 否则 skipped.reason |
|----|----------|---------------------|
| fofa | 请求含 fofa/all 且 `FOFA_KEYS` 非空 | `missing FOFA_KEYS` |
| shodan | 请求含 shodan/all 且 `SHODAN_KEYS` 非空 | `missing SHODAN_KEYS` |
| github | 请求含 github/all 且 `GITHUB_TOKENS` 非空 且 `DATABASE_URL` 已启用 | `missing GITHUB_TOKENS` / `requires DATABASE_URL` |
| manual | 请求含 manual 且 `manual_targets` 非空 | `no manual targets` |
| manual_enrich | 请求含 manual 且 `manual_enrich` 非空 | 不产生 skipped（可选增强） |

对 fofa/shodan：过去即使无 key 也会挂上、逐 query 失败刷进日志。改为无 key 即 skipped 是**行为收敛**——把「只会报错的源」显式标为跳过，语义更清晰，且不改变「有 key 时」的任何行为。

## 影响

- 依赖方向不变：`api → services → discovery / clients / core`。api 不再承担业务拼装。
- `ScanStatus` 是 additive schema 变更，旧客户端忽略新字段即可。
- CLI 与 Web 的发现范围从此一致，修一处即两处生效。
- 装配器为同步纯函数（`manual_targets` 由调用方预取传入），可无网络/无 DB 单测。

## 备选

- **在 api 内提取私有函数**：仍无法给 CLI 复用，漂移会复发。否决。
- **只记日志、不进 ScanStatus**：前端无法稳定展示跳过原因，需解析日志文本。否决（用户已选结构化字段）。

## 关联

- 实施方案：[dev/plans/scan-assembly-and-routes-split.md](../plans/scan-assembly-and-routes-split.md)
- [扫描流水线](../../docs/systems/scan-pipeline.md) · [安全边界](../../docs/guides/security.md) · [Web API](../../docs/reference/api.md)

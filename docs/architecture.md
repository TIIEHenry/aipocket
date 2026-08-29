---
title: "架构概览"
type: architecture
status: current
updated: 2026-08-29
summary: "AIPocket 系统全景、crate 依赖方向、扫描流水线与数据职责"
---

# 架构概览

> 详细协作见 [扫描流水线](systems/scan-pipeline.md)、[存储](systems/storage.md)。依赖方向与 Do Not 以 [AGENTS.md](../AGENTS.md) 为准。

## 系统全景

AIPocket 扫描已授权范围内的 AI 基础设施暴露面，从 FOFA / Shodan / GitHub 提取凭证候选，验证后查余额，并把结果持久化到 PostgreSQL。

```
浏览器  →  frontend (Nginx / Vite)
              ↓ /api
         aipocket serve (Axum)
              ↓
    PostgreSQL 16（真源）  +  Redis 7（跨 run 去重 / scan lease）
              ↓
         FOFA / Shodan / GitHub / 目标站点（出站 HTTP）
```

两个可部署件：`aipocket` 二进制（CLI + HTTP API）与 `frontend/`。Compose 另附 PostgreSQL 与 Redis。

## Crate 边界

依赖方向保持 **api → services → discovery / prober / clients / db / core**。禁止反向依赖。

```
aipocket (bin: clap + serve 装配)
    ├── aipocket-api          Axum 路由、JWT、SSE、ScanManager
    ├── aipocket-services     Scanner / Analyzer / Balance / Scheduler
    ├── aipocket-discovery    DiscoverySource、query packs、GitHub artifacts
    ├── aipocket-prober       Prober trait、风险门控、Validator
    ├── aipocket-clients      FOFA / Shodan / GitHub / Tavily HTTP
    ├── aipocket-db           Repository、schema、Redis dedup/lease
    └── aipocket-core         Settings、领域模型、URL 规范化
```

| crate | 做什么 | 不做什么 |
|-------|--------|----------|
| core | 配置、模型、规范化 | HTTP、SQL |
| db | 持久化、去重、lease | 业务编排 |
| clients | 上游 API 客户端 | 扫描策略 |
| discovery | 拉 hits / 制品、query packs | 验证密钥 |
| prober | 对目标做风险门控探测 | 发现源查询 |
| services | 编排整次 scan | 直接暴露 HTTP |
| api | 路由与 DTO | 业务逻辑（放 services） |
| aipocket | 装配 CLI/进程 | 新业务代码 |

新 HTTP 路由放 `aipocket-api`；编排放 `aipocket-services`。前端所有后端调用走 `frontend/src/lib/api.ts`。

## 扫描流水线（摘要）

一次 run 的阶段（`runs.phase`，见 `PipelinePhase` / [ADR-002](../dev/decisions/002-pipeline-phase-spill.md)）：

`started → discovery → extract → probe → gpt → validate → finalize → finished`

phase 是**已完成 spill 的游标**，不是「正在执行」。有 PG 时 Scanner 不把全部 discovery hits 留在内存里。

1. **discovery**：FOFA / Shodan / GitHub / manual targets 拉 hits，必要时 spill 到 PostgreSQL。
2. **extract**：正则 + 制品观察提取 `Credential`。
3. **probe**：产品 Prober，默认仅 L0 被动读；L1+ fail closed。
4. **gpt**：可选 LLM 增强提取（无 `GPT_KEY` 则跳过）。
5. **validate**：对候选密钥做提供商验证。
6. **finalize**：余额、高价值入库、蜜罐标记、写 run 日志。

增量模式走 query budget 与 Redis 去重；`mode=full` 忽略 watermark、关闭跨 run 去重并强制重验。GitHub 在缺 token 或未启用 PostgreSQL 时**不会挂上** `GithubSource`。细节见 [扫描流水线](systems/scan-pipeline.md)。

## 数据职责

| 存储 | 职责 |
|------|------|
| PostgreSQL | 持久化真源：runs / results / high_value_keys / CVE / spill 表 |
| Redis | 跨 run 去重缓存 + 全局 scan lease。不可达时不去重、不拿锁 |
| JSONL / `RESULTS_DIR` | 无 `DATABASE_URL` 时的原始行为；`PG_DUAL_WRITE=true` 时双写 |

Schema 必须幂等、可加不可减。禁止为迁移 drop/rebuild 生产表或改 Redis key 格式。见 [存储](systems/storage.md)。

## 运行时约束

- 全部 HTTP 出站经共享 `reqwest::Client`，禁止同步 HTTP。
- 配置名与默认值必须与 `.env.example` 兼容；新增环境变量要有向后兼容默认值。
- 探测默认 `INTRUSIVE_CHECKS=false`、`PROBE_MAX_RISK=0`。主动探测必须同时满足授权范围 allowlist。见 [安全边界](guides/security.md)。

## 相关文档

- [扫描流水线](systems/scan-pipeline.md)
- [验证与高价值](systems/validation.md)
- [存储](systems/storage.md)
- [Web 鉴权](systems/auth.md)
- [ADR](../dev/decisions/INDEX.md)

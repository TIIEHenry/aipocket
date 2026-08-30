---
title: "存储"
type: architecture
status: current
updated: 2026-08-30
summary: "PostgreSQL 真源、spill 表职责、Redis key 格式（稳定契约）与 JSONL 双写"
---

# 存储

表结构真源：[`migrations/schema.sql`](../../migrations/schema.sql)（启动时 `ensure_schema` 幂等执行）。**Redis key 格式是稳定契约**：改前缀或 hash 算法会让去重/lease 与旧部署错位。禁止 flush 生产 Redis「清理环境」。

## 分层

| 层 | 职责 | 丢失的后果 |
|----|------|------------|
| PostgreSQL | runs、结果、高价值、CVE、spill、蜜罐、手工目标 | 丢扫描产物 |
| Redis | 跨 run 去重 TTL；scan lease | 重复劳动；lease 残留会挡住下一轮 scan |
| `RESULTS_DIR` JSONL | 无 PG 时的原始路径；`PG_DUAL_WRITE` 迁移双写 | 视配置 |

`DATABASE_URL` 为空则 `connect_pg` 返回 `None`。Compose 注入捆绑实例 URL，结果目录挂 `/data/aipocket`。

## Redis key（禁止改格式）

前缀与 hash 见 `aipocket-db` 的 `dedup.rs` / `scan_lock.rs`。主机与凭证身份均 **SHA-1 hex**。凭证 hash 材料：`apikey|apiurl`（apiurl 空则用 host）。

| Key | TTL 设置 | 含义 |
|-----|----------|------|
| `aipocket:scan:lock` | `SCAN_LOCK_TTL`（秒，SET NX EX） | 全局扫描租约；值是 UUID token；TTL/3 用 Lua 续约 |
| `aipocket:dedup:host:{sha1}` | `DEDUP_HOST_TTL` | 主机已处理 |
| `aipocket:dedup:target:{stage}:{sha1}` | `DEDUP_HOST_TTL` | 某阶段目标身份已处理 |
| `aipocket:dedup:cred:ok:{sha1}` | `DEDUP_CRED_TTL` | 成功验证缓存（JSON） |
| `aipocket:dedup:cred:outcome:{sha1}` | rejected→`DEDUP_REJECTED_TTL`；其它→`DEDUP_TRANSIENT_TTL` | 结果分类 |
| `aipocket:dedup:cred:bal:{sha1}` | `DEDUP_BALANCE_TTL` | 余额缓存 |

`DEDUP_ENABLED=false` 或 Redis 连不上：去重全关，扫描继续；lease 也拿不到（单进程自行避免并行 scan）。

## PostgreSQL 表（职责）

| 表 | 职责 |
|----|------|
| `runs` | 一次扫描元数据、`phase` / `phase_detail`、完整 `log` |
| `results` | 最终 valid/suspicious/rejected；`record JSONB` 为完整 ValidationResult；明文 apikey 仅供 reveal/export |
| `high_value_keys` | 跨 run 按 apikey UPSERT |
| `cves` / `advisories` | CVE / 公告 |
| `scan_discovery_hits` | 发现 hits spill（banner/header 完整） |
| `scan_candidates` | 提取候选 spill |
| `scan_validation_results` | 中途验证结果，供 validate resume |
| `scan_probe_events` | probe 遥测，避免证据 blob 常驻内存 |
| `request_ledger` | 物理 HTTP 尝试账本（`PLANNER_METRICS_VERSION=3` 分母） |
| `query_metrics` | 每条查询的 hits/验证漏斗 |
| `source_checkpoints` | GitHub shard watermark + cursor |
| `github_artifacts` | 制品工作队列（**无明文密钥**） |
| `honeypot_sites` | `host_key` = hostname:port，扫描开始加载 |
| `manual_targets` | 自定义狩猎 origin（规范化后的 scheme://host[:port]） |
| `audit_events` | 敏感操作审计（`action` / `client` / `detail` JSONB；无明文密钥） |

Schema 只允许 `IF NOT EXISTS` / `ADD COLUMN IF NOT EXISTS`。禁止 drop 生产表、`down -v`。

## 相关文档

- [架构](../architecture.md)
- [扫描流水线](scan-pipeline.md)
- [日常运维](../guides/operations.md)
- [`crates/aipocket-db/README.md`](../../crates/aipocket-db/README.md)

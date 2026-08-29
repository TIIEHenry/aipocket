---
title: "aipocket-db"
type: concept
status: current
updated: 2026-08-18
summary: "SQLx Repository、幂等 schema、Redis 去重与 scan lease"
---

# aipocket-db

## 职责

- `ensure_schema`：执行幂等 `migrations/schema.sql`
- `Repository`：runs / results / spill / high-value / CVE
- Redis `DedupStore` 与 `ScanLease`

## 依赖

| crate | 用途 |
|-------|------|
| aipocket-core | Settings、领域类型 |

## 不做什么

- 不编排扫描
- 不调用 FOFA/Shodan

## 相关文档

- [存储](../../docs/systems/storage.md)（含 Redis key 契约）

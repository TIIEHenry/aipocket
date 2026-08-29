---
title: "aipocket-discovery"
type: concept
status: current
updated: 2026-08-18
summary: "DiscoverySource、provider packs、GitHub artifact 观察"
---

# aipocket-discovery

## 职责

- `DiscoverySource`：FOFA / Shodan / GitHub 等源的 fetch 与 budget
- Query packs（按产品组织查询）
- GitHub artifact 工作队列与 checkpoint 更新（无明文密钥）

## 依赖

| crate | 用途 |
|-------|------|
| aipocket-clients | 上游 HTTP |
| aipocket-db | checkpoint / artifact 持久化 |
| aipocket-core | ScanMode、Credential |

## 不做什么

- 不跑 Validator / Prober
- 不写最终 `results` 行

## 相关文档

- [扫描流水线](../../docs/systems/scan-pipeline.md)
- [Provider Packs](docs/packs.md)

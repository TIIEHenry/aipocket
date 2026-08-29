---
title: "术语表"
type: concept
status: current
updated: 2026-08-18
summary: "AIPocket 常用术语；概念只在此定义一次"
---

# 术语表

| 术语 | 含义 |
|------|------|
| **run** | 一次扫描，主键 `run_YYYY_MM_DD_HH-MM-SS` |
| **hit** | 发现源返回的主机/页面记录，尚未提取凭证 |
| **credential** | `apikey` + `apiurl`（及 host）对 |
| **spill** | 扫描中把中间结果写入 PostgreSQL，避免进程内堆积 |
| **high-value** | 跨 run 按 apikey UPSERT 的高价值密钥；条件见 [验证与高价值](systems/validation.md) |
| **L0–L3** | Prober 风险：L0 被动读；L1 弱口令/IDOR；L2 SSRF/SQLi；L3 RCE |
| **fail closed** | 缺配置或未授权时不执行危险路径（GitHub 无 token/PG 则不挂源；L1+ 无 allowlist 不发包） |
| **pack** | 按产品组织的 FOFA/Shodan/GitHub 查询包 |
| **ScanLease** | Redis `aipocket:scan:lock`，全局同时只允许一个 scan |
| **ProbeSpec** | 带 class / risk / depends_on / max_requests 的一条探测规格 |
| **ValidationState** | 凭证状态机（Candidate → … → FinalVerified 等） |
| **reveal** | 按条返回明文 apikey；列表接口只打码 |
| **incremental / full** | incremental：budget + watermark + Redis 去重；full：全查询、强制重验、不去重 |
| **origin allowlist** | `AUTHORIZED_PROBE_SCOPE`，精确 `scheme://host[:port]`，无 path |
| **legacy queries** | pack 之外的历史 FOFA/Shodan 产品查询，full scan 仍会合并进去 |

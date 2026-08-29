---
title: "添加平台 Prober"
type: guide
status: current
updated: 2026-08-18
summary: "实际要改的文件、RiskPolicy 不变量、以及规格引擎 vs 被动 Prober"
---

# 添加平台 Prober

内部设计：[风险门控](../../crates/aipocket-prober/docs/risk-gating.md)。不要在 PR 描述或 docs 里粘贴 L2/L3 payload。

## 先选哪条路径

| 目标 | 改哪里 |
|------|--------|
| 给已有产品加 L0 路径 / L1+ 规格 | `migrated_specs.rs` 或 `product_specs.rs` 的 `SURFACES` |
| 新产品被动指纹（几条 GET） | `products.rs` 的 `passive_prober!` + `default_probers()` |
| 新协议的**已有密钥**验证 | `provider.rs` 的 `SPECS` + 必要时 `specialized.rs` |

扫描主机暴露面走规格引擎；验证用户已有的 apikey 走 Validator / ProviderRegistry。两者都叫 “probe”，不要混文件。

## 规格引擎步骤

1. 为产品增加 `ProbeSpec`（`id` 稳定，如 `dify.unauth`）：`vuln_class`、`risk_level`、`depends_on`、`max_requests`、`entry` JSON。
2. 确认 L1+ 在 `INTRUSIVE_CHECKS=false` 或空 `AUTHORIZED_PROBE_SCOPE` 时变成 `SkippedGate`，而不是发出请求。
3. 依赖：例如 IDOR spec `depends_on` 弱口令 spec；依赖失败应为 `SkippedDependency`。
4. 行为测试：门控跳过、预算耗尽 `SkippedBudget`、未知 `PROBE_VULN_CLASSES` 启动失败。
5. 若产品名出现在 UI / CVE 映射，更新 [扫描流水线](../systems/scan-pipeline.md) 或 CVE 说明（1 篇）。

## 被动 Prober 步骤

1. `products.rs`：`passive_prober!(Name, "product_id", &["/path", ...])`。
2. 加入 `default_probers()`。
3. 只做成功 GET 的 snippet，风险固定 0 / `unauth_read`。

## 不变量

- 出站 HTTP 只用传入的共享 `reqwest::Client`。
- L1+ 必须能被设置完全关掉；缺 allowlist 不得打主动包。
- 未知 vuln class、非法 `PROBE_MAX_RISK`：进程启动失败。

## 相关文档

- [`crates/aipocket-prober/README.md`](../../crates/aipocket-prober/README.md)
- [安全边界](security.md)

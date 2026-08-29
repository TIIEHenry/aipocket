---
title: "Prober 风险门控"
type: architecture
status: current
updated: 2026-08-18
summary: "L0–L3、RiskPolicy、ProbeSpec 计划与 execute_plan；L1+ 必须能被设置关掉"
---

# Prober 风险门控

本 crate 内部设计。对外贡献步骤见 [加 Prober](../../../docs/guides/add-prober.md)。**不要**在文档里复制 L1+ 的攻击 payload；规格在代码里。

## 两套入口

| 层 | 文件 | 做什么 |
|----|------|--------|
| 被动产品探测 | `products.rs` | `Prober` trait：对已知路径做 L0 GET |
| 规格引擎 | `product_specs.rs` / `migrated_specs.rs` + `engines.rs` | `ProbeSpec` 图：class、risk、depends_on、max_requests |
| 凭证验证 | `validator.rs` + `provider.rs` + `specialized.rs` | 已有 key 的提供商协议探测，不是扫主机 |

扫描主路径走规格引擎 `execute_plan` / `plan_specs`。`default_probers()` 是产品指纹/被动读的补充。

## RiskPolicy

`RiskPolicy::from_settings`：

- `PROBE_VULN_CLASSES`：`*` / `all` / 空 = 全部 class；否则逗号列表。**未知名字启动失败**。
- `PROBE_MAX_RISK`：0..=3，否则启动失败。
- `intrusive_checks` 实际值 = `INTRUSIVE_CHECKS && AUTHORIZED_PROBE_SCOPE 非空`。空 allowlist 时 L1+ 不会跑。
- L2/L3 还要对应 `PROBE_SSRF_ENABLED` / `PROBE_SQLI_ENABLED` / `PROBE_RCE_ENABLED`。

`RiskPolicy::allows(spec, target)` 对 L0 只检查 class 与 max_risk；对更高风险再查 intrusive + origin allowlist（精确 origin，无 path/query）。

## 计划与跳过原因

`plan_specs` 过滤后 `execute_plan` 按 `depends_on` 调度。未入计划的 spec 仍记 `NodeOutcome`：

| NodeStatus | 含义 |
|------------|------|
| Executed | 发出了请求 |
| SkippedGate | 风险门 / 授权范围 |
| SkippedClass | class 未启用 |
| SkippedBudget | `MAX_REQUESTS_PER_TARGET` 用尽 |
| SkippedNoAuth | 需要鉴权但还没有 token |
| SkippedDependency | 依赖失败或被跳过 |
| Failed | 执行出错 |

预算：产品 spec 自己的 `max_requests` 与全局 `MAX_REQUESTS_PER_TARGET`；generic 探测另用 `GENERIC_MAX_REQUESTS_PER_TARGET`。

## 相关代码

- `capability.rs`：`VulnClass`、`ProbeSpec`、`RiskPolicy`
- `engine.rs`：`Prober`、`ProbeContext::allows`（trait 路径的同类门）
- `engines.rs`：规格执行

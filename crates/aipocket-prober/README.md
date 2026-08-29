---
title: "aipocket-prober"
type: concept
status: current
updated: 2026-08-18
summary: "产品 Prober、L0–L3 风险门控、凭证 Validator"
---

# aipocket-prober

## 职责

- `Prober`：按产品探测暴露面，产出 `ProbeFinding` / 候选凭证
- `ProbeContext::allows`：L1+ fail closed
- `Validator`：对已有 apikey/url 做提供商验证

## 依赖

| crate | 用途 |
|-------|------|
| aipocket-core | Credential、Settings |

## 不做什么

- 不查 FOFA/Shodan
- 不写 PostgreSQL

## 相关文档

- [风险门控](docs/risk-gating.md)
- [加 Prober](../../docs/guides/add-prober.md)
- [安全边界](../../docs/guides/security.md)

---
title: "文档总览"
type: concept
status: accepted
updated: 2026-08-18
summary: "按读者说明如何阅读 AIPocket 文档；完整链接见 INDEX.md"
---

# 系统文档

> 完整导航：[INDEX.md](INDEX.md) · 维护规则：[DOCUMENTATION.md](DOCUMENTATION.md)

## 怎么读

| 角色 | 推荐路径 |
|------|----------|
| 要跑起来 | [快速开始](guides/getting-started.md) → [配置](guides/configuration.md) |
| 运维 / 部署 | [部署](guides/deploy.md) → [日常运维](guides/operations.md) → [安全边界](guides/security.md) |
| 用 Web UI | [界面](guides/web-ui.md) · [截图](screenshots.md) |
| 改扫描 / 探测 | [架构](architecture.md) → [扫描流水线](systems/scan-pipeline.md) → [风险门控](../crates/aipocket-prober/docs/risk-gating.md) → [加 Prober](guides/add-prober.md) |
| 加 API / 页面 | [HTTP API](reference/api.md) → [加端点](guides/add-endpoint.md) → [前端约定](guides/frontend.md) |
| Agent | 根目录 [AGENTS.md](../AGENTS.md) → [DOCUMENTATION.md](DOCUMENTATION.md) → [status](../dev/progress/status.md) |

## 结构

```
docs/architecture.md   全局架构
docs/systems/          跨 crate 协作
docs/guides/           指南
docs/reference/        CLI / HTTP 场景
crates/*/README.md     单 crate 介绍
dev/                   方案、ADR、进度
```

结构变更后更新 [INDEX.md](INDEX.md)，并运行 `python3 scripts/check-docs-health.py`。

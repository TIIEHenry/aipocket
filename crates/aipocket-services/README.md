---
title: "aipocket-services"
type: concept
status: current
updated: 2026-08-29
summary: "Scanner、Analyzer、Balance、Scheduler：扫描编排，不暴露 HTTP"
---

# aipocket-services

## 职责

- `Scanner`：整次 run 的阶段推进（`PipelinePhase`）、resume、ScanEvent
- 正则提取与结果 finalize
- `BalanceService`、可选 GPT `Analyzer`、`Scheduler`

## 依赖

discovery、prober、clients、db、core。

## 不做什么

- 不注册 Axum 路由
- 不直接成为用户入口（由 CLI / api 调用）

## 相关文档

- [扫描流水线](../../docs/systems/scan-pipeline.md)
- [验证与高价值](../../docs/systems/validation.md)
- [ADR-002](../../dev/decisions/002-pipeline-phase-spill.md)

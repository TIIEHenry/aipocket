---
title: "Development Progress"
type: progress
status: current
updated: 2026-08-30
summary: "FOFA/Shodan 泄露扫描闭环：上下文提取、官方验证、三源 pack"
---

# Current session

- 2026-08-30：Phase 2 完成 — `routes.rs` 已拆为 `routes/` 域模块；CI 覆盖率不再豁免该路径。

# Completed

- 2026-08-30：`aipocket-api` routes 按域拆到 `routes/`；契约测试覆盖 handler；CI llvm-cov 去掉 `routes.rs` 豁免。
- 2026-08-30：ADR-003 — `assemble_sources` 为 HTTP/CLI 单一装配器；装配 fail-closed 的源经 `ScanStatus.skipped_sources` 可见。
- 2026-08-29：Pipeline phase 真源 + 流式 spill（ADR-002）。
- 2026-08-29：FOFA/Shodan 泄露扫描闭环（提取、发现 pack、官方验证与余额）。
- 2026-08-26：火山方舟 `volcengine_ark` 专项发现与验证。
- 2026-08-20：Cursor `crsr_` 官方可用性判定（发现、路由、401/403 过期）。
- 2026-08-18：文档系统骨架（规范、INDEX、指南、crate README、健康检查、ADR-001）。
- 2026-08-18：细化 `systems/scan-pipeline.md`、`storage.md`、`validation.md`、`auth.md`；`reference/api.md`；`crates/aipocket-prober/docs/risk-gating.md`；`crates/aipocket-discovery/docs/packs.md`。

# Open

- 本机 `docs/local-deploy.md` / 口令文件不入库。
- OpenAPI 与从 `Settings` 生成配置表仍留后续。

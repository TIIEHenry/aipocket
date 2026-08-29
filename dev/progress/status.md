---
title: "Development Progress"
type: progress
status: current
updated: 2026-08-29
summary: "FOFA/Shodan 泄露扫描闭环：上下文提取、官方验证、三源 pack"
---

# Current session

- 2026-08-29：ADR-002 — `runs.phase` 为已完成 spill 游标；PG 下 discovery hits 按页读写，resume 不再把未提取标成 `validate`。
- FOFA/Shodan API key 泄露闭环：上下文关键字提取（含 JSON）、GitHub 泄漏页强制官方 URL、`product` 归因验证、`info/my` / `api-info` 查配额；`fofa_leak`/`shodan_leak` pack 进入增量预算。验证路由按 host 而非 URL 子串；FOFA 成功判定与客户端 error 旗标对齐，且要求数值 fcoin 或非空账号字段。

# Completed

- 2026-08-29：Pipeline phase 真源 + 流式 spill（ADR-002）。
- 2026-08-29：FOFA/Shodan 泄露扫描闭环（提取、发现 pack、官方验证与余额）。
- 2026-08-26：火山方舟 `volcengine_ark` 专项发现与验证。
- 2026-08-20：Cursor `crsr_` 官方可用性判定（发现、路由、401/403 过期）。
- 2026-08-18：文档系统骨架（规范、INDEX、指南、crate README、健康检查、ADR-001）。
- 2026-08-18：细化 `systems/scan-pipeline.md`、`storage.md`、`validation.md`、`auth.md`；`reference/api.md`；`crates/aipocket-prober/docs/risk-gating.md`；`crates/aipocket-discovery/docs/packs.md`。

# Open

- 本机 `docs/local-deploy.md` / 口令文件不入库。
- OpenAPI 与从 `Settings` 生成配置表仍留后续。

---
title: "Development Progress"
type: progress
status: current
updated: 2026-08-29
summary: "runs.phase 为已完成 spill 游标；PG 下 discovery hits 按页读写"
---

# Current session

- 2026-08-29：ADR-002 — `runs.phase` 为已完成 spill 游标；PG 下 discovery hits 按页读写，resume 不再把未提取标成 `validate`。

# Completed

- 2026-08-29：Pipeline phase 真源 + 流式 spill（ADR-002）。

# Open

- 本机 `docs/local-deploy.md` / 口令文件不入库。
- OpenAPI 与从 `Settings` 生成配置表仍留后续。

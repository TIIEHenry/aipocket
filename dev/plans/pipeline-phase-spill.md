---
title: "Pipeline phase 真源 + 流式 spill 实施方案"
type: plan
status: implemented
updated: 2026-08-29
created: 2026-08-29
summary: "PipelinePhase、分页 spill、Scanner 按完成态推进 phase"
---

# Pipeline phase + 流式 spill

> 决策：[ADR-002](../decisions/002-pipeline-phase-spill.md)

**Goal:** `runs.phase` 只在对应 spill 完成后推进；有 PG 时 Scanner 不保留全部 discovery hits。

**Architecture:** 阶段枚举放 `aipocket-core`。db 增加 hit 分页与计数。Scanner 按 Discovery → Extract → Probe → Gpt → Validate → Finalize 推进，每步先 spill 再 `update_phase`。

## Files

- Create: `crates/aipocket-core/src/pipeline_phase.rs`
- Modify: `crates/aipocket-core/src/lib.rs`
- Modify: `crates/aipocket-db/src/pipeline.rs`（`load_discovery_hit_page`、count）
- Modify: `crates/aipocket-db/tests/postgres_compat.rs`
- Modify: `crates/aipocket-services/src/scanner.rs`
- Modify: `crates/aipocket-api/src/scan_manager.rs`
- Modify: `docs/systems/scan-pipeline.md`、`docs/architecture.md`、`dev/progress/status.md`、INDEX

## Tasks

1. `PipelinePhase`：`as_str` / `parse` / `rank` / `reconcile(stored, hits, candidates)`
2. db：`load_discovery_hit_page(after_id, limit)`、`count_discovery_hits`、`count_candidates`；`load_discovery_hits` 保留给测试
3. Scanner：有 PG 则源返回后立刻 upsert 并丢 hits；extract/probe/gpt 分页；validate 分页 candidates；事件名与枚举一致
4. `phase_label` 覆盖新阶段名；测试从 `extract_validate` 改为 `validate`
5. 知识层：`scan-pipeline.md` 写明 phase=已完成 spill

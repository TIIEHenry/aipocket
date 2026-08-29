---
title: "runs.phase 表示已完成的可恢复阶段；PG 下 hits 按页 spill"
type: decision
status: accepted
updated: 2026-08-29
summary: "持久化 phase 是 spill 完成游标，不是 UI 粗阶段；有 PostgreSQL 时 Scanner 不保留 O(total hits)"
created: 2026-08-29
---

# 002. Pipeline phase 真源 + 流式 spill

## Context

文档约定 `runs.phase` 顺序为：

`started → discovery → extract → probe → gpt → validate → finalize → finished`

实现里 discovery spill 之后立刻写成 `validate`，提取 / GPT 尚未落 `scan_candidates`。`phase_rank >= validate` 的 resume 会跳过提取，候选为空时验证空转。

同时文档写「按页 spill、进程不保留 O(total_hits)」，Scanner 却把全部 banner 堆在 `Vec` 里，结束后才 `upsert_discovery_hits`。

UI 另用 `extract_validate` / `balance_finalize` 作为 `ScanEvent::Phase`，与持久化 phase 混用同一字符串通道。

## Decision

1. **`runs.phase` = 最后一个已完成且 spill 已落库的阶段。** 崩溃时 phase 停在上一完成态；该阶段的 UPSERT 必须幂等，以便重跑当前阶段。
2. **阶段名与 `PipelinePhase`（`aipocket-core`）为单一真源。** Scanner、resume、`ScanEvent::Phase`、`scan_manager` 标签都用这组名字。禁止再把 UI 粗阶段写入 `runs.phase`。
3. **进入下一阶段的条件：**

   | 写入 phase | 必须已经发生 |
   |------------|----------------|
   | `discovery` | `scan_discovery_hits` 已写入本 run 本轮发现结果（无 PG 则仅内存完成） |
   | `extract` | 正则候选已 `upsert_candidates`（或无 PG 时已并入内存凭证） |
   | `probe` | Prober 候选已 upsert |
   | `gpt` | GPT 提取已跑完（无 `GPT_KEY` 视为空成功） |
   | `validate` | `scan_validation_results` 已写入（或无 PG 时验证循环结束） |
   | `finalize` | 余额 + `results` / high_value 已持久化 |
   | `finished` | run 收口指标写完 |

4. **Resume 跳过规则：** `completed >= 某阶段` 则不再执行该阶段。无 PostgreSQL 时不支持跨进程 resume（保持现契约）。
5. **撒谎修复：** 仅当持久化 phase ∈ {`validate`,`finalize`}（旧实现会在提取前写入 `validate`），且 `scan_candidates` 为空、`scan_discovery_hits` 非空时，把有效 completed 回退为 `discovery`。`extract`/`probe`/`gpt` 且候选为空视为该阶段零结果完成，不回退。
6. **内存：** 有 PG 时，每个 `DiscoverySource::fetch` 返回后立即 upsert hits 并丢弃 `host_hits`。后续 extract / probe / gpt 用 `load_discovery_hit_page` 按页读。凭证体积远小于 banner，允许在进程内累积候选；validate 有 PG 时按 `validate_batch_size` 分页读 `scan_candidates`，禁止 `LIMIT` 为 `i64::MAX`。
7. **非目标（本 ADR 不做）：** 改 `DiscoverySource` 为回调/channel（单源 fetch 仍可在源内缓冲）；ScanFacade 合并 CLI/API 装配；改 Redis lease。

## Consequences

- 提取中途崩溃后 resume 会从 `discovery` 之后重跑 extract，而不会空验证。
- 大预算 FOFA/Shodan 的 banner 不再在 Scanner 里 O(total) 常驻（单源 fetch 峰值仍存在）。
- Web 阶段文案从「提取与验证」拆成独立阶段；旧别名仅留在 `phase_label` 以免误读历史日志。
- 现网若已有 `phase=validate` 且无候选的中断 run，resume 会走撒谎修复重提取，而不是空跑。

## Alternatives considered

- **只改 resume 判断（phase≥validate 且无候选则重提取），不改写入时机：** 治标；下一次在 probe 中崩溃仍会把未完成标成 validate。
- **立刻把 DiscoverySource 改成按页回调：** 正确的下一步，但会同时改 FOFA/Shodan/GitHub 三个源；本 ADR 先收 Scanner 契约，源内缓冲留待后续。
- **UI 继续用粗阶段、DB 用细阶段：** 两套词汇会再次漂移；统一比多一层映射便宜。

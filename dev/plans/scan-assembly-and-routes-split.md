---
title: "扫描装配收口 + routes 拆分 实施方案"
type: plan
status: in_progress
updated: 2026-08-30
created: 2026-08-29
summary: "assemble_sources 落 services、SkippedSource 结构化回传、routes.rs 按域拆分并纳入覆盖率门禁"
---

# 扫描装配收口 + routes 拆分

> 决策：[ADR-003](../decisions/003-scan-assembly-in-services.md)
>
> **For agentic workers:** 每个 Task 以失败测试开头、以 commit 收尾。Phase 1 与 Phase 2 相互独立，可分别交付。

**Goal:** 发现源拼装收敛到 `aipocket-services::assemble_sources`（api/CLI 共用），被跳过的源经 `ScanStatus.skipped_sources` 结构化回传并在前端展示；`routes.rs`（1636 行）按域拆分并移出覆盖率 ignore。

**Architecture:** `SkippedSource` 落 `aipocket-core`；query/pack 组合下沉 `aipocket-discovery::compose_queries`；`assemble_sources` 在 `aipocket-services` 返回 `ScanPlan { sources, skipped }`；`scan_start` 与 CLI `run_scan` 都调它；`ScanManager` 新增 `set_skipped`。Phase 2 把 `routes.rs` 拆成 `routes/{mod,scan,keys,runs,cve,settings,honeypot,manual,system,shared}.rs`，补契约测试后从 CI 覆盖率 ignore 移除。

**Tech Stack:** Rust 2024（Axum + SQLx + reqwest + async-trait）、React 19 + TanStack Query + Vitest。

## Global Constraints

- 依赖方向 `api → services → discovery / prober / clients / db / core`，不得反向。
- Schema/DTO 变更只可加不可减；`ScanStatus` 新字段带 `#[serde(default)]`。
- 不新增无兼容默认值的 env；不改 Redis key 格式；不加破坏性迁移。
- 后端出站 HTTP 只走共享 `reqwest::Client`，禁止同步 HTTP。
- 验证：`cargo test --workspace`；`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings`；`cargo llvm-cov --workspace --fail-under-lines 86`。
- 前端：`cd frontend && pnpm lint && pnpm test -- --run && pnpm build`。
- 后端调用只经 `frontend/src/lib/api.ts`；UI 只用 `components/ui/`。
- 提交遵守 [文档门禁](../../docs/DOCUMENTATION.md)：实施 commit 同步行动层，用户可见行为变更同步知识层。

## File Structure

**Phase 1（装配收口 + skipped 回传）**

- Modify `crates/aipocket-core/src/models.rs`：`SkippedSource`；`ScanStatus.skipped_sources`。
- Create `crates/aipocket-discovery/src/queries.rs`：`ComposedQueries` + `compose_queries`（承接现 `routes.rs::discovery_queries` 与 github term 组合）。
- Modify `crates/aipocket-discovery/src/lib.rs`：`pub mod queries;` 与 re-export。
- Create `crates/aipocket-services/src/scan_assembly.rs`：`AssembleParams` / `ScanPlan` / `assemble_sources`。
- Modify `crates/aipocket-services/src/lib.rs`：re-export。
- Modify `crates/aipocket-api/src/scan_manager.rs`：`set_skipped`。
- Modify `crates/aipocket-api/src/routes.rs`：`scan_start` 改调 `assemble_sources`，删本地拼装与 `discovery_queries`。
- Modify `crates/aipocket/src/main.rs`：`run_scan` 改调 `assemble_sources`，逐条 `warn` skipped。
- Modify `crates/aipocket-api/tests/contract.rs`：`scan/status` 含 `skipped_sources`。
- Modify `frontend/src/lib/api.ts`：`SkippedSource` + `ScanStatusResponse.skipped_sources`。
- Modify `frontend/src/pages/ScanPage.tsx`：扫描台展示跳过源。
- Modify 知识层：`docs/reference/api.md`、`docs/systems/scan-pipeline.md`、`dev/progress/status.md`。

**Phase 2（routes 拆分 + 覆盖率）**

- Create `crates/aipocket-api/src/routes/mod.rs`（`router()` + `pub(crate)` 共享）及 `scan.rs` `keys.rs` `runs.rs` `cve.rs` `settings.rs` `honeypot.rs` `manual.rs` `system.rs` `shared.rs`。
- Delete `crates/aipocket-api/src/routes.rs`（内容迁入上列模块）。
- Modify `.github/workflows/main.yml`：从两处 `--ignore-filename-regex` 去掉 `crates/aipocket-api/src/routes\.rs`。
- Modify/Create `crates/aipocket-api/tests/`：补齐 handler 契约测试以维持 ≥86。

---

## Phase 1 — 装配收口 + skipped 回传

### Task 1：`SkippedSource` 与 `ScanStatus` 字段（core）

**Files:**
- Modify: `crates/aipocket-core/src/models.rs`
- Test: 同文件 `#[cfg(test)]`

**Interfaces:**
- Produces: `aipocket_core::SkippedSource { source: String, reason: String }`；`ScanStatus.skipped_sources: Vec<SkippedSource>`。

- [ ] **Step 1：写失败测试**（`models.rs` 测试模块内）

```rust
#[test]
fn scan_status_defaults_skipped_sources_empty() {
    let status = ScanStatus::default();
    assert!(status.skipped_sources.is_empty());
    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(json["skipped_sources"], serde_json::json!([]));
}

#[test]
fn scan_status_deserializes_without_skipped_sources() {
    let status: ScanStatus = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(status.skipped_sources.is_empty());
}
```

- [ ] **Step 2：跑测试确认失败**

Run: `cargo test -p aipocket-core scan_status_defaults_skipped_sources_empty`
Expected: 编译失败（`skipped_sources` 未定义）。

- [ ] **Step 3：实现**

在 `models.rs` 加结构，并给 `ScanStatus` 追加字段（放在 `log_seq` 之后）：

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkippedSource {
    pub source: String,
    pub reason: String,
}
```

```rust
    pub log_seq: u64,
    pub skipped_sources: Vec<SkippedSource>,
}
```

- [ ] **Step 4：跑测试确认通过**

Run: `cargo test -p aipocket-core`
Expected: PASS。

- [ ] **Step 5：commit**

```bash
git add crates/aipocket-core/src/models.rs
git commit -m "feat(core): add SkippedSource and ScanStatus.skipped_sources"
```

### Task 2：`compose_queries`（discovery）

**Files:**
- Create: `crates/aipocket-discovery/src/queries.rs`
- Modify: `crates/aipocket-discovery/src/lib.rs`
- Test: `crates/aipocket-discovery/src/queries.rs` `#[cfg(test)]`

**Interfaces:**
- Consumes: `packs::ProviderPack`、`legacy_queries::{fofa_queries, shodan_product_queries, prioritize_fofa_queries}`。
- Produces: `ComposedQueries { fofa: Vec<String>, shodan: Vec<String>, github: Vec<String> }`；`pub fn compose_queries(selected_packs: &[&ProviderPack]) -> ComposedQueries`。

- [ ] **Step 1：写失败测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs;

    #[test]
    fn compose_matches_web_full_scan_shape() {
        let registry = packs::registry();
        let selected = registry.values().copied().collect::<Vec<_>>();
        let q = compose_queries(&selected);
        assert!(q.fofa.len() > 60);
        assert!(q.fofa.iter().any(|s| s.contains("api.deepseek.com") && s.contains("sk-")));
        assert!(q.fofa.first().is_some_and(|s| s.starts_with("header=")));
        assert!(q.shodan.iter().any(|s| s == "http.html:sk-"));
        assert!(!q.github.is_empty());
    }
}
```

- [ ] **Step 2：跑测试确认失败**

Run: `cargo test -p aipocket-discovery compose_matches_web_full_scan_shape`
Expected: 编译失败（`compose_queries` 未定义）。

- [ ] **Step 3：实现**（`queries.rs`；逻辑与现 `routes.rs::discovery_queries` + `scan_start` github term 组合一致）

```rust
use crate::legacy_queries::{fofa_queries, prioritize_fofa_queries, shodan_product_queries};
use crate::packs::ProviderPack;

#[derive(Clone, Debug, Default)]
pub struct ComposedQueries {
    pub fofa: Vec<String>,
    pub shodan: Vec<String>,
    pub github: Vec<String>,
}

pub fn compose_queries(selected_packs: &[&ProviderPack]) -> ComposedQueries {
    let mut fofa = fofa_queries();
    fofa.extend(
        selected_packs
            .iter()
            .flat_map(|pack| pack.fofa_queries)
            .map(|q| q.to_string()),
    );
    let mut shodan = selected_packs
        .iter()
        .flat_map(|pack| pack.shodan_queries)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    shodan.extend(shodan_product_queries());
    let mut github = selected_packs
        .iter()
        .flat_map(|pack| pack.github_terms)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    prioritize_fofa_queries(&mut fofa);
    prioritize_fofa_queries(&mut shodan);
    prioritize_fofa_queries(&mut github);
    ComposedQueries { fofa, shodan, github }
}
```

在 `lib.rs` 加 `pub mod queries;` 与 `pub use queries::{ComposedQueries, compose_queries};`。

- [ ] **Step 4：跑测试确认通过**

Run: `cargo test -p aipocket-discovery`
Expected: PASS。

- [ ] **Step 5：commit**

```bash
git add crates/aipocket-discovery/src/queries.rs crates/aipocket-discovery/src/lib.rs
git commit -m "feat(discovery): add compose_queries as single source of scan queries"
```

### Task 3：`assemble_sources`（services）

**Files:**
- Create: `crates/aipocket-services/src/scan_assembly.rs`
- Modify: `crates/aipocket-services/src/lib.rs`
- Test: `crates/aipocket-services/src/scan_assembly.rs` `#[cfg(test)]`

**Interfaces:**
- Consumes: `aipocket_core::{Settings, SkippedSource}`、`aipocket_discovery::{DiscoverySource, compose_queries, packs}`、`aipocket_clients::{FofaClient, ShodanClient, GithubClient}`、`reqwest::Client`。
- Produces:

```rust
pub struct AssembleParams {
    pub requested: Vec<String>,       // ["all"] | ["fofa","github"] ...
    pub github_pack_ids: Vec<String>, // [] | ["all"] = 全部 pack
    pub manual_enrich: Vec<String>,   // ["fofa","shodan"]
    pub resume_run_id: String,
    pub manual_targets: Vec<String>,  // 由调用方预取
}
pub struct ScanPlan {
    pub sources: Vec<std::sync::Arc<dyn aipocket_discovery::DiscoverySource>>,
    pub skipped: Vec<aipocket_core::SkippedSource>,
}
pub fn assemble_sources(settings: &Settings, http: &reqwest::Client, params: &AssembleParams) -> ScanPlan;
```

- [ ] **Step 1：写失败测试**（纯函数，无网络/DB）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use aipocket_core::Settings;

    fn http() -> reqwest::Client { reqwest::Client::new() }

    fn params(requested: &[&str]) -> AssembleParams {
        AssembleParams {
            requested: requested.iter().map(|s| s.to_string()).collect(),
            github_pack_ids: vec![],
            manual_enrich: vec![],
            resume_run_id: String::new(),
            manual_targets: vec![],
        }
    }

    #[test]
    fn github_skipped_without_token_and_pg() {
        let settings = Settings::default(); // no github token, no DATABASE_URL
        let plan = assemble_sources(&settings, &http(), &params(&["github"]));
        assert!(plan.sources.iter().all(|s| s.name() != "github"));
        assert!(plan.skipped.iter().any(|s| s.source == "github"));
    }

    #[test]
    fn fofa_skipped_without_key() {
        let settings = Settings::default();
        let plan = assemble_sources(&settings, &http(), &params(&["fofa"]));
        assert!(plan.sources.iter().all(|s| s.name() != "fofa"));
        assert!(plan.skipped.iter().any(|s| s.source == "fofa" && s.reason.contains("FOFA_KEYS")));
    }

    #[test]
    fn fofa_present_with_key() {
        let mut settings = Settings::default();
        settings.fofa_keys = "k1".into();
        let plan = assemble_sources(&settings, &http(), &params(&["fofa"]));
        assert!(plan.sources.iter().any(|s| s.name() == "fofa"));
        assert!(plan.skipped.iter().all(|s| s.source != "fofa"));
    }
}
```

- [ ] **Step 2：跑测试确认失败**

Run: `cargo test -p aipocket-services github_skipped_without_token_and_pg`
Expected: 编译失败（`assemble_sources` 未定义）。

- [ ] **Step 3：实现**（gating 表见 ADR-003；query 组合走 `compose_queries`）

```rust
use std::sync::Arc;
use aipocket_core::{Settings, SkippedSource};
use aipocket_clients::{FofaClient, GithubClient, ShodanClient};
use aipocket_discovery::{
    DiscoverySource, compose_queries,
    legacy_queries::prioritize_fofa_queries,
    packs,
    sources::{FofaSource, GithubSource, ManualEnrichSource, ManualSource, ShodanSource},
};

pub struct AssembleParams { /* 见 Interfaces */ }
pub struct ScanPlan { /* 见 Interfaces */ }

pub fn assemble_sources(settings: &Settings, http: &reqwest::Client, params: &AssembleParams) -> ScanPlan {
    let wants = |name: &str| params.requested.iter().any(|v| v == "all" || v == name);
    let registry = packs::registry();
    let selected: Vec<_> = if params.github_pack_ids.is_empty()
        || params.github_pack_ids.iter().any(|v| v == "all")
    {
        registry.values().copied().collect()
    } else {
        params
            .github_pack_ids
            .iter()
            .filter_map(|id| registry.get(id.as_str()).copied())
            .collect()
    };
    let q = compose_queries(&selected);

    let mut sources: Vec<Arc<dyn DiscoverySource>> = Vec::new();
    let mut skipped: Vec<SkippedSource> = Vec::new();
    let mut skip = |src: &str, reason: &str, skipped: &mut Vec<SkippedSource>| {
        skipped.push(SkippedSource { source: src.into(), reason: reason.into() });
    };

    if wants("fofa") {
        if settings.fofa_key_list().is_empty() {
            skip("fofa", "missing FOFA_KEYS", &mut skipped);
        } else {
            sources.push(Arc::new(FofaSource {
                client: FofaClient::new(http.clone(), settings),
                queries: q.fofa.clone(),
                page_size: settings.fofa_page_size,
                max_pages: settings.fofa_max_pages,
                page_delay: settings.fofa_page_delay,
            }));
        }
    }
    if wants("shodan") {
        if settings.shodan_key_list().is_empty() {
            skip("shodan", "missing SHODAN_KEYS", &mut skipped);
        } else {
            sources.push(Arc::new(ShodanSource {
                client: ShodanClient::new(http.clone(), settings),
                queries: q.shodan.clone(),
                max_pages: settings.shodan_max_pages,
                page_delay: settings.shodan_page_delay,
            }));
        }
    }
    if wants("github") {
        if settings.github_token_list().is_empty() {
            skip("github", "missing GITHUB_TOKENS", &mut skipped);
        } else if !settings.pg_enabled() {
            skip("github", "requires DATABASE_URL", &mut skipped);
        } else {
            let mut queries = q.github.clone();
            prioritize_fofa_queries(&mut queries);
            sources.push(Arc::new(GithubSource {
                client: GithubClient::new(http.clone(), settings),
                queries,
                per_page: settings.github_search_page_size,
                run_id: params.resume_run_id.clone(),
                pack_id: if params.github_pack_ids.len() == 1 {
                    params.github_pack_ids[0].clone()
                } else {
                    String::new()
                },
            }));
        }
    }
    if params.requested.iter().any(|v| v == "manual") {
        if params.manual_targets.is_empty() {
            skip("manual", "no manual targets", &mut skipped);
        } else {
            sources.push(Arc::new(ManualSource { targets: params.manual_targets.clone() }));
            let engines: Vec<String> = params
                .manual_enrich
                .iter()
                .map(|e| e.trim().to_ascii_lowercase())
                .filter(|e| e == "fofa" || e == "shodan")
                .collect();
            if !engines.is_empty() {
                sources.push(Arc::new(ManualEnrichSource {
                    targets: params.manual_targets.clone(),
                    engines,
                    fofa: FofaClient::new(http.clone(), settings),
                    shodan: ShodanClient::new(http.clone(), settings),
                }));
            }
        }
    }
    ScanPlan { sources, skipped }
}
```

在 `lib.rs` 加 `pub mod scan_assembly;` 与 `pub use scan_assembly::{AssembleParams, ScanPlan, assemble_sources};`。

> 注意 `compose_queries` 已对 github 做过一次 `prioritize_fofa_queries`；此处对 GithubSource 再排一次是保持与旧 `scan_start` 完全一致的行为，可接受（幂等）。

- [ ] **Step 4：跑测试确认通过**

Run: `cargo test -p aipocket-services scan_assembly`
Expected: PASS。

- [ ] **Step 5：commit**

```bash
git add crates/aipocket-services/src/scan_assembly.rs crates/aipocket-services/src/lib.rs
git commit -m "feat(services): assemble_sources as single source of scan discovery wiring"
```

### Task 4：`ScanManager::set_skipped`

**Files:**
- Modify: `crates/aipocket-api/src/scan_manager.rs`
- Test: `crates/aipocket-api/tests/scan_manager.rs`

**Interfaces:**
- Produces: `pub async fn set_skipped(&self, skipped: Vec<aipocket_core::SkippedSource>)` —— 写入 `status.skipped_sources` 并对每条 push 一行日志。

- [ ] **Step 1：写失败测试**（`tests/scan_manager.rs` 新增）

```rust
#[tokio::test]
async fn set_skipped_records_status_and_logs() {
    let manager = std::sync::Arc::new(ScanManager::new(64));
    manager
        .set_skipped(vec![aipocket_core::SkippedSource {
            source: "github".into(),
            reason: "missing GITHUB_TOKENS".into(),
        }])
        .await;
    let status = manager.status().await;
    assert_eq!(status.skipped_sources.len(), 1);
    assert_eq!(status.skipped_sources[0].source, "github");
    assert!(manager.log_text().await.contains("github"));
}
```

- [ ] **Step 2：跑测试确认失败**

Run: `cargo test -p aipocket-api set_skipped_records_status_and_logs`
Expected: 编译失败（`set_skipped` 未定义）。

- [ ] **Step 3：实现**（放在 `set_options` 附近）

```rust
    pub async fn set_skipped(&self, skipped: Vec<aipocket_core::SkippedSource>) {
        if skipped.is_empty() {
            return;
        }
        for item in &skipped {
            self.push_log(format!("跳过数据源 · {} · {}", item.source, item.reason))
                .await;
        }
        self.status.write().await.skipped_sources = skipped;
    }
```

- [ ] **Step 4：跑测试确认通过**

Run: `cargo test -p aipocket-api set_skipped_records_status_and_logs`
Expected: PASS。

- [ ] **Step 5：commit**

```bash
git add crates/aipocket-api/src/scan_manager.rs crates/aipocket-api/tests/scan_manager.rs
git commit -m "feat(api): ScanManager.set_skipped surfaces skipped sources on status and log"
```

### Task 5：`scan_start` 改用 `assemble_sources`

**Files:**
- Modify: `crates/aipocket-api/src/routes.rs:1430-1577`（`scan_start`）；删除 `discovery_queries`（1315-1334）及其单测中的 `web_full_scan_includes_legacy_product_queries`（已被 Task 2 覆盖）。
- Test: `crates/aipocket-api/tests/contract.rs`

**Interfaces:**
- Consumes: `aipocket_services::{AssembleParams, assemble_sources}`、`ScanManager::set_skipped`。

- [ ] **Step 1：写/改契约测试**（`contract.rs`：`scan/status` 响应含 `skipped_sources` 数组，默认空）

```rust
#[tokio::test]
async fn scan_status_exposes_skipped_sources_field() {
    let app = test_app().await; // 复用现有测试装配
    let res = get_json(&app, "/api/scan/status").await;
    assert!(res["skipped_sources"].is_array());
}
```

> 若 `contract.rs` 无 `test_app`/`get_json`，复用该文件既有的 app 构造与请求辅助（保持与邻近测试一致的写法）。

- [ ] **Step 2：跑测试确认失败**

Run: `cargo test -p aipocket-api scan_status_exposes_skipped_sources_field`
Expected: FAIL（字段缺失或未回传）。

- [ ] **Step 3：实现**：把 `scan_start` spawn 内的发现源拼装整段替换为：

```rust
    let manual_targets = if sources.iter().any(|v| v == "manual") {
        scanner.manual_targets().await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let plan = aipocket_services::assemble_sources(
        &settings,
        &http,
        &aipocket_services::AssembleParams {
            requested: sources.clone(),
            github_pack_ids: b.github_pack_ids.clone(),
            manual_enrich: b.manual_enrich.clone(),
            resume_run_id: b.resume_run_id.clone(),
            manual_targets,
        },
    );
    s.scan_manager.set_skipped(plan.skipped).await;
    // ...随后把 plan.sources 传给 scanner.run_resumable(...)
```

删除本地 `FofaSource/ShodanSource/GithubSource/ManualSource/ManualEnrichSource` 构造、`discovery_queries` 函数、以及 `scan_query_tests` 中 `web_full_scan_includes_legacy_product_queries`（迁往 Task 2）。`set_skipped` 需在 `set_options` 之后、`scanner.run` 之前调用（此时 status 已是 running，不会被 `start_channel` 重置覆盖）。

- [ ] **Step 4：跑测试确认通过**

Run: `cargo test -p aipocket-api && cargo clippy -p aipocket-api --all-targets -- -D warnings`
Expected: PASS，无 clippy 告警（确认 `aipocket_discovery` / 未用 import 已清理）。

- [ ] **Step 5：commit**

```bash
git add crates/aipocket-api/src/routes.rs crates/aipocket-api/tests/contract.rs
git commit -m "refactor(api): scan_start uses assemble_sources; report skipped sources"
```

### Task 6：CLI `run_scan` 改用 `assemble_sources`

**Files:**
- Modify: `crates/aipocket/src/main.rs:223-296`（`run_scan`）
- Test: 手动验证 + 现有 `cargo test --workspace`（CLI 拼装无独立单测，逻辑已被 Task 3 覆盖）

**Interfaces:**
- Consumes: `aipocket_services::{AssembleParams, assemble_sources}`。

- [ ] **Step 1：实现**：把 `run_scan` 里 registry/query/source 构造整段替换为：

```rust
    let manual_targets = if source == "manual" {
        scanner.manual_targets().await?
    } else {
        Vec::new()
    };
    let plan = aipocket_services::assemble_sources(
        &settings,
        &http,
        &aipocket_services::AssembleParams {
            requested: vec![source.clone()],
            github_pack_ids: Vec::new(), // CLI 走全部 pack
            manual_enrich: Vec::new(),
            resume_run_id: resume.clone().unwrap_or_default(),
            manual_targets,
        },
    );
    for item in &plan.skipped {
        tracing::warn!(source = %item.source, reason = %item.reason, "discovery source skipped");
    }
    let sources = plan.sources;
```

删除 `run_scan` 内 registry / `fofa_queries` / `shodan_queries` / 各 `sources.push(...)` 旧块。

> 行为变更（有意）：CLI 现在与 Web 一致地对无 key 的 fofa/shodan fail closed，并支持 pack 语义（此处仍传全部 pack，保持 CLI 旧的「全量」默认）。

- [ ] **Step 2：跑构建与测试**

Run: `cargo build -p aipocket && cargo test --workspace`
Expected: PASS。

- [ ] **Step 3：commit**

```bash
git add crates/aipocket/src/main.rs
git commit -m "refactor(cli): run_scan uses assemble_sources; log skipped sources"
```

### Task 7：前端展示 skipped

**Files:**
- Modify: `frontend/src/lib/api.ts:150-165`
- Modify: `frontend/src/pages/ScanPage.tsx`
- Test: `frontend/src/pages/` 现有测试 + `pnpm build`

**Interfaces:**
- Consumes: `ScanStatusResponse.skipped_sources`。

- [ ] **Step 1：api.ts 加类型**

```ts
export interface SkippedSource {
  source: string
  reason: string
}
```

在 `ScanStatusResponse` 追加：

```ts
  /** Sources fail-closed at assembly time, e.g. github without token/PG. */
  skipped_sources?: SkippedSource[]
```

- [ ] **Step 2：ScanPage 渲染**：在状态区（`phaseLabel` 附近，约 854 行的 running/phase 块内）加一段——当 `status?.skipped_sources?.length` 时，用现有 `components/ui` 的展示元素列出 `source · reason`（跟随现有配色，跳过项用 muted/警示色）。示例：

```tsx
{status?.skipped_sources?.length ? (
  <div className="mt-2 space-y-1 font-mono text-xs text-amber-500">
    {status.skipped_sources.map((s) => (
      <div key={s.source}>跳过 {s.source} · {s.reason}</div>
    ))}
  </div>
) : null}
```

- [ ] **Step 3：验证**

Run: `cd frontend && pnpm lint && pnpm test -- --run && pnpm build`
Expected: PASS。

- [ ] **Step 4：commit**

```bash
git add frontend/src/lib/api.ts frontend/src/pages/ScanPage.tsx
git commit -m "feat(frontend): show skipped discovery sources in scan console"
```

### Task 8：Phase 1 文档同步

**Files:**
- Modify: `docs/reference/api.md`（`scan/status` 增列 `skipped_sources`）
- Modify: `docs/systems/scan-pipeline.md`（发现源装配段引用 `assemble_sources` + 跳过可见）
- Modify: `dev/progress/status.md`（Completed / Current session）
- Modify: 本 plan `status: draft → implemented`；`dev/plans/INDEX.md` 状态列

- [ ] **Step 1：更新知识层与行动层**（不复制 gating 表，链到 ADR-003）
- [ ] **Step 2：`status.md` 总行数 ≤ 200，超出迁 archive**
- [ ] **Step 3：commit**

```bash
git add docs/reference/api.md docs/systems/scan-pipeline.md dev/progress/status.md dev/plans/scan-assembly-and-routes-split.md dev/plans/INDEX.md
git commit -m "docs: record scan assembly consolidation and skipped_sources"
```

---

## Phase 2 — routes.rs 拆分 + 覆盖率门禁

> 目标：把 `routes.rs` 拆成按域模块，行为零变化（纯移动 + 可见性调整），补契约测试后移出覆盖率 ignore。每个 Task 后 `cargo test -p aipocket-api` 必须绿。

### Task 9：建立 `routes/` 骨架与 `shared.rs`

**Files:**
- Create: `crates/aipocket-api/src/routes/mod.rs`、`crates/aipocket-api/src/routes/shared.rs`
- Modify: `crates/aipocket-api/src/lib.rs`（`pub mod routes;` 路径不变，Rust 自动识别 `routes/mod.rs`）

**Interfaces:**
- Produces: `pub fn router() -> Router<AppState>`（签名不变）；`shared.rs` 放 `pub(crate)` 通用件：`valid_kind`、`all_source`、`mask`、`Since`、以及被多域复用的 DTO/helper。

- [ ] **Step 1**：新建 `routes/mod.rs`，暂时 `mod` 引入后续域模块并保留 `router()` 里全部 `.route(...)`（handler 先从各 `pub(crate)` 域模块 `use` 进来）。先把 `shared.rs` 抽出并让 `routes.rs` 变为 `routes/mod.rs`（`git mv`）。
- [ ] **Step 2**：`cargo test -p aipocket-api` 绿（此步仅重命名 + 抽 shared，无逻辑变化）。
- [ ] **Step 3**：commit

```bash
git add -A crates/aipocket-api/src
git commit -m "refactor(api): convert routes.rs into routes/ module with shared helpers"
```

### Task 10–14：按域迁移 handler（每域一个 Task）

每个 Task 把下列 handler（含各自 `#[cfg(test)]`）从 `mod.rs` 迁入对应文件，改为 `pub(crate) async fn`，在 `mod.rs` 保留 `.route()` 注册（`use` 自域模块）。**每步末 `cargo test -p aipocket-api` 必须绿。**

- [ ] **Task 10 · `runs.rs`**：`runs` `run_results` `run_log` `delete_run` `gpt_failed` `retry_gpt_failed`。commit `refactor(api): move run handlers into routes/runs.rs`
- [ ] **Task 11 · `keys.rs`**：`transition_keys` `all_keys` `high_value` `high_value_reveal` `key_models` `key_balance` `keys_balance` `key_chat` `key_reveal` `export` `probe_and_persist_balance` + helper `definitive_expiry` `normalized_batch_provider` `MAX_BATCH_BALANCE_KEYS`，以及 `key_probe_tests`、`scan_query_tests` 中 `batch_provider_normalization_is_strict_and_case_insensitive`。commit `refactor(api): move key/balance handlers into routes/keys.rs`
- [ ] **Task 12 · `cve.rs`**：`cves` `cve_sync` `cve_add` + `cve_records_from_search_item` 及其测试。commit `refactor(api): move cve handlers into routes/cve.rs`
- [ ] **Task 13 · `honeypot.rs` + `manual.rs`**：蜜罐 5 个 + 手工目标 4 个 handler。commit `refactor(api): move honeypot and manual-target handlers`
- [ ] **Task 14 · `settings.rs` + `scan.rs` + `system.rs`**：设置/check 6 个；`scan_start` `scan_stop` `scan_status` `scan_logs` `scan_stream` + `ScanStart`/`Since` DTO；`system_restart`；`health` 留 `mod.rs`。commit `refactor(api): move settings/scan/system handlers`

> 迁移准则：只移动与改可见性，不改函数体；跨域复用的私有件下沉 `shared.rs`；`mod.rs` 只留 `router()` + `health` + `mod`/`use`。

### Task 15：补契约测试 + 移出覆盖率 ignore

**Files:**
- Modify: `.github/workflows/main.yml`（两处 `--ignore-filename-regex` 去掉 `crates/aipocket-api/src/routes\.rs`）
- Create/Modify: `crates/aipocket-api/tests/*`（对未覆盖 handler 补契约测试）

- [ ] **Step 1**：本地测覆盖率门槛

Run: `cargo llvm-cov --workspace --summary-only --fail-under-lines 86 --ignore-filename-regex 'crates/aipocket/src/(main|maintenance)\.rs' -- --include-ignored`
Expected: 若 < 86，按报告对缺口 handler（如 export 分支、balance 分支）补契约测试，直至通过。

- [ ] **Step 2**：更新 CI workflow 两行正则（删 `|crates/aipocket-api/src/routes\.rs`）。
- [ ] **Step 3**：`cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- [ ] **Step 4：commit**

```bash
git add crates/aipocket-api/tests .github/workflows/main.yml
git commit -m "test(api): cover route handlers and drop routes coverage exemption"
```

### Task 16：Phase 2 文档同步

- [ ] 更新 `docs/DOCUMENTATION.md`、`docs/INDEX.md`、`docs/reference/api.md` 中「HTTP 真源 = routes.rs」的表述为 `routes/`（路径以模块目录为准）。
- [ ] `dev/progress/status.md` 记 Completed。
- [ ] commit `docs: routes split; http source of truth is routes/ module`

---

## Self-Review

- **Spec coverage**：ADR-003 五条决策 → Task 1（SkippedSource/ScanStatus）、Task 2（compose_queries 单一真源）、Task 3（assemble_sources + gating）、Task 4/5（api 回传）、Task 6（CLI 一致）、Task 7（前端展示）；routes 拆分与覆盖率 → Task 9–16。
- **行为变更点**：无 key 的 fofa/shodan 由「挂上后逐条报错」改为「skipped 可见」——已在 ADR-003 与 Task 5/6 显式标注。
- **类型一致**：`SkippedSource`（core）贯穿 services/api/前端；`ScanPlan.sources` 为 `Vec<Arc<dyn DiscoverySource>>`，与 `scanner.run_resumable` 入参一致。
- **依赖方向**：assemble 在 services 用 discovery/clients/core；api 只 `use` services 装配器 + discovery trait 类型；未反向。
- **Phase 独立性**：Phase 1 交付后系统可用；Phase 2 为纯结构重构，可单独排期。

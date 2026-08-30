---
title: "Readiness + 敏感操作审计 实施方案"
type: plan
status: implemented
updated: 2026-08-30
created: 2026-08-30
summary: "/api/ready 探 PG/Redis；audit_events 记 reveal/chat/export/restart；run_log 去掉 blocking_read"
---

# Readiness + 敏感操作审计

> 决策：[ADR-004](../decisions/004-readiness-and-audit.md)

**Goal:** 编排器能区分保活与依赖就绪；reveal/chat/export/restart 留下不含明文密钥的审计；`run_log` 不再在 async 路径阻塞读锁。

**Architecture:** `audit_events` 加表走现有 `ensure_schema`。探测函数放 `aipocket-db`（`postgres_ready` / `redis_ready`）。API 层组 `/api/ready` 与审计写入。`client_key(&HeaderMap)` 从 `auth.rs` 抽出供登录节流与审计共用。

**Tech Stack:** Axum、SQLx、redis-rs、tokio RwLock。

## Global Constraints

- 依赖方向 `api → services → db / core`。探测放 db，不放 handler 里直接拼 SQL 以外的业务。
- Schema 只 `IF NOT EXISTS` / `ADD COLUMN IF NOT EXISTS`。
- 不新增无默认值的必填 env。
- 审计 `detail` 不得含明文 `apikey`。
- `/api/health` 响应形状保持 `{ok:true}`。
- 验证：`cargo test --workspace`；fmt；clippy `-D warnings`。

## File Structure

- Modify `migrations/schema.sql`：`audit_events`
- Modify `crates/aipocket-db/src/postgres.rs`：`postgres_ready`
- Modify `crates/aipocket-db/src/scan_lock.rs` 或新 `ready.rs`：`redis_ready`
- Modify `crates/aipocket-db/src/repository.rs`：`insert_audit` / `list_audit`
- Modify `crates/aipocket-db/src/lib.rs`：re-export
- Modify `crates/aipocket-api/src/auth.rs`：`pub fn client_key(headers: &HeaderMap) -> String`
- Modify `crates/aipocket-api/src/routes/mod.rs`：注册 `/api/ready`、`/api/audit`
- Modify `crates/aipocket-api/src/routes/runs.rs`：`blocking_read` → `read().await`
- Modify `crates/aipocket-api/src/routes/keys.rs`：reveal/chat/export 记审计
- Modify `crates/aipocket-api/src/routes/system.rs`：restart 先审计
- Create `crates/aipocket-api/src/routes/ops.rs`（可选：`ready` + `list_audit`，避免再膨胀 `mod.rs`）
- Test: `crates/aipocket-api/tests/contract.rs`；有 PG 时 `postgres_compat` 插一条审计
- Docs: `docs/systems/auth.md`、`docs/reference/api.md`、`docs/systems/storage.md`、`docs/guides/security.md`、`dev/progress/status.md`

---

### Task 1：`client_key` + 修 `run_log` 阻塞读

**Files:** `crates/aipocket-api/src/auth.rs`、`crates/aipocket-api/src/routes/runs.rs`、`crates/aipocket-api/tests/contract.rs`

- [ ] 从 `login` 抽出 `pub fn client_key(headers: &HeaderMap) -> String`（逻辑不变：XFF 第一段，否则 `"unknown"`）。`login` 改调它。
- [ ] `run_log`：在 `or_else` 之前 `let settings = s.settings.read().await;`，用 `settings.results_path()` 读 `run.log`。删除 `blocking_read()`。
- [ ] 单测：`client_key` 对空头 → `"unknown"`；对 `x-forwarded-for: 1.1.1.1, 2.2.2.2` → `"1.1.1.1"`。
- [ ] `cargo test -p aipocket-api`
- [ ] commit `fix(api): extract client_key and stop blocking_read in run_log`

### Task 2：schema `audit_events` + Repository

**Files:** `migrations/schema.sql`、`crates/aipocket-db/src/repository.rs`、`crates/aipocket-db/src/lib.rs`、`crates/aipocket-db/tests/postgres_compat.rs`

```sql
CREATE TABLE IF NOT EXISTS audit_events (
    id     BIGSERIAL PRIMARY KEY,
    at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    action TEXT NOT NULL,
    client TEXT NOT NULL,
    detail JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX IF NOT EXISTS audit_events_at_desc ON audit_events (at DESC);
```

```rust
pub struct AuditEvent {
    pub action: String,
    pub client: String,
    pub detail: serde_json::Value,
}
// insert_audit: 无 pool → Ok(())；有 pool → INSERT
// list_audit(limit: i64) -> Vec<Value> 按 at DESC；无 pool → Ok(vec![])
```

- [ ] PG 测试：insert 两条，list(1) 只返回最新且 `detail` 无 `apikey` 明文（测一条 `{"masked":"sk-****abcd"}`）。
- [ ] commit `feat(db): add audit_events table and repository helpers`

### Task 3：`postgres_ready` / `redis_ready`

**Files:** `crates/aipocket-db/src/postgres.rs`、`crates/aipocket-db/src/lib.rs`（可把 redis ping 放 `scan_lock.rs` 旁的小函数）

```rust
pub async fn postgres_ready(pool: &PgPool) -> Result<()> {
    sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool).await?;
    Ok(())
}
pub async fn redis_ready(redis_url: &str) -> Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = redis::aio::ConnectionManager::new(client).await?;
    let _: String = redis::cmd("PING").query_async(&mut conn).await?;
    Ok(())
}
```

- [ ] 无网络单测不必强求；`cargo test -p aipocket-db` 现有测试绿即可。PG/Redis 探测由 Task 4 契约覆盖。
- [ ] commit `feat(db): postgres_ready and redis_ready probes`

### Task 4：`GET /api/ready` + `GET /api/audit`

**Files:** 新建 `crates/aipocket-api/src/routes/ops.rs`；改 `mod.rs` 注册（`/api/ready` 无 Auth；`/api/audit` 要 Auth）

`ready` 形状：

```json
{"ok":true,"degraded":false,"postgres":"ok|skipped|error","redis":"ok|skipped|error"}
```

- postgres：`settings.pg_enabled()` 假 → skipped；真则对 `repository.pool()` 调 `postgres_ready`，无 pool 当 error。
- redis：`!dedup_enabled` → skipped；否则 `redis_ready(&dedup_redis_url)`。
- `ok` = postgres 不是 error；`degraded` = redis == error。postgres error → `StatusCode::SERVICE_UNAVAILABLE`。

`audit`：`Query { limit: Option<u32> }` 默认 100 clamp 1..=500；`repository.list_audit`。

- [ ] 契约：`GET /api/ready` 无 token → 200，含 `postgres`/`redis` 字段（测试 app 无 DATABASE_URL → postgres skipped）。
- [ ] `GET /api/audit` 无 token → 401；有 token → 200 且 `events` 为数组。
- [ ] `GET /api/health` 仍 `{ok:true}`。
- [ ] commit `feat(api): add /api/ready and /api/audit`

### Task 5：敏感操作写入审计

**Files:** `keys.rs`（`key_reveal`、`high_value_reveal`、`key_chat`、`export`）、`system.rs`

抽 `async fn record_audit(s: &AppState, headers: &HeaderMap, action: &str, detail: Value)`：`tracing::info` + `repository.insert_audit`（错误只 `tracing::warn`）。

各成功返回前调用：

| action | detail |
|--------|--------|
| `reveal` | `masked`, `run_id`, `kind` |
| `high_value_reveal` | `masked` |
| `chat` | `masked`（`mask_apikey`）, `model` |
| `export` | `dataset`, `format`, `count`（导出条数） |
| `restart` | `{}` |

`key_chat` / `export` / reveal handlers 增加 `headers: HeaderMap` 参数。

- [ ] 契约：login 后 `POST /api/key/reveal` 对空库 404，**不**要求写审计；对 `POST /api/system/restart` **不要**在单测里打（会 `exit`）。restart 用 `#[cfg(test)]` 可测的 `record_audit` 或对 `insert_audit` 的 db 测试覆盖。
- [ ] 另测：有 token 时 `GET /api/audit` 在一次成功 `export`（空 selected → 按现有行为）之后，若 handler 在失败/空导出仍返回 200，则应有 `action=export` 事件；若空导出 400，则跳过，改用直接 `insert_audit` 的 API 单测不必。**最低要求：** `record_audit` 对无 pool 的 `Repository::default()` 不 panic。
- [ ] commit `feat(api): audit reveal, chat, export, and restart`

### Task 6：文档

- [ ] `docs/reference/api.md`：health vs ready；audit GET；敏感操作会记审计。`updated: 2026-08-30`
- [ ] `docs/systems/auth.md`：ready 未鉴权；审计字段；`client_key`
- [ ] `docs/systems/storage.md`：表 `audit_events`
- [ ] `docs/guides/security.md`：一行链到审计
- [ ] `dev/progress/status.md`；本 plan `status: implemented`；`dev/plans/INDEX.md`
- [ ] `python3 scripts/check-docs-health.py`
- [ ] commit `docs: readiness endpoint and sensitive-operation audit`

## Self-Review

- ADR 六条 → Task 1（blocking_read + client_key）、2（表）、3（探测）、4（ready/audit HTTP）、5（写入点）、6（文档）。
- 明文密钥：detail 白名单字段，测试断言无 raw key。
- `/api/health` 形状不变。
- Redis 503：明确不做。

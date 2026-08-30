use crate::{
    auth::{Auth, verify},
    error::ApiError,
    settings::{SettingsUpdate, SettingsView, persist_env},
    state::AppState,
};
use aipocket_core::{Credential, ScanMode, ScanStatus};
use aipocket_db::mask_apikey;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{convert::Infallible, path::PathBuf, time::Duration};
use tokio_stream::wrappers::BroadcastStream;

mod keys;
mod runs;
mod shared;
use keys::{
    all_keys, export, high_value, high_value_reveal, key_balance, key_chat, key_models, key_reveal,
    keys_balance, transition_keys,
};
use runs::{delete_run, gpt_failed, retry_gpt_failed, run_log, run_results, runs};
use shared::{Since, all_source, valid_kind};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", post(crate::auth::login))
        .route("/api/auth/logout", post(crate::auth::logout))
        .route("/api/runs", get(runs))
        .route("/api/runs/{id}/{kind}", get(run_results))
        .route("/api/runs/{id}/log", get(run_log))
        .route("/api/runs/{id}", axum::routing::delete(delete_run))
        .route("/api/runs/{id}/gpt-failed", get(gpt_failed))
        .route("/api/runs/{id}/retry-gpt-failed", post(retry_gpt_failed))
        .route("/api/high-value", get(high_value))
        .route("/api/high-value/reveal", post(high_value_reveal))
        .route("/api/keys/{kind}", get(all_keys))
        .route("/api/keys/status", post(transition_keys))
        .route("/api/key/models", post(key_models))
        .route("/api/key/balance", post(key_balance))
        .route("/api/keys/balance", post(keys_balance))
        .route("/api/key/chat", post(key_chat))
        .route("/api/key/reveal", post(key_reveal))
        .route("/api/export", post(export))
        .route("/api/cve", get(cves))
        .route("/api/cve/sync", post(cve_sync))
        .route("/api/cve/add", post(cve_add))
        .route(
            "/api/honeypot",
            get(honeypots)
                .post(create_honeypot)
                .patch(update_honeypot)
                .delete(delete_honeypot),
        )
        .route("/api/honeypot/bulk-delete", post(bulk_delete_honeypots))
        .route(
            "/api/manual-targets",
            get(manual_targets)
                .post(save_manual_targets)
                .delete(delete_manual_target),
        )
        .route(
            "/api/manual-targets/bulk-delete",
            post(bulk_delete_manual_targets),
        )
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/settings/check/fofa", post(check_fofa))
        .route("/api/settings/check/shodan", post(check_shodan))
        .route("/api/settings/check/github", post(check_github))
        .route("/api/settings/check/tavily", post(check_tavily))
        .route("/api/scan/start", post(scan_start))
        .route("/api/scan/stop", post(scan_stop))
        .route("/api/scan/status", get(scan_status))
        .route("/api/scan/logs", get(scan_logs))
        .route("/api/scan/logs/stream", get(scan_stream))
        .route("/api/system/restart", post(system_restart))
}
async fn cve_sync(_: Auth, State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let queries = [
        "latest AI infrastructure security CVE GHSA Dify LiteLLM Flowise Langflow Open WebUI",
        "latest AI gateway agent framework CVE GHSA MLflow vLLM OpenRouter FastGPT",
    ];
    let mut added = 0;
    let mut discovered = 0;
    for query in queries {
        let value = s.tavily().await.search(query).await?;
        for item in value
            .get("results")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for record in cve_records_from_search_item(item) {
                discovered += 1;
                if s.repository.upsert_cve(&record).await? {
                    added += 1;
                }
            }
        }
    }
    Ok(Json(json!({
        "total": s.repository.cves().await?.len(),
        "discovered": discovered,
        "added": added,
    })))
}

fn cve_records_from_search_item(item: &Value) -> Vec<Value> {
    static ADVISORY_ID: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)\b(?:CVE-\d{4}-\d{4,7}|GHSA-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4})\b")
            .expect("advisory id regex")
    });
    let text = ["id", "cve_id", "advisory_id", "title", "content", "url"]
        .into_iter()
        .filter_map(|key| item.get(key).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let mut seen = std::collections::BTreeSet::new();
    ADVISORY_ID
        .find_iter(&text)
        .filter_map(|found| {
            let id = found.as_str().to_ascii_uppercase();
            if !seen.insert(id.clone()) {
                return None;
            }
            Some(json!({
                "id": id,
                "title": item.get("title"),
                "description": item.get("content"),
                "source_url": item.get("url"),
                "source": "tavily",
                "synced_at": chrono::Utc::now().to_rfc3339(),
            }))
        })
        .collect()
}
#[derive(Deserialize)]
struct CveAdd {
    url: Option<String>,
    id: Option<String>,
    product: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    description: Option<String>,
    cvss: Option<f64>,
    huntable: Option<String>,
}
async fn cve_add(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<CveAdd>,
) -> Result<Json<Value>, ApiError> {
    let id =
        b.id.or_else(|| {
            b.url.as_ref().and_then(|v| {
                regex::Regex::new(r"(?i)CVE-\d{4}-\d{4,7}")
                    .ok()?
                    .find(v)
                    .map(|m| m.as_str().to_uppercase())
            })
        })
        .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "bad_request", "CVE id required"))?;
    let record = json!({"id":id,"url":b.url,"product":b.product,"type":b.kind,"description":b.description,"cvss":b.cvss,"huntable":b.huntable});
    let created = s.repository.upsert_cve(&record).await?;
    Ok(Json(
        json!({"created":created,"total":s.repository.cves().await?.len(),"cve":record}),
    ))
}
async fn health() -> Json<Value> {
    Json(json!({"ok":true}))
}
async fn cves(_: Auth, State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let cves = s.repository.cves().await?;
    Ok(Json(json!({"cves":cves,"advisories":cves})))
}
#[derive(Default, Deserialize)]
struct PageQuery {
    #[serde(default)]
    q: String,
    source: Option<String>,
    enabled_only: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}
async fn honeypots(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<PageQuery>,
) -> Result<Json<Value>, ApiError> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let (rows, total) = s
        .repository
        .list_honeypots(&q.q, q.source.as_deref(), limit, offset)
        .await?;
    Ok(Json(
        json!({"results":rows,"total":total,"limit":limit,"offset":offset}),
    ))
}
async fn manual_targets(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<PageQuery>,
) -> Result<Json<Value>, ApiError> {
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let (rows, total) = s
        .repository
        .list_manual_targets(q.enabled_only.unwrap_or(false), limit, offset)
        .await?;
    Ok(Json(
        json!({"results":rows,"total":total,"limit":limit,"offset":offset}),
    ))
}
#[derive(Deserialize)]
struct HoneypotCreate {
    host: String,
    #[serde(default = "manual_reason")]
    reason: String,
    #[serde(default)]
    notes: String,
}
fn manual_reason() -> String {
    "honeypot:manual".into()
}
#[derive(Deserialize)]
struct HoneypotUpdate {
    host_key: String,
    reason: Option<String>,
    notes: Option<String>,
}
#[derive(Deserialize)]
struct HoneypotDeleteQuery {
    host_key: String,
}
#[derive(Deserialize)]
struct HoneypotBulkDelete {
    #[serde(default)]
    host_keys: Vec<String>,
}
async fn create_honeypot(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<HoneypotCreate>,
) -> Result<Json<Value>, ApiError> {
    let origin =
        aipocket_core::url_sanitize::sanitize_origin(&b.host).map_err(ApiError::internal)?;
    let key = aipocket_core::url_sanitize::host_key(&origin).map_err(ApiError::internal)?;
    Ok(Json(
        serde_json::to_value(
            s.repository
                .create_honeypot(&origin, &key, &b.reason, &b.notes)
                .await?,
        )
        .map_err(ApiError::internal)?,
    ))
}
async fn update_honeypot(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<HoneypotUpdate>,
) -> Result<Json<Value>, ApiError> {
    let row = s
        .repository
        .update_honeypot(&b.host_key, b.reason.as_deref(), b.notes.as_deref())
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "not_found", "honeypot not found"))?;
    Ok(Json(serde_json::to_value(row).map_err(ApiError::internal)?))
}
async fn delete_honeypot(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<HoneypotDeleteQuery>,
) -> Result<Json<Value>, ApiError> {
    s.repository
        .delete_honeypots(std::slice::from_ref(&q.host_key))
        .await?;
    Ok(Json(json!({"ok":true,"host_key":q.host_key})))
}
async fn bulk_delete_honeypots(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<HoneypotBulkDelete>,
) -> Result<Json<Value>, ApiError> {
    let deleted = s.repository.delete_honeypots(&b.host_keys).await?;
    Ok(Json(json!({"deleted":deleted})))
}
#[derive(Deserialize)]
struct ManualTargetsSave {
    urls: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    replace: bool,
}
#[derive(Deserialize)]
struct ManualTargetDeleteQuery {
    url: String,
}
#[derive(Deserialize)]
struct ManualTargetsDelete {
    #[serde(default)]
    urls: Vec<String>,
}
async fn save_manual_targets(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<ManualTargetsSave>,
) -> Result<Json<Value>, ApiError> {
    if b.replace {
        let (existing, _) = s.repository.list_manual_targets(false, 10_000, 0).await?;
        let urls: Vec<_> = existing.into_iter().map(|target| target.url).collect();
        s.repository.delete_manual_targets(&urls).await?;
    }
    let mut targets = Vec::new();
    let mut rejected = Vec::new();
    for raw in b
        .urls
        .lines()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        match aipocket_core::url_sanitize::sanitize_origin(raw) {
            Ok(origin) => {
                let parsed = url::Url::parse(&origin).map_err(ApiError::internal)?;
                let target = aipocket_core::ManualTarget {
                    url: origin.clone(),
                    host_key: aipocket_core::url_sanitize::host_key(&origin)
                        .map_err(ApiError::internal)?,
                    scheme: parsed.scheme().into(),
                    hostname: parsed.host_str().unwrap_or_default().into(),
                    port: parsed.port_or_known_default().unwrap_or(443),
                    enabled: true,
                    notes: b.notes.clone(),
                    ..Default::default()
                };
                targets.push(s.repository.upsert_manual_target(&target).await?);
            }
            Err(_) => rejected.push(raw.to_owned()),
        }
    }
    Ok(Json(
        json!({"added":targets.len(),"updated":0,"rejected":rejected,"targets":targets}),
    ))
}
async fn delete_manual_target(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<ManualTargetDeleteQuery>,
) -> Result<Json<Value>, ApiError> {
    let url = aipocket_core::url_sanitize::sanitize_origin(&q.url).map_err(ApiError::internal)?;
    let deleted = s.repository.delete_manual_targets(&[url]).await?;
    Ok(Json(json!({"deleted":deleted})))
}
async fn bulk_delete_manual_targets(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<ManualTargetsDelete>,
) -> Result<Json<Value>, ApiError> {
    let urls: Vec<_> = b
        .urls
        .iter()
        .filter_map(|url| aipocket_core::url_sanitize::sanitize_origin(url).ok())
        .collect();
    let deleted = s.repository.delete_manual_targets(&urls).await?;
    Ok(Json(json!({"deleted":deleted})))
}
async fn get_settings(_: Auth, State(s): State<AppState>) -> Json<SettingsView> {
    let settings = s.settings.read().await;
    Json(SettingsView::from_settings(&settings))
}
async fn update_settings(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<SettingsUpdate>,
) -> Result<Json<Value>, ApiError> {
    let updates = b.env_updates();
    persist_env(&PathBuf::from(".env"), &updates).map_err(ApiError::internal)?;
    let new = aipocket_core::Settings::load().map_err(ApiError::internal)?;
    *s.settings.write().await = new;
    let settings = s.settings.read().await;
    Ok(Json(
        json!({"updated":updates.keys().collect::<Vec<_>>(),"hot_reloaded":updates.keys().collect::<Vec<_>>(),"restart_required":[],"settings":SettingsView::from_settings(&settings)}),
    ))
}
async fn check_fofa(_: Auth, State(s): State<AppState>) -> Json<Value> {
    match s.fofa().await.check().await {
        Ok(_) => Json(json!({"status":"ok","message":"reachable","consumes_quota":true})),
        Err(e) => Json(json!({"status":"invalid","message":e.to_string(),"consumes_quota":true})),
    }
}
async fn check_shodan(_: Auth, State(s): State<AppState>) -> Json<Value> {
    let results = s.shodan().await.info_all().await;
    let keys:Vec<_>=results.iter().map(|(key,r)|match r{Ok(info)=>json!({"key_masked":mask_apikey(key),"plan":info.plan,"query_credits":info.query_credits,"alive":true}),Err(_)=>json!({"key_masked":mask_apikey(key),"plan":"","query_credits":0,"alive":false})}).collect();
    let total = keys
        .iter()
        .filter_map(|v| v.get("query_credits").and_then(Value::as_i64))
        .sum::<i64>();
    let dead = keys
        .iter()
        .filter(|v| v.get("alive") == Some(&Value::Bool(false)))
        .count();
    Json(
        json!({"keys":keys,"total_query_credits":total,"n_keys":results.len(),"n_dead":dead,"consumes_quota":false}),
    )
}
async fn check_github(_: Auth, State(s): State<AppState>) -> Json<Value> {
    let n = s.settings.read().await.github_token_list().len();
    if n == 0 {
        return Json(
            json!({"status":"disabled","message":"no tokens","core_remaining":null,"search_remaining":null,"code_search_remaining":null,"n_tokens":0}),
        );
    }
    match s.github().await.rate_limit().await {
        Ok(v) => Json(
            json!({"status":"ok","message":"reachable","core_remaining":v.pointer("/resources/core/remaining"),"search_remaining":v.pointer("/resources/search/remaining"),"code_search_remaining":v.pointer("/resources/code_search/remaining"),"n_tokens":n}),
        ),
        Err(e) => {
            let detail = e.to_string();

            let message = if detail.contains("Bad credentials") {
                format!("GitHub token 无效或已撤销。{detail}")
            } else if detail.contains("rate limit") || detail.contains("remaining=0") {
                format!("GitHub token 有效但额度已耗尽或触发限流。{detail}")
            } else if detail.contains("403") {
                format!(
                    "GitHub 拒绝了全部 token；可能是权限不足、账号风控或 token 已失效。{detail}"
                )
            } else {
                detail
            };
            Json(
                json!({"status":"invalid","message":message,"core_remaining":null,"search_remaining":null,"code_search_remaining":null,"n_tokens":n}),
            )
        }
    }
}
async fn check_tavily(_: Auth, State(s): State<AppState>) -> Json<Value> {
    match s.tavily().await.check().await {
        Ok(_) => Json(json!({"status":"ok","message":"reachable","consumes_quota":true})),
        Err(e) => Json(json!({"status":"invalid","message":e.to_string(),"consumes_quota":true})),
    }
}
#[derive(Deserialize)]
struct ScanStart {
    #[serde(default = "all_source")]
    source: String,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    mode: ScanMode,
    #[serde(default)]
    github_pack_ids: Vec<String>,
    #[serde(default)]
    manual_enrich: Vec<String>,
    #[serde(default)]
    resume_run_id: String,
}

#[cfg(test)]
mod scan_query_tests {
    use super::*;

    #[test]
    fn cve_search_items_are_normalized_before_persistence() {
        let rows = cve_records_from_search_item(&json!({
            "title":"Dify CVE-2026-12345 and GHSA-2345-6789-cfgh",
            "content":"CVE-2026-12345",
            "url":"https://example.test/advisory"
        }));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["id"], "CVE-2026-12345");
        assert_eq!(rows[1]["id"], "GHSA-2345-6789-CFGH");
    }
}
async fn scan_start(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<ScanStart>,
) -> Result<Json<ScanStatus>, ApiError> {
    if !b.resume_run_id.is_empty() {
        let Some((state, phase)) = s.repository.resumable_run(&b.resume_run_id).await? else {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                "resume run not found",
            ));
        };
        if state == "finished" || phase == "finished" {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "conflict",
                "run already finished",
            ));
        }
    }
    let sources = if b.sources.is_empty() {
        vec![b.source.clone()]
    } else {
        b.sources.clone()
    };
    let (cancel, tx, rx, stopped) = s
        .scan_manager
        .start_channel(sources.join(","), b.mode.clone())
        .await
        .map_err(|_| ApiError::new(StatusCode::CONFLICT, "conflict", "scan already running"))?;
    s.scan_manager
        .set_options(b.github_pack_ids.clone(), b.manual_enrich.clone())
        .await;
    let manager = s.scan_manager.clone();
    let repository = s.repository.clone();
    tokio::spawn(manager.consume(rx, repository, stopped));
    let scanner = s.scanner.clone();
    let settings = s.settings.read().await.clone();
    let http = s.http.clone();
    let manual_targets = if sources.iter().any(|v| v == "manual") {
        scanner.manual_targets().await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let plan = aipocket_services::assemble_sources(
        &settings,
        &http,
        &aipocket_services::AssembleParams {
            requested: sources,
            github_pack_ids: b.github_pack_ids.clone(),
            manual_enrich: b.manual_enrich.clone(),
            resume_run_id: b.resume_run_id.clone(),
            manual_targets,
        },
    );
    s.scan_manager.set_skipped(plan.skipped).await;
    tokio::spawn(async move {
        let resume = (!b.resume_run_id.is_empty()).then_some(b.resume_run_id.clone());
        if let Err(error) = scanner
            .run_resumable(plan.sources, b.mode, resume.clone(), cancel, tx.clone())
            .await
        {
            // run_id is created inside the scanner; without resume the old path used
            // "unknown" and left the real runs row stuck in `running`.
            let run_id = if let Some(run_id) = resume {
                run_id
            } else {
                scanner
                    .latest_running_run_id()
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "unknown".into())
            };
            scanner.fail_run(&run_id, error.to_string(), &tx).await;
            // Belt-and-suspenders: error paths may drop the lease before release();
            // clear Redis so the next scan is not blocked by a stale lock.
            aipocket_db::clear_stale_scan_lock(&settings).await;
        }
    });
    Ok(Json(s.scan_manager.status().await))
}
async fn scan_stop(_: Auth, State(s): State<AppState>) -> Result<Json<ScanStatus>, ApiError> {
    if !s.scan_manager.stop().await {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "no scan running",
        ));
    }
    Ok(Json(s.scan_manager.status().await))
}
async fn scan_status(_: Auth, State(s): State<AppState>) -> Json<ScanStatus> {
    Json(s.scan_manager.status().await)
}
async fn scan_logs(_: Auth, State(s): State<AppState>, Query(q): Query<Since>) -> Json<Value> {
    let lines = s.scan_manager.logs_since(q.since).await;
    let last_seq = lines.last().map(|l| l.seq).unwrap_or(q.since);
    Json(json!({"lines":lines,"last_seq":last_seq}))
}
async fn scan_stream(
    State(s): State<AppState>,
    Query(q): Query<Since>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    verify(q.token.as_deref().unwrap_or_default(), &s).await?;
    // Subscribe before replaying the buffer. Filtering by the replay high-water
    // mark closes the otherwise possible gap between replay and live subscribe.
    let receiver = s.scan_manager.subscribe();
    let replay_lines = s.scan_manager.logs_since(q.since).await;
    let replay_last = replay_lines.last().map_or(q.since, |line| line.seq);
    let replay = replay_lines.into_iter().map(|line| {
        Ok(Event::default()
            .event("log")
            .id(line.seq.to_string())
            .data(line.line))
    });
    let live = BroadcastStream::new(receiver).filter_map(move |item| async move {
        item.ok().filter(|line| line.seq > replay_last).map(|line| {
            Ok(Event::default()
                .event("log")
                .id(line.seq.to_string())
                .data(line.line))
        })
    });
    Ok(Sse::new(stream::iter(replay).chain(live))
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
async fn system_restart(_: Auth) -> Json<Value> {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(75)
    });
    Json(json!({"restarting":true}))
}

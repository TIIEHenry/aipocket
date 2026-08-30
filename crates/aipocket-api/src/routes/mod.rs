use crate::state::AppState;
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::{Value, json};

mod cve;
mod honeypot;
mod keys;
mod manual;
mod ops;
mod runs;
mod scan;
mod settings;
mod shared;
mod system;
use cve::{cve_add, cve_sync, cves};
use honeypot::{
    bulk_delete_honeypots, create_honeypot, delete_honeypot, honeypots, update_honeypot,
};
use keys::{
    all_keys, export, high_value, high_value_reveal, key_balance, key_chat, key_models, key_reveal,
    keys_balance, transition_keys,
};
use manual::{
    bulk_delete_manual_targets, delete_manual_target, manual_targets, save_manual_targets,
};
use ops::{list_audit, ready};
use runs::{delete_run, gpt_failed, retry_gpt_failed, run_log, run_results, runs};
use scan::{scan_logs, scan_start, scan_status, scan_stop, scan_stream};
use settings::{
    check_fofa, check_github, check_shodan, check_tavily, get_settings, update_settings,
};
use system::system_restart;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/ready", get(ready))
        .route("/api/audit", get(list_audit))
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
async fn health() -> Json<Value> {
    Json(json!({"ok":true}))
}

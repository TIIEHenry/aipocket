use crate::{
    auth::Auth,
    error::ApiError,
    settings::{SettingsUpdate, SettingsView, persist_env},
    state::AppState,
};
use aipocket_db::mask_apikey;
use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(crate) async fn get_settings(_: Auth, State(s): State<AppState>) -> Json<SettingsView> {
    let settings = s.settings.read().await;
    Json(SettingsView::from_settings(&settings))
}
pub(crate) async fn update_settings(
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
pub(crate) async fn check_fofa(_: Auth, State(s): State<AppState>) -> Json<Value> {
    match s.fofa().await.check().await {
        Ok(_) => Json(json!({"status":"ok","message":"reachable","consumes_quota":true})),
        Err(e) => Json(json!({"status":"invalid","message":e.to_string(),"consumes_quota":true})),
    }
}
pub(crate) async fn check_shodan(_: Auth, State(s): State<AppState>) -> Json<Value> {
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
pub(crate) async fn check_github(_: Auth, State(s): State<AppState>) -> Json<Value> {
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
pub(crate) async fn check_tavily(_: Auth, State(s): State<AppState>) -> Json<Value> {
    match s.tavily().await.check().await {
        Ok(_) => Json(json!({"status":"ok","message":"reachable","consumes_quota":true})),
        Err(e) => Json(json!({"status":"invalid","message":e.to_string(),"consumes_quota":true})),
    }
}

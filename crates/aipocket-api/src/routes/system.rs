use crate::{auth::Auth, routes::ops::record_audit, state::AppState};
use axum::{Json, extract::State, http::HeaderMap};
use serde_json::{Value, json};
use std::time::Duration;

pub(crate) async fn system_restart(
    _: Auth,
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Json<Value> {
    record_audit(&s, &headers, "restart", json!({})).await;
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(75)
    });
    Json(json!({"restarting":true}))
}

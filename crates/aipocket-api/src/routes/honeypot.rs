use crate::{auth::Auth, error::ApiError, routes::shared::PageQuery, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) async fn honeypots(
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
#[derive(Deserialize)]
pub(crate) struct HoneypotCreate {
    host: String,
    #[serde(default = "manual_reason")]
    reason: String,
    #[serde(default)]
    notes: String,
}
pub(crate) fn manual_reason() -> String {
    "honeypot:manual".into()
}
#[derive(Deserialize)]
pub(crate) struct HoneypotUpdate {
    host_key: String,
    reason: Option<String>,
    notes: Option<String>,
}
#[derive(Deserialize)]
pub(crate) struct HoneypotDeleteQuery {
    host_key: String,
}
#[derive(Deserialize)]
pub(crate) struct HoneypotBulkDelete {
    #[serde(default)]
    host_keys: Vec<String>,
}
pub(crate) async fn create_honeypot(
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
pub(crate) async fn update_honeypot(
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
pub(crate) async fn delete_honeypot(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<HoneypotDeleteQuery>,
) -> Result<Json<Value>, ApiError> {
    s.repository
        .delete_honeypots(std::slice::from_ref(&q.host_key))
        .await?;
    Ok(Json(json!({"ok":true,"host_key":q.host_key})))
}
pub(crate) async fn bulk_delete_honeypots(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<HoneypotBulkDelete>,
) -> Result<Json<Value>, ApiError> {
    let deleted = s.repository.delete_honeypots(&b.host_keys).await?;
    Ok(Json(json!({"deleted":deleted})))
}

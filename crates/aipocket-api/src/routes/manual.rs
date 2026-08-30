use crate::{auth::Auth, error::ApiError, routes::shared::PageQuery, state::AppState};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) async fn manual_targets(
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
pub(crate) struct ManualTargetsSave {
    urls: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    replace: bool,
}
#[derive(Deserialize)]
pub(crate) struct ManualTargetDeleteQuery {
    url: String,
}
#[derive(Deserialize)]
pub(crate) struct ManualTargetsDelete {
    #[serde(default)]
    urls: Vec<String>,
}
pub(crate) async fn save_manual_targets(
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
pub(crate) async fn delete_manual_target(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<ManualTargetDeleteQuery>,
) -> Result<Json<Value>, ApiError> {
    let url = aipocket_core::url_sanitize::sanitize_origin(&q.url).map_err(ApiError::internal)?;
    let deleted = s.repository.delete_manual_targets(&[url]).await?;
    Ok(Json(json!({"deleted":deleted})))
}
pub(crate) async fn bulk_delete_manual_targets(
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

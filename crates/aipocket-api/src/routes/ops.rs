use crate::{
    auth::{Auth, client_key},
    error::ApiError,
    state::AppState,
};
use aipocket_db::{AuditEvent, postgres_ready, redis_ready};
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub(crate) struct AuditQuery {
    limit: Option<u32>,
}

pub(crate) async fn ready(State(s): State<AppState>) -> Response {
    let settings = s.settings.read().await;
    let pg_enabled = settings.pg_enabled();
    let dedup_enabled = settings.dedup_enabled;
    let redis_url = settings.dedup_redis_url.clone();
    drop(settings);

    let postgres = if !pg_enabled {
        "skipped"
    } else {
        match s.repository.pool() {
            None => "error",
            Some(pool) => match postgres_ready(pool).await {
                Ok(()) => "ok",
                Err(_) => "error",
            },
        }
    };

    let redis = if !dedup_enabled {
        "skipped"
    } else {
        match redis_ready(&redis_url).await {
            Ok(()) => "ok",
            Err(_) => "error",
        }
    };

    let ok = postgres != "error";
    let degraded = redis == "error";
    let status = if postgres == "error" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    (
        status,
        Json(json!({
            "ok": ok,
            "degraded": degraded,
            "postgres": postgres,
            "redis": redis,
        })),
    )
        .into_response()
}

pub(crate) async fn list_audit(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Value>, ApiError> {
    let limit = i64::from(q.limit.unwrap_or(100).clamp(1, 500));
    Ok(Json(
        json!({"events": s.repository.list_audit(limit).await?}),
    ))
}

#[allow(dead_code)] // wired onto reveal/chat/export/restart in the next change
pub(crate) async fn record_audit(s: &AppState, headers: &HeaderMap, action: &str, detail: Value) {
    let client = client_key(headers);
    tracing::info!(action, client = %client, "audit");
    if let Err(error) = s
        .repository
        .insert_audit(&AuditEvent {
            action: action.to_owned(),
            client,
            detail,
        })
        .await
    {
        tracing::warn!(error = %error, action, "audit insert failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use aipocket_core::Settings;
    use aipocket_db::Repository;

    #[tokio::test]
    async fn record_audit_without_pool_does_not_panic() {
        let state = AppState::new(
            Settings {
                web_password: "test-password".into(),
                web_jwt_secret: "test-secret-that-is-long-enough".into(),
                ..Settings::default()
            },
            Repository::default(),
        )
        .await
        .unwrap();
        record_audit(&state, &HeaderMap::new(), "export", json!({"count": 0})).await;
    }
}

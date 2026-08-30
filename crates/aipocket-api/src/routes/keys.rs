use crate::{
    auth::Auth,
    error::ApiError,
    routes::{ops::record_audit, shared::valid_kind},
    state::AppState,
};
use aipocket_core::Credential;
use aipocket_db::mask_apikey;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub(crate) struct TransitionRequest {
    result_ids: Vec<i64>,
    status: String,
    #[serde(default)]
    note: String,
}
pub(crate) async fn transition_keys(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<TransitionRequest>,
) -> Result<Json<Value>, ApiError> {
    if !matches!(b.status.as_str(), "valid" | "suspicious" | "unavailable") {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "status must be valid, suspicious, or unavailable",
        ));
    }
    let (transitioned, skipped) = s
        .repository
        .transition_results(&b.result_ids, &b.status, &b.note)
        .await?;
    Ok(Json(json!({"transitioned":transitioned,"skipped":skipped})))
}

pub(crate) async fn high_value(
    _: Auth,
    State(s): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        json!({"results":s.repository.high_value(true).await?}),
    ))
}
#[derive(Deserialize)]
pub(crate) struct HighReveal {
    masked: String,
    apiurl: Option<String>,
}
pub(crate) async fn high_value_reveal(
    _: Auth,
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(b): Json<HighReveal>,
) -> Result<Json<Value>, ApiError> {
    for row in s.repository.high_value(false).await? {
        let key = row
            .get("apikey")
            .and_then(Value::as_str)
            .or_else(|| row.pointer("/credential/apikey").and_then(Value::as_str))
            .unwrap_or_default();
        let url = row
            .get("apiurl")
            .and_then(Value::as_str)
            .or_else(|| row.pointer("/credential/apiurl").and_then(Value::as_str))
            .unwrap_or_default();
        if mask_apikey(key) == b.masked && b.apiurl.as_deref().is_none_or(|v| v == url) {
            record_audit(
                &s,
                &headers,
                "high_value_reveal",
                json!({"masked": b.masked}),
            )
            .await;
            return Ok(Json(json!({"apikey":key,"apiurl":url})));
        }
    }
    Err(ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "key not found",
    ))
}
pub(crate) async fn all_keys(
    _: Auth,
    State(s): State<AppState>,
    Path(kind): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        json!({"kind":kind,"results":s.repository.all_records(&kind,true).await?}),
    ))
}
#[derive(Deserialize)]
pub(crate) struct KeyRef {
    apikey: String,
    #[serde(default)]
    apiurl: String,
    result_id: Option<i64>,
    #[serde(default)]
    high_value: bool,
}
pub(crate) async fn key_models(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<KeyRef>,
) -> Result<Json<Value>, ApiError> {
    if b.apikey.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "apikey required",
        ));
    }
    let probe = s
        .balance
        .probe_models(Credential {
            apikey: b.apikey,
            apiurl: b.apiurl,
            ..Default::default()
        })
        .await?;
    let expired = if probe.is_definitive_auth_rejection() {
        if let Some(result_id) = b.result_id {
            s.repository
                .mark_result_expired(
                    result_id,
                    &probe.provider,
                    probe.status_code.unwrap_or_default(),
                )
                .await?
        } else {
            false
        }
    } else {
        false
    };
    Ok(Json(json!({
        "models":probe.models,
        "status_code":probe.status_code,
        "provider":probe.provider,
        "key_state":probe.key_state,
        "error":probe.error,
        "expired":expired,
        "high_value_removed":b.high_value && expired
    })))
}
#[derive(Deserialize)]
pub(crate) struct BalanceRequest {
    apikey: String,
    #[serde(default)]
    apiurl: String,
    result_id: Option<i64>,
    #[serde(default)]
    high_value: bool,
}
pub(crate) fn definitive_expiry(probe: &aipocket_services::BalanceResult) -> Option<u16> {
    if probe.alive != Some(false) || !matches!(probe.provider.as_str(), "deepseek" | "cursor") {
        return None;
    }
    probe
        .detail
        .get("status_code")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .filter(|status| matches!(status, 401 | 403))
}

pub(crate) async fn probe_and_persist_balance(
    s: &AppState,
    credential: Credential,
    result_id: Option<i64>,
    high_value: bool,
) -> Result<Value, ApiError> {
    let r = s.balance.query(&credential).await?;
    let evidence = serde_json::to_value(&r).map_err(ApiError::internal)?;
    if let Some(status_code) = definitive_expiry(&r) {
        let expired = if let Some(result_id) = result_id {
            s.repository
                .mark_result_expired(result_id, &r.provider, status_code)
                .await?
        } else {
            false
        };
        return Ok(json!({
            "gateway":r.gateway,
            "balance_usd":"",
            "tier":"",
            "detail":evidence,
            "persisted":false,
            "result_id":result_id,
            "high_value_updated":false,
            "key_state":"expired",
            "expired":expired,
            "high_value_removed":high_value && expired
        }));
    }
    if !r.matched {
        return Ok(json!({
            "gateway":"unsupported",
            "balance_usd":"",
            "tier":"",
            "detail":evidence,
            "persisted":false,
            "result_id":result_id,
            "high_value_updated":false
        }));
    }
    let mut result = aipocket_core::ValidationResult {
        credential: credential.clone(),
        ..Default::default()
    };
    aipocket_services::apply_probe_result(&mut result, r.clone());
    let balance_display = result.balance;
    let (persisted, high_value_updated) = if result_id.is_some() || high_value {
        s.repository
            .persist_balance(aipocket_db::BalancePersistence {
                result_id,
                apikey: &credential.apikey,
                gateway: &r.gateway,
                balance: &balance_display,
                tier: &r.tier,
                detail: &evidence,
                high_value,
            })
            .await?
    } else {
        (false, false)
    };
    Ok(
        json!({"gateway":r.gateway,"balance_usd":balance_display,"tier":r.tier,"detail":evidence,"persisted":persisted,"result_id":result_id,"high_value_updated":high_value_updated}),
    )
}

pub(crate) async fn key_balance(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<BalanceRequest>,
) -> Result<Json<Value>, ApiError> {
    if b.apikey.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "apikey required",
        ));
    }
    Ok(Json(
        probe_and_persist_balance(
            &s,
            Credential {
                apikey: b.apikey,
                apiurl: b.apiurl,
                ..Default::default()
            },
            b.result_id,
            b.high_value,
        )
        .await?,
    ))
}

pub(crate) fn normalized_batch_provider(row: &Value) -> String {
    row.pointer("/provider_info/provider")
        .and_then(Value::as_str)
        .or_else(|| row.get("provider").and_then(Value::as_str))
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase()
}

pub(crate) const MAX_BATCH_BALANCE_KEYS: usize = 50;

#[derive(Deserialize)]
pub(crate) struct BatchBalanceRequest {
    result_ids: Vec<i64>,
    provider: String,
}

pub(crate) async fn keys_balance(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<BatchBalanceRequest>,
) -> Result<Json<Value>, ApiError> {
    if b.provider.trim().is_empty() || b.provider.trim().eq_ignore_ascii_case("all") {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "select one provider before batch balance testing",
        ));
    }
    if b.result_ids.is_empty() || b.result_ids.len() > MAX_BATCH_BALANCE_KEYS {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            format!("result_ids must contain 1..={MAX_BATCH_BALANCE_KEYS} records"),
        ));
    }
    let mut unique_ids = std::collections::HashSet::with_capacity(b.result_ids.len());
    if !b.result_ids.iter().all(|id| unique_ids.insert(*id)) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "result_ids must be unique",
        ));
    }
    let rows = s.repository.records_by_ids(&b.result_ids).await?;
    let missing = b.result_ids.len().saturating_sub(rows.len());
    let requested_provider = b.provider.trim().to_ascii_lowercase();
    let concurrency = s
        .settings
        .read()
        .await
        .balance_batch_concurrency
        .clamp(1, MAX_BATCH_BALANCE_KEYS);
    let mut tasks = stream::iter(rows.into_iter().enumerate().map(|(position, row)| {
        let state = s.clone();
        let requested_provider = requested_provider.clone();
        async move {
            let result_id = row
                .get("result_id")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let provider = normalized_batch_provider(&row);
            let result = if provider != requested_provider {
                json!({"result_id":result_id,"ok":false,"error":"provider mismatch"})
            } else {
                let credential = row
                    .get("credential")
                    .cloned()
                    .and_then(|value| serde_json::from_value::<Credential>(value).ok());
                let Some(credential) = credential.filter(|value| !value.apikey.is_empty()) else {
                    return (
                        position,
                        json!({"result_id":result_id,"ok":false,"error":"credential missing"}),
                    );
                };
                match probe_and_persist_balance(&state, credential, Some(result_id), false).await {
                    Ok(value) if value["key_state"] == "expired" => json!({
                        "result_id":result_id,
                        "ok":false,
                        "error":"credential expired or revoked"
                    }),
                    Ok(value) => json!({"result_id":result_id,"ok":true,"balance":value}),
                    Err(error) => json!({"result_id":result_id,"ok":false,"error":error.message}),
                }
            };
            (position, result)
        }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<_>>()
    .await;
    tasks.sort_unstable_by_key(|(position, _)| *position);
    let tasks = tasks
        .into_iter()
        .map(|(_, result)| result)
        .collect::<Vec<_>>();
    let succeeded = tasks.iter().filter(|item| item["ok"] == true).count();
    let failed = tasks.len().saturating_sub(succeeded) + missing;
    Ok(Json(json!({
        "requested":b.result_ids.len(),
        "succeeded":succeeded,
        "failed":failed,
        "results":tasks
    })))
}
#[derive(Deserialize)]
pub(crate) struct ChatRequest {
    apikey: String,
    #[serde(default)]
    apiurl: String,
    model: String,
}
pub(crate) async fn key_chat(
    _: Auth,
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(b): Json<ChatRequest>,
) -> Result<Json<Value>, ApiError> {
    if b.apikey.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "apikey required",
        ));
    }
    if b.model.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "model required (pick one from /api/key/models first)",
        ));
    }
    let masked = mask_apikey(&b.apikey);
    let result = s
        .balance
        .test_chat(
            Credential {
                apikey: b.apikey,
                apiurl: b.apiurl,
                ..Default::default()
            },
            &b.model,
        )
        .await?;
    record_audit(
        &s,
        &headers,
        "chat",
        json!({"masked": masked, "model": b.model}),
    )
    .await;
    Ok(Json(json!({
        "success":result.success,
        "status_code":result.status_code,
        "model":result.model,
        "snippet":result.snippet,
        "error":result.error,
        "consumes_credit":true
    })))
}
#[derive(Deserialize)]
pub(crate) struct RevealRequest {
    run_id: String,
    #[serde(default = "valid_kind")]
    kind: String,
    masked: Option<String>,
    apiurl: Option<String>,
    index: Option<usize>,
}
pub(crate) async fn key_reveal(
    _: Auth,
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(b): Json<RevealRequest>,
) -> Result<Json<Value>, ApiError> {
    let rows = s.repository.run_records(&b.run_id, &b.kind, false).await?;
    for (i, row) in rows.into_iter().enumerate() {
        let key = row
            .pointer("/credential/apikey")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let url = row
            .pointer("/credential/apiurl")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if b.index == Some(i)
            || (b.index.is_none()
                && b.masked.as_deref() == Some(&mask_apikey(key))
                && b.apiurl.as_deref().is_none_or(|v| v == url))
        {
            let masked = b.masked.clone().unwrap_or_else(|| mask_apikey(key));
            record_audit(
                &s,
                &headers,
                "reveal",
                json!({"masked": masked, "run_id": b.run_id, "kind": b.kind}),
            )
            .await;
            return Ok(Json(json!({"apikey":key,"apiurl":url})));
        }
    }
    Err(ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "key not found",
    ))
}
#[derive(Deserialize)]
pub(crate) struct ExportRequest {
    dataset: String,
    #[serde(default = "json_format")]
    format: String,
    run_id: Option<String>,
    #[serde(default = "valid_kind")]
    kind: String,
    #[serde(default)]
    keys: Vec<KeyRef>,
    #[serde(default)]
    indices: Vec<usize>,
}
pub(crate) fn json_format() -> String {
    "json".into()
}
pub(crate) async fn export(
    _: Auth,
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(b): Json<ExportRequest>,
) -> Result<Response, ApiError> {
    let rows = match b.dataset.as_str() {
        "selected" if b.run_id.is_some() && !b.indices.is_empty() => {
            let all = s
                .repository
                .run_records(b.run_id.as_deref().unwrap_or_default(), &b.kind, false)
                .await?;
            b.indices
                .into_iter()
                .filter_map(|index| all.get(index).cloned())
                .collect()
        }
        "selected" if !b.keys.is_empty() => b
            .keys
            .into_iter()
            .map(|k| json!({"apikey":k.apikey,"apiurl":k.apiurl}))
            .collect(),
        "selected" => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "selected export requires run_id+indices or keys",
            ));
        }
        "run" if b.run_id.is_none() => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "run export requires run_id",
            ));
        }
        "run" => {
            s.repository
                .run_records(b.run_id.as_deref().unwrap_or_default(), &b.kind, false)
                .await?
        }
        "high-value" => s.repository.high_value(false).await?,
        "all" => {
            let all = s.repository.all_records(&b.kind, false).await?;
            if b.indices.is_empty() {
                all
            } else {
                b.indices
                    .into_iter()
                    .filter_map(|index| all.get(index).cloned())
                    .collect()
            }
        }
        _ => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "unknown dataset",
            ));
        }
    };
    let (content, media, ext) = match b.format.as_str() {
        "csv" => {
            let mut w = csv::Writer::from_writer(vec![]);
            w.write_record([
                "apikey", "apiurl", "provider", "valid", "tier", "balance", "gateway",
            ])
            .map_err(ApiError::internal)?;
            for row in &rows {
                let (key, url) = export_key_url(row);
                w.write_record([
                    key,
                    url,
                    export_provider(row),
                    &row.get("valid")
                        .and_then(Value::as_bool)
                        .unwrap_or_default()
                        .to_string(),
                    row.get("tier").and_then(Value::as_str).unwrap_or_default(),
                    row.get("balance")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                    row.get("gateway")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                ])
                .map_err(ApiError::internal)?;
            }
            (
                w.into_inner().map_err(ApiError::internal)?,
                "text/csv",
                "csv",
            )
        }
        "sub2api" => (
            serde_json::to_vec_pretty(&sub2api_payload(&rows)).map_err(ApiError::internal)?,
            "application/json",
            "sub2api.json",
        ),
        "json" => (
            serde_json::to_vec_pretty(&rows).map_err(ApiError::internal)?,
            "application/json",
            "json",
        ),
        _ => {
            return Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "format must be json, csv, or sub2api",
            ));
        }
    };
    record_audit(
        &s,
        &headers,
        "export",
        json!({
            "dataset": b.dataset,
            "format": b.format,
            "count": rows.len(),
        }),
    )
    .await;
    Ok((
        [
            (header::CONTENT_TYPE, media),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"aipocket-export.{ext}\""),
            ),
        ],
        content,
    )
        .into_response())
}
pub(crate) fn export_key_url(row: &Value) -> (&str, &str) {
    let key = row
        .get("apikey")
        .and_then(Value::as_str)
        .or_else(|| row.pointer("/credential/apikey").and_then(Value::as_str))
        .unwrap_or_default();
    let url = row
        .get("apiurl")
        .and_then(Value::as_str)
        .or_else(|| row.pointer("/credential/apiurl").and_then(Value::as_str))
        .unwrap_or_default();
    (key, url)
}

pub(crate) fn export_provider(row: &Value) -> &str {
    row.pointer("/provider_info/provider")
        .and_then(Value::as_str)
        .or_else(|| row.get("provider").and_then(Value::as_str))
        .unwrap_or("openai")
}

pub(crate) fn sub2api_payload(rows: &[Value]) -> Value {
    let accounts = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let (key, url) = export_key_url(row);
            if key.is_empty() {
                return None;
            }
            let provider = export_provider(row).to_ascii_lowercase();
            let platform = match provider.as_str() {
                "anthropic" => "anthropic",
                "gemini" | "google" | "vertex" => "gemini",
                "xai" | "grok" => "grok",
                _ => "openai",
            };
            let mut credentials = serde_json::Map::new();
            credentials.insert("api_key".into(), Value::String(key.into()));
            if !url.is_empty() {
                credentials.insert(
                    "base_url".into(),
                    Value::String(url.trim_end_matches('/').trim_end_matches("/v1").into()),
                );
            }
            Some(json!({
                "name": format!("AIPocket {} {}", platform, index + 1),
                "platform": platform,
                "type": "apikey",
                "credentials": credentials,
                "concurrency": 3,
                "priority": 50,
                "rate_multiplier": 1.0
            }))
        })
        .collect::<Vec<_>>();
    json!({
        "type": "sub2api-data",
        "version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "proxies": [],
        "accounts": accounts
    })
}

#[cfg(test)]
mod key_probe_tests {
    use super::*;

    #[test]
    fn only_definitive_provider_auth_rejection_expires_a_key() {
        let probe = |provider: &str, status_code: u16, alive| aipocket_services::BalanceResult {
            provider: provider.into(),
            alive,
            detail: json!({"status_code":status_code}),
            ..Default::default()
        };
        assert_eq!(
            definitive_expiry(&probe("deepseek", 401, Some(false))),
            Some(401)
        );
        assert_eq!(
            definitive_expiry(&probe("deepseek", 403, Some(false))),
            Some(403)
        );
        assert_eq!(
            definitive_expiry(&probe("cursor", 401, Some(false))),
            Some(401)
        );
        assert_eq!(
            definitive_expiry(&probe("cursor", 403, Some(false))),
            Some(403)
        );
        assert_eq!(
            definitive_expiry(&probe("deepseek", 429, Some(false))),
            None
        );
        assert_eq!(definitive_expiry(&probe("deepseek", 401, Some(true))), None);
        assert_eq!(definitive_expiry(&probe("openai", 401, Some(false))), None);
        assert_eq!(definitive_expiry(&probe("cursor", 401, Some(true))), None);
    }

    #[test]
    fn batch_provider_normalization_is_strict_and_case_insensitive() {
        assert_eq!(
            normalized_batch_provider(&json!({"provider_info":{"provider":" OpenAI "}})),
            "openai"
        );
        assert_eq!(
            normalized_batch_provider(&json!({"provider":"ANTHROPIC"})),
            "anthropic"
        );
        assert_eq!(normalized_batch_provider(&json!({})), "unknown");
        assert_eq!(MAX_BATCH_BALANCE_KEYS, 50);
    }
}

use crate::{auth::Auth, error::ApiError, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

pub(crate) async fn delete_run(
    _: Auth,
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let deleted = s.repository.delete_run(&id).await?;
    let disk = s.settings.read().await.results_path().join(&id);
    let disk_removed = if disk.exists() {
        std::fs::remove_dir_all(disk).is_ok()
    } else {
        false
    };
    Ok(Json(
        json!({"run_id":id,"deleted":deleted,"disk_removed":disk_removed}),
    ))
}
pub(crate) async fn gpt_failed(
    _: Auth,
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_run_id(&id)?;
    let root = s.settings.read().await.results_path();
    let files = inspect_failed_files(&root, &id);
    let failed_hits = files
        .iter()
        .filter_map(|file| file.get("hits").and_then(Value::as_u64))
        .sum::<u64>();
    let retry = s.retry_manager.0.lock().await.clone();
    let retry = if retry
        .get("run_id")
        .and_then(Value::as_str)
        .is_none_or(|run| run == id)
    {
        retry
    } else {
        idle_retry()
    };
    Ok(Json(
        json!({"run_id":id,"failed_files":files.len(),"failed_hits":failed_hits,"files":files,"retry":retry}),
    ))
}
pub(crate) async fn retry_gpt_failed(
    _: Auth,
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_run_id(&id)?;
    if matches!(
        s.scan_manager.status().await.state,
        aipocket_core::ScanState::Running | aipocket_core::ScanState::Stopping
    ) {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "cannot retry while a scan is running",
        ));
    }
    let run_dir = s.settings.read().await.results_path().join(&id);
    if !run_dir.is_dir() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "run directory not found",
        ));
    }
    if inspect_failed_files(&s.settings.read().await.results_path(), &id).is_empty() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "no gpt_failed_batch_*.jsonl files to retry",
        ));
    }
    let mut status = s.retry_manager.0.lock().await;
    if status.get("state").and_then(Value::as_str) == Some("running") {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "a GPT-failed retry is already running",
        ));
    }
    let started = chrono::Utc::now().to_rfc3339();
    *status = json!({"state":"running","run_id":id,"started_at":started,"finished_at":null,"error":null,"report":null});
    let response = status.clone();
    drop(status);
    let manager = s.retry_manager.clone();
    let analyzer = aipocket_services::Analyzer::new(
        std::sync::Arc::new(s.settings.read().await.clone()),
        s.http.clone(),
    );
    let repository = s.repository.clone();
    tokio::spawn(async move {
        let outcome = analyzer.retry_failed(&id, &run_dir, &repository).await;
        let mut status = manager.0.lock().await;
        let finished = chrono::Utc::now().to_rfc3339();
        *status = match outcome {
            Ok(report) => {
                json!({"state":"finished","run_id":id,"started_at":started,"finished_at":finished,"error":null,"report":report})
            }
            Err(error) => {
                json!({"state":"error","run_id":id,"started_at":started,"finished_at":finished,"error":error.to_string(),"report":null})
            }
        };
    });
    Ok(Json(response))
}
pub(crate) fn idle_retry() -> Value {
    json!({"state":"idle","run_id":null,"started_at":null,"finished_at":null,"error":null,"report":null})
}

pub(crate) fn validate_run_id(run_id: &str) -> Result<(), ApiError> {
    let valid = regex::Regex::new(r"^run_\d{4}_\d{2}_\d{2}_\d{2}-\d{2}-\d{2}$")
        .expect("run id regex")
        .is_match(run_id);
    if valid {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "invalid run id",
        ))
    }
}

pub(crate) fn inspect_failed_files(root: &std::path::Path, run_id: &str) -> Vec<Value> {
    let dir = root.join(run_id);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if !(name.starts_with("gpt_failed") || name.starts_with("failed_batch")) {
                return None;
            }
            let (hits, batch_idx) = parse_failed_batch(&path);
            Some(json!({"name":name,"hits":hits,"batch_idx":batch_idx}))
        })
        .collect()
}
pub(crate) fn parse_failed_batch(path: &std::path::Path) -> (usize, Option<i64>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return (0, None);
    };
    let mut count = 0;
    let mut batch_idx = None;
    for (index, line) in text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
    {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if index == 0 && value.get("batch_idx").is_some() {
            batch_idx = value.get("batch_idx").and_then(Value::as_i64);
        } else if let Some(rows) = value.as_array() {
            count += rows.len();
        } else if value.is_object() {
            count += 1;
        }
    }
    (count, batch_idx)
}

pub(crate) async fn runs(_: Auth, State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({"days":s.repository.list_runs().await?})))
}
pub(crate) async fn run_results(
    _: Auth,
    State(s): State<AppState>,
    Path((id, kind)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    if !matches!(kind.as_str(), "valid" | "suspicious") {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "invalid result kind",
        ));
    }
    Ok(Json(
        json!({"run_id":id,"results":s.repository.run_records(&id,&kind,true).await?}),
    ))
}
pub(crate) async fn run_log(
    _: Auth,
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let status = s.scan_manager.status().await;
    if status.run_id.as_deref() == Some(id.as_str()) {
        let live = s.scan_manager.log_text().await;
        if !live.is_empty() {
            return Ok(
                ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], live).into_response(),
            );
        }
    }
    let log = s
        .repository
        .run_log(&id)
        .await?
        .or_else(|| {
            std::fs::read_to_string(
                s.settings
                    .blocking_read()
                    .results_path()
                    .join(&id)
                    .join("run.log"),
            )
            .ok()
        })
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "not_found", "no log for run"))?;
    Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], log).into_response())
}

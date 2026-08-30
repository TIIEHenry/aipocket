use crate::{
    auth::{Auth, verify},
    error::ApiError,
    routes::shared::{Since, all_source},
    state::AppState,
};
use aipocket_core::{ScanMode, ScanStatus};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{convert::Infallible, time::Duration};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Deserialize)]
pub(crate) struct ScanStart {
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

pub(crate) async fn scan_start(
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
pub(crate) async fn scan_stop(
    _: Auth,
    State(s): State<AppState>,
) -> Result<Json<ScanStatus>, ApiError> {
    if !s.scan_manager.stop().await {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "conflict",
            "no scan running",
        ));
    }
    Ok(Json(s.scan_manager.status().await))
}
pub(crate) async fn scan_status(_: Auth, State(s): State<AppState>) -> Json<ScanStatus> {
    Json(s.scan_manager.status().await)
}
pub(crate) async fn scan_logs(
    _: Auth,
    State(s): State<AppState>,
    Query(q): Query<Since>,
) -> Json<Value> {
    let lines = s.scan_manager.logs_since(q.since).await;
    let last_seq = lines.last().map(|l| l.seq).unwrap_or(q.since);
    Json(json!({"lines":lines,"last_seq":last_seq}))
}
pub(crate) async fn scan_stream(
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

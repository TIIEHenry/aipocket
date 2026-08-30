use crate::auth::Auth;
use axum::Json;
use serde_json::{Value, json};
use std::time::Duration;

pub(crate) async fn system_restart(_: Auth) -> Json<Value> {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(75)
    });
    Json(json!({"restarting":true}))
}

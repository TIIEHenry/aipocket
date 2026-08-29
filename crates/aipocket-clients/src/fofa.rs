use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use aipocket_core::Settings;

#[derive(Clone)]
pub struct FofaClient {
    http: Client,
    base_url: String,
    keys: Vec<String>,
    next_key: Arc<AtomicUsize>,
}
impl FofaClient {
    pub fn new(http: Client, settings: &Settings) -> Self {
        Self {
            http,
            base_url: settings.fofa_base_url.trim_end_matches('/').into(),
            keys: settings
                .fofa_key_list()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            next_key: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub async fn search(&self, query: &str, page: u32, size: u32) -> Result<Value> {
        anyhow::ensure!(!self.keys.is_empty(), "FOFA_KEYS not configured");
        let start = self.next_key.fetch_add(1, Ordering::Relaxed) % self.keys.len();
        let mut last_error = None;
        for offset in 0..self.keys.len() {
            let key = &self.keys[(start + offset) % self.keys.len()];
            match self.search_with_key(key, query, page, size).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    let message = error.to_string();
                    if fofa_quota_exhausted(&message) || fofa_wrong_endpoint(&message) {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.context("FOFA_KEYS not configured")?)
    }
    async fn search_with_key(&self, key: &str, query: &str, page: u32, size: u32) -> Result<Value> {
        let qbase64 = STANDARD.encode(query);
        let response = self
            .http
            .get(format!("{}/api/v1/search/all", self.base_url))
            .query(&[
                ("key", key),
                ("qbase64", qbase64.as_str()),
                ("page", &page.to_string()),
                ("size", &size.to_string()),
                // Keep field list aligned with Python DEFAULT_FIELDS / discovery FOFA_FIELDS.
                (
                    "fields",
                    "host,ip,port,protocol,title,header,banner,server,product,link,domain,cert",
                ),
            ])
            .send()
            .await?;
        let status = response.status();
        let bytes = response.bytes().await?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            anyhow::anyhow!(
                "fofa invalid JSON response status={status}: {}: {error}",
                crate::response_preview(&bytes)
            )
        })?;
        if let Some(message) = fofa_error_message(&value) {
            anyhow::bail!("{}", classify_fofa_message(&message));
        }
        if !status.is_success() {
            anyhow::bail!(
                "fofa response status={status}: {}",
                crate::response_preview(&bytes)
            );
        }
        Ok(value)
    }
    pub async fn check(&self) -> Result<Value> {
        self.search("title=\"123\"", 1, 1).await
    }
}

pub fn fofa_quota_exhausted(message: &str) -> bool {
    message.contains("配额已用完") || message.contains("已用完")
}

pub fn fofa_wrong_endpoint(message: &str) -> bool {
    message.contains("账号无效")
}

fn fofa_error_message(value: &Value) -> Option<String> {
    if !aipocket_core::fofa_error_flagged(value) {
        return None;
    }
    let message = value
        .get("errmsg")
        .or_else(|| value.get("message"))
        .map(|item| match item {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown FOFA error".into());
    Some(message)
}

fn classify_fofa_message(raw: &str) -> String {
    let text = raw.trim();
    if text.contains("账号无效") {
        return format!(
            "FOFA 账号无效：接口地址没改对。把 FOFA_BASE_URL 设为 https://fofoapi.com（或备用 http://107.173.248.139:18999），不要用 fofa.info。原始信息：{text}"
        );
    }
    if text.contains("key 不存在") || text.contains("key不存在") {
        return format!(
            "FOFA key 不存在：接口对了但 key 填错。检查 FOFA_KEYS 是否有空格或引号。原始信息：{text}"
        );
    }
    if text.contains("已用完") {
        return format!("FOFA 配额已用完：停止继续查询以免被封。原始信息：{text}");
    }
    format!("FOFA 查询失败：{text}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, extract::Query, routing::get};
    use serde_json::json;
    use std::collections::HashMap;

    async fn server(payload: Value) -> (String, tokio::task::JoinHandle<()>) {
        let app = Router::new().route(
            "/api/v1/search/all",
            get(move |Query(query): Query<HashMap<String, String>>| {
                let payload = payload.clone();
                async move {
                    if query.get("key").is_some_and(|key| key == "bad") {
                        return Json(json!({"error":true,"errmsg":"key 不存在"}));
                    }
                    Json(payload)
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{address}"), task)
    }

    #[test]
    fn classifies_proxy_error_messages() {
        assert!(classify_fofa_message("账号无效").contains("FOFA_BASE_URL"));
        assert!(classify_fofa_message("key 不存在").contains("FOFA_KEYS"));
        assert!(fofa_quota_exhausted(&classify_fofa_message("已用完")));
        assert!(fofa_wrong_endpoint(&classify_fofa_message("账号无效")));
        assert!(fofa_error_message(&json!({"error":"false","results":[]})).is_none());
        assert!(fofa_error_message(&json!({"error":0,"results":[]})).is_none());
        assert_eq!(
            fofa_error_message(&json!({"error":"true","errmsg":"key 不存在"})).as_deref(),
            Some("key 不存在")
        );
    }

    #[tokio::test]
    async fn search_surfaces_fofa_json_error_on_http_200() {
        let (base, task) = server(json!({"error":true,"errmsg":"账号无效"})).await;
        let client = FofaClient::new(
            reqwest::Client::new(),
            &Settings {
                fofa_keys: "ok".into(),
                fofa_base_url: base,
                ..Settings::default()
            },
        );
        let error = client.search("title=\"123\"", 1, 1).await.unwrap_err();
        assert!(error.to_string().contains("账号无效"));
        task.abort();
    }

    #[tokio::test]
    async fn search_rotates_to_next_key_after_missing_key() {
        let (base, task) = server(json!({"error":false,"results":[{"host":"ok"}]})).await;
        let client = FofaClient::new(
            reqwest::Client::new(),
            &Settings {
                fofa_keys: "bad,good".into(),
                fofa_base_url: base,
                ..Settings::default()
            },
        );
        let value = client.search("title=\"123\"", 1, 1).await.unwrap();
        assert_eq!(value["results"][0]["host"], "ok");
        task.abort();
    }
}

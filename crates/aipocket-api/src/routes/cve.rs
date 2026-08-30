use crate::{auth::Auth, error::ApiError, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) async fn cve_sync(_: Auth, State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let queries = [
        "latest AI infrastructure security CVE GHSA Dify LiteLLM Flowise Langflow Open WebUI",
        "latest AI gateway agent framework CVE GHSA MLflow vLLM OpenRouter FastGPT",
    ];
    let mut added = 0;
    let mut discovered = 0;
    for query in queries {
        let value = s.tavily().await.search(query).await?;
        for item in value
            .get("results")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for record in cve_records_from_search_item(item) {
                discovered += 1;
                if s.repository.upsert_cve(&record).await? {
                    added += 1;
                }
            }
        }
    }
    Ok(Json(json!({
        "total": s.repository.cves().await?.len(),
        "discovered": discovered,
        "added": added,
    })))
}

pub(crate) fn cve_records_from_search_item(item: &Value) -> Vec<Value> {
    static ADVISORY_ID: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)\b(?:CVE-\d{4}-\d{4,7}|GHSA-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4})\b")
            .expect("advisory id regex")
    });
    let text = ["id", "cve_id", "advisory_id", "title", "content", "url"]
        .into_iter()
        .filter_map(|key| item.get(key).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    let mut seen = std::collections::BTreeSet::new();
    ADVISORY_ID
        .find_iter(&text)
        .filter_map(|found| {
            let id = found.as_str().to_ascii_uppercase();
            if !seen.insert(id.clone()) {
                return None;
            }
            Some(json!({
                "id": id,
                "title": item.get("title"),
                "description": item.get("content"),
                "source_url": item.get("url"),
                "source": "tavily",
                "synced_at": chrono::Utc::now().to_rfc3339(),
            }))
        })
        .collect()
}
#[derive(Deserialize)]
pub(crate) struct CveAdd {
    url: Option<String>,
    id: Option<String>,
    product: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    description: Option<String>,
    cvss: Option<f64>,
    huntable: Option<String>,
}
pub(crate) async fn cve_add(
    _: Auth,
    State(s): State<AppState>,
    Json(b): Json<CveAdd>,
) -> Result<Json<Value>, ApiError> {
    let id =
        b.id.or_else(|| {
            b.url.as_ref().and_then(|v| {
                regex::Regex::new(r"(?i)CVE-\d{4}-\d{4,7}")
                    .ok()?
                    .find(v)
                    .map(|m| m.as_str().to_uppercase())
            })
        })
        .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "bad_request", "CVE id required"))?;
    let record = json!({"id":id,"url":b.url,"product":b.product,"type":b.kind,"description":b.description,"cvss":b.cvss,"huntable":b.huntable});
    let created = s.repository.upsert_cve(&record).await?;
    Ok(Json(
        json!({"created":created,"total":s.repository.cves().await?.len(),"cve":record}),
    ))
}

pub(crate) async fn cves(_: Auth, State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    let cves = s.repository.cves().await?;
    Ok(Json(json!({"cves":cves,"advisories":cves})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cve_search_items_are_normalized_before_persistence() {
        let rows = cve_records_from_search_item(&json!({
            "title":"Dify CVE-2026-12345 and GHSA-2345-6789-cfgh",
            "content":"CVE-2026-12345",
            "url":"https://example.test/advisory"
        }));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["id"], "CVE-2026-12345");
        assert_eq!(rows[1]["id"], "GHSA-2345-6789-CFGH");
    }
}

use aipocket_core::{Credential, ProviderInfo, ValidationResult};
use anyhow::Result;
use base64::Engine;
use regex::Regex;
use reqwest::Client;
use serde_json::{Value, json};
use std::sync::LazyLock;

pub const CLASH_USER_AGENT: &str = "clash.meta";

static URI_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(ss|ssr|vmess|trojan|hysteria2|tuic|vless)://").expect("uri line")
});

static YAML_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*-\s*name:\s*(.+)$").expect("yaml name"));

pub async fn validate_subscription(
    http: &Client,
    credential: &Credential,
    provider_info: ProviderInfo,
) -> Result<ValidationResult> {
    let mut result = ValidationResult {
        credential: credential.clone(),
        provider_info,
        credential_kind: aipocket_core::CREDENTIAL_KIND_PROXY_SUB.into(),
        validated_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let url = credential.apiurl.trim();
    if url.is_empty() {
        result.validation_state = "rejected".into();
        result.error = "empty subscription url".into();
        return Ok(result);
    }
    let response = http
        .get(url)
        .header("User-Agent", CLASH_USER_AGENT)
        .send()
        .await?;
    result.status_code = Some(response.status().as_u16());
    if !response.status().is_success() {
        result.valid = false;
        result.validation_state = if response.status().is_server_error() {
            "transient".into()
        } else {
            "rejected".into()
        };
        result.error = format!("subscription http {}", response.status());
        return Ok(result);
    }
    let bytes = response.bytes().await?;
    if bytes.len() > 2 * 1024 * 1024 {
        result.valid = false;
        result.validation_state = "rejected".into();
        result.error = "subscription body too large".into();
        return Ok(result);
    }
    let text = decode_body(&bytes);
    let (node_count, tags, regions, protocols) = analyze_body(&text);
    if node_count == 0 {
        result.valid = false;
        result.validation_state = "rejected".into();
        result.error = "no nodes parsed".into();
        return Ok(result);
    }
    result.valid = true;
    result.validation_state = "final_verified".into();
    result.provider_evidence = json!({
        "subscription_valid": true,
        "panel": credential.product.trim_start_matches("proxy_"),
        "node_count": node_count,
        "protocols": protocols,
        "tags": tags,
        "regions": regions,
        "airport_tier": airport_tier(&tags),
    });
    Ok(result)
}

pub async fn fetch_subscription_body(http: &Client, url: &str) -> Result<String> {
    let response = http
        .get(url)
        .header("User-Agent", CLASH_USER_AGENT)
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("subscription http {}", response.status());
    }
    let bytes = response.bytes().await?;
    if bytes.len() > 2 * 1024 * 1024 {
        anyhow::bail!("subscription body too large");
    }
    Ok(decode_body(&bytes))
}

fn decode_body(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(text.trim())
            && let Ok(inner) = String::from_utf8(decoded)
            && (inner.contains("proxies:") || URI_LINE.is_match(&inner))
        {
            return inner;
        }
        return text.to_string();
    }
    String::new()
}

fn analyze_body(text: &str) -> (usize, Vec<String>, Vec<String>, Value) {
    let mut names = Vec::new();
    for cap in YAML_NAME.captures_iter(text) {
        if let Some(name) = cap.get(1) {
            names.push(name.as_str().trim().trim_matches('"').to_string());
        }
    }
    let uri_count = URI_LINE.find_iter(text).count();
    let node_count = names.len().max(uri_count);
    let mut tags = std::collections::BTreeSet::new();
    let mut regions = std::collections::BTreeSet::new();
    for name in &names {
        for tag in tag_name(name) {
            tags.insert(tag);
        }
        for region in region_codes(name) {
            regions.insert(region);
        }
    }
    let mut protocols = serde_json::Map::new();
    for prefix in ["ss", "ssr", "vmess", "trojan", "hysteria2", "tuic", "vless"] {
        let count = text.matches(&format!("{prefix}://")).count();
        if count > 0 {
            protocols.insert(prefix.into(), json!(count));
        }
    }
    (
        node_count,
        tags.into_iter().map(str::to_owned).collect(),
        regions.into_iter().map(str::to_owned).collect(),
        Value::Object(protocols),
    )
}

fn tag_name(name: &str) -> Vec<&'static str> {
    let lower = name.to_ascii_lowercase();
    let mut out = Vec::new();
    if ["家宽", "住宅", "宽带", "住宅ip"]
        .iter()
        .any(|m| lower.contains(m))
    {
        out.push("residential");
    }
    if lower.contains("iepl") {
        out.push("iepl");
    }
    if lower.contains("iplc") {
        out.push("iplc");
    }
    if ["中转", "转发"].iter().any(|m| lower.contains(m)) {
        out.push("relay");
    }
    if lower.contains("bgp") {
        out.push("bgp");
    }
    out
}

fn region_codes(name: &str) -> Vec<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("回国") {
        return vec![];
    }
    let mut out = Vec::new();
    if ["香港", " hk", "hk-"].iter().any(|m| lower.contains(m)) {
        out.push("hk");
    }
    if lower.contains("台湾") || lower.contains(" tw") {
        out.push("tw");
    }
    if ["日本", "东京", "大阪", " jp"]
        .iter()
        .any(|m| lower.contains(m))
    {
        out.push("jp");
    }
    if lower.contains("新加坡") || lower.contains(" sg") {
        out.push("sg");
    }
    if ["美国", "硅谷", "洛杉矶"].iter().any(|m| lower.contains(m)) || lower.contains(" us ")
    {
        out.push("us");
    }
    out
}

fn airport_tier(tags: &[String]) -> &'static str {
    let has_premium = tags.iter().any(|t| t == "iepl" || t == "iplc");
    let has_res = tags.iter().any(|t| t == "residential");
    if has_premium && has_res {
        "premium_residential"
    } else if has_premium {
        "premium"
    } else if tags.iter().any(|t| t == "relay" || t == "bgp") {
        "transit"
    } else {
        "standard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_yaml_names() {
        let body = "proxies:\n  - name: 香港-IEPL-家宽-01\n    type: ss\n  - name: 日本-中转\n";
        let (count, tags, regions, _) = analyze_body(body);
        assert_eq!(count, 2);
        assert!(tags.contains(&"iepl".to_string()));
        assert!(tags.contains(&"residential".to_string()));
        assert!(regions.contains(&"hk".to_string()));
    }
}

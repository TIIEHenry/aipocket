use crate::Credential;
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;
use url::Url;

pub const FOFA_OFFICIAL_API_URL: &str = "https://fofoapi.com";
pub const SHODAN_OFFICIAL_API_URL: &str = "https://api.shodan.io";

static FOFA_CONTEXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?:FOFA|FOFO)[_-]?(?:API)?[_-]?(?:KEY|TOKEN)["']?\s*[:=]\s*['"]?([A-Za-z0-9]{32})\b"#,
    )
    .expect("fofa recon regex")
});
static SHODAN_CONTEXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)SHODAN[_-]?(?:API)?[_-]?(?:KEY|TOKEN)["']?\s*[:=]\s*['"]?([A-Za-z0-9]{32,40})\b"#,
    )
    .expect("shodan recon regex")
});

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconKey<'a> {
    pub provider: &'static str,
    pub apikey: &'a str,
    pub official_api_url: &'static str,
}

pub fn is_recon_provider(name: &str) -> bool {
    matches!(name, "fofa" | "shodan" | "fofa_leak" | "shodan_leak")
}

pub fn recon_product(name: &str) -> Option<&'static str> {
    match name {
        "fofa" | "fofa_leak" => Some("fofa"),
        "shodan" | "shodan_leak" => Some("shodan"),
        _ => None,
    }
}

pub fn recon_official_apiurl(name: &str) -> Option<&'static str> {
    match recon_product(name)? {
        "fofa" => Some(FOFA_OFFICIAL_API_URL),
        "shodan" => Some(SHODAN_OFFICIAL_API_URL),
        _ => None,
    }
}

pub fn recon_host_provider(value: &str) -> Option<&'static str> {
    let host = Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .or_else(|| {
            Url::parse(&format!("https://{value}"))
                .ok()
                .and_then(|url| url.host_str().map(str::to_owned))
        })?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "fofoapi.com"
        || host.ends_with(".fofoapi.com")
        || host == "fofa.info"
        || host.ends_with(".fofa.info")
    {
        Some("fofa")
    } else if host == "shodan.io" || host.ends_with(".shodan.io") {
        Some("shodan")
    } else {
        None
    }
}

pub fn is_recon_credential(credential: &Credential) -> bool {
    recon_product(&credential.product).is_some()
        || recon_host_provider(&credential.apiurl).is_some()
}

pub fn fofa_error_flagged(body: &Value) -> bool {
    match body.get("error") {
        Some(Value::Bool(true)) => true,
        Some(Value::String(flag)) if flag == "true" || flag == "1" => true,
        Some(Value::Number(number)) if number.as_u64() == Some(1) => true,
        _ => false,
    }
}

pub fn fofa_account_evidence(body: &Value) -> bool {
    body.get("email")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
        || body
            .get("username")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
        || json_number(body.get("fcoin")).is_some()
}

pub fn fofa_info_valid(body: &Value) -> bool {
    let errmsg = body
        .get("errmsg")
        .or_else(|| body.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if errmsg.contains("key") || errmsg.contains("账号") {
        return false;
    }
    !fofa_error_flagged(body) && fofa_account_evidence(body)
}

pub fn json_number(value: Option<&Value>) -> Option<f64> {
    value.and_then(|item| {
        item.as_f64()
            .or_else(|| item.as_i64().map(|n| n as f64))
            .or_else(|| item.as_u64().map(|n| n as f64))
    })
}

pub fn extract_recon_keys(text: &str) -> Vec<ReconKey<'_>> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (re, provider, official_api_url) in [
        (&*FOFA_CONTEXT_RE, "fofa", FOFA_OFFICIAL_API_URL),
        (&*SHODAN_CONTEXT_RE, "shodan", SHODAN_OFFICIAL_API_URL),
    ] {
        for caps in re.captures_iter(text) {
            let Some(apikey) = caps.get(1).map(|m| m.as_str()) else {
                continue;
            };
            if !seen.insert(apikey) {
                continue;
            }
            out.push(ReconKey {
                provider,
                apikey,
                official_api_url,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_env_and_json_but_not_bare_or_overlong_tokens() {
        let fofa = "ooigvvhdstmbnjd6zxiijxj8ij9exdd8";
        let shodan = "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456";
        let env_text = format!("FOFA_API_KEY={fofa}\nSHODAN_API_KEY={shodan}\nbare {fofa}");
        let env = extract_recon_keys(&env_text);
        assert_eq!(
            env.iter().map(|item| item.apikey).collect::<Vec<_>>(),
            vec![fofa, shodan]
        );
        let json_text = format!(r#"{{"FOFA_API_KEY": "{fofa}", "SHODAN_API_KEY": "{shodan}"}}"#);
        let json = extract_recon_keys(&json_text);
        assert_eq!(json.len(), 2);
        assert!(extract_recon_keys(&format!("FOFA_API_KEY={fofa}EXTRA")).is_empty());
        assert!(extract_recon_keys(fofa).is_empty());
    }

    #[test]
    fn recon_host_matches_official_host_not_path_or_query() {
        assert_eq!(
            recon_host_provider("https://fofoapi.com/api/v1/info/my"),
            Some("fofa")
        );
        assert_eq!(
            recon_host_provider("http://api.shodan.io:443"),
            Some("shodan")
        );
        assert_eq!(
            recon_host_provider("https://paste.example/view?next=https://fofoapi.com"),
            None
        );
        assert_eq!(
            recon_host_provider("https://paste.example/shodan.io/leak"),
            None
        );
    }

    #[test]
    fn fofa_info_accepts_proxy_error_shapes_and_numeric_fcoin() {
        use serde_json::json;
        assert!(fofa_info_valid(
            &json!({"error":false,"email":"a@b.c","fcoin":12})
        ));
        assert!(fofa_info_valid(
            &json!({"error":"false","username":"lab","fcoin":8})
        ));
        assert!(fofa_info_valid(&json!({"error":0,"fcoin":3.5})));
        assert!(!fofa_info_valid(&json!({"error":false,"fcoin":null})));
        assert!(!fofa_info_valid(&json!({"error":"true","email":"a@b.c"})));
        assert!(!fofa_info_valid(&json!({"error":1,"email":"a@b.c"})));
        assert!(!fofa_info_valid(
            &json!({"error":true,"errmsg":"key 不存在","fcoin":9})
        ));
    }
}

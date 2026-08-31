use regex::Regex;
use std::sync::LazyLock;
use url::Url;

pub const CREDENTIAL_KIND_PROXY_SUB: &str = "proxy_sub";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedSubUrl {
    pub apiurl: String,
    pub token: String,
    pub product: String,
}

static ABSOLUTE_SUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)https?://[^\s"'<>]+?(?:/api/v1/client/subscribe\?token=|/api/v1/client/sub\?token=|/link/|/sub/)([a-zA-Z0-9_-]{8,})"#,
    )
    .expect("absolute sub regex")
});

static ENV_SUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:SUBSCRIBE_URL|CLASH_SUB_URL)\s*=\s*(https?://[^\s"'<>]+)"#)
        .expect("env sub regex")
});

static RELATIVE_V2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(/api/v1/client/sub(?:scribe)?\?token=([a-fA-F0-9]{16,}))"#)
        .expect("relative v2 regex")
});

static RELATIVE_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(/link/([a-zA-Z0-9_-]{8,}))"#).expect("relative link regex")
});

static RELATIVE_SUB: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(/sub/([a-zA-Z0-9_-]{8,}))"#).expect("relative sub regex")
});

pub fn is_proxy_sub_credential(credential: &crate::Credential) -> bool {
    credential.credential_kind == CREDENTIAL_KIND_PROXY_SUB
}

pub fn mask_subscription_url(url: &str) -> String {
    if let Ok(mut parsed) = Url::parse(url) {
        if let Some((_, token)) = parsed
            .query_pairs()
            .find(|(k, _)| k.eq_ignore_ascii_case("token"))
        {
            let masked = if token.len() <= 4 {
                "****".to_string()
            } else {
                format!(
                    "{}****{}",
                    &token[..2],
                    &token[token.len().saturating_sub(2)..]
                )
            };
            let pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .map(|(k, v)| {
                    if k.eq_ignore_ascii_case("token") {
                        (k.into_owned(), masked.clone())
                    } else {
                        (k.into_owned(), v.into_owned())
                    }
                })
                .collect();
            parsed.set_query(None);
            if !pairs.is_empty() {
                let mut serializer = parsed.query_pairs_mut();
                serializer.clear();
                for (key, value) in pairs {
                    serializer.append_pair(&key, &value);
                }
            }
            return parsed.to_string();
        }
        let path = parsed.path().to_string();
        if let Some(idx) = path.to_ascii_lowercase().find("/link/") {
            let prefix = &path[..idx + "/link/".len()];
            let suffix = path[idx + "/link/".len()..].split('/').next().unwrap_or("");
            if !suffix.is_empty() {
                parsed.set_path(&format!("{prefix}****"));
                return parsed.to_string();
            }
        }
    }
    url.to_string()
}

pub fn extract_subscription_urls(
    text: &str,
    base_host: &str,
    product: &str,
) -> Vec<NormalizedSubUrl> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for cap in ABSOLUTE_SUB.captures_iter(text) {
        let full = cap.get(0).map(|m| m.as_str()).unwrap_or_default();
        if let Some(norm) = normalize_subscription_url(full, base_host, product)
            && seen.insert(norm.apiurl.clone())
        {
            out.push(norm);
        }
    }
    for cap in ENV_SUB.captures_iter(text) {
        let full = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        if let Some(norm) = normalize_subscription_url(full, base_host, product)
            && seen.insert(norm.apiurl.clone())
        {
            out.push(norm);
        }
    }
    for cap in RELATIVE_V2.captures_iter(text) {
        let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        if !relative_match_allowed(text, start) {
            continue;
        }
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        if let Some(norm) = normalize_subscription_url(path, base_host, product)
            && seen.insert(norm.apiurl.clone())
        {
            out.push(norm);
        }
    }
    for cap in RELATIVE_LINK.captures_iter(text) {
        let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        if !relative_match_allowed(text, start) {
            continue;
        }
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let prod = if product.is_empty() {
            "proxy_sspanel"
        } else {
            product
        };
        if let Some(norm) = normalize_subscription_url(path, base_host, prod)
            && seen.insert(norm.apiurl.clone())
        {
            out.push(norm);
        }
    }
    for cap in RELATIVE_SUB.captures_iter(text) {
        let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        if !relative_match_allowed(text, start) {
            continue;
        }
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let prod = if product.is_empty() {
            "proxy_marzban"
        } else {
            product
        };
        if let Some(norm) = normalize_subscription_url(path, base_host, prod)
            && seen.insert(norm.apiurl.clone())
        {
            out.push(norm);
        }
    }
    out
}

pub fn normalize_subscription_url(
    raw: &str,
    base_host: &str,
    product: &str,
) -> Option<NormalizedSubUrl> {
    let trimmed = raw.trim().trim_matches('"').trim_matches('\'');
    if trimmed.is_empty() || is_placeholder_token(trimmed) {
        return None;
    }
    let joined = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.starts_with("//") {
        format!("https:{trimmed}")
    } else if trimmed.starts_with('/') {
        join_host_path(base_host, trimmed)?
    } else {
        return None;
    };
    let mut parsed = Url::parse(&joined).ok()?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }
    parsed.set_fragment(None);
    let token = parsed
        .query_pairs()
        .find(|(k, _)| k.eq_ignore_ascii_case("token"))
        .map(|(_, v)| v.into_owned())
        .or_else(|| link_path_token(parsed.path()))
        .or_else(|| sub_path_token(parsed.path()))?;
    if token.len() < 8 || is_placeholder_token(&token) {
        return None;
    }
    let apiurl = parsed.to_string();
    let product = if product.is_empty() {
        infer_product(&apiurl)
    } else {
        product.to_string()
    };
    Some(NormalizedSubUrl {
        apiurl,
        token,
        product,
    })
}

fn join_host_path(base_host: &str, path: &str) -> Option<String> {
    let host = base_host.trim();
    if host.is_empty() {
        return None;
    }
    let base = if host.contains("://") {
        host.to_string()
    } else {
        format!("https://{host}")
    };
    let mut url = Url::parse(&base).ok()?;
    let (path_only, query) = match path.split_once('?') {
        Some((path_only, query)) => (path_only, Some(query)),
        None => (path, None),
    };
    let path_only = if path_only.starts_with('/') {
        path_only.to_string()
    } else {
        format!("/{path_only}")
    };
    url.set_path(&path_only);
    if let Some(query) = query {
        url.set_query(Some(query));
    }
    Some(url.to_string())
}

fn link_path_token(path: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase();
    let idx = lower.find("/link/")?;
    path_token_after(path, idx + "/link/".len())
}

fn sub_path_token(path: &str) -> Option<String> {
    let lower = path.to_ascii_lowercase();
    let idx = lower.find("/sub/")?;
    path_token_after(path, idx + "/sub/".len())
}

fn path_token_after(path: &str, start: usize) -> Option<String> {
    let token = path[start..]
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or_default();
    if token.len() >= 8 {
        Some(token.to_string())
    } else {
        None
    }
}

fn infer_product(apiurl: &str) -> String {
    let lower = apiurl.to_ascii_lowercase();
    if lower.contains("/link/") {
        "proxy_sspanel".into()
    } else if lower.contains("/sub/") {
        "proxy_marzban".into()
    } else if lower.contains("/client/subscribe") || lower.contains("/client/sub?") {
        "proxy_v2board".into()
    } else {
        "proxy_sub".into()
    }
}

fn relative_match_allowed(text: &str, start: usize) -> bool {
    !text
        .get(..start)
        .is_some_and(|prefix| prefix.contains("://"))
}

fn is_placeholder_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "your-token",
        "your_token",
        "xxxxxxxx",
        "changeme",
        "example",
        "token=xxx",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_absolute_v2board_url() {
        let url = "https://panel.test/api/v1/client/subscribe?token=\
                   abcdef0123456789abcdef0123456789";
        let norm = normalize_subscription_url(url, "", "proxy_v2board").unwrap();
        assert_eq!(norm.token, "abcdef0123456789abcdef0123456789");
        assert!(norm.apiurl.contains("subscribe?token="));
    }

    #[test]
    fn joins_relative_path_with_host() {
        let norm = normalize_subscription_url(
            "/api/v1/client/subscribe?token=abcdef0123456789ab",
            "panel.test",
            "proxy_v2board",
        )
        .unwrap();
        assert_eq!(
            norm.apiurl,
            "https://panel.test/api/v1/client/subscribe?token=abcdef0123456789ab"
        );
    }

    #[test]
    fn masks_token_query() {
        let masked = mask_subscription_url(
            "https://panel.test/api/v1/client/subscribe?token=abcdef0123456789ab",
        );
        assert!(masked.contains("token=ab****ab") || masked.contains("****"));
        assert!(!masked.contains("abcdef0123456789ab"));
    }

    #[test]
    fn extracts_from_env_line() {
        let text = "SUBSCRIBE_URL=https://x.test/link/abcd1234abcd1234";
        let urls = extract_subscription_urls(text, "ignored.test", "proxy_sspanel");
        assert_eq!(urls.len(), 1);
        assert!(urls[0].apiurl.contains("/link/"));
    }
}

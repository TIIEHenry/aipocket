use aipocket_core::Credential;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    Standard,
    Admin,
    ServiceAccount,
    OAuth,
    Unknown,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecializedValidation {
    pub valid: bool,
    pub status_code: Option<u16>,
    pub credential_kind: String,
    pub scope: String,
    pub tier_evidence: String,
    pub models: Vec<String>,
    pub error: String,
    pub evidence: Value,
}

pub fn classify_credential(provider: &str, key: &str) -> CredentialKind {
    match provider {
        "openai" if key.starts_with("sk-admin-") => CredentialKind::Admin,
        "openai" if key.starts_with("sk-svcacct-") => CredentialKind::ServiceAccount,
        "anthropic" if key.starts_with("sk-ant-admin") => CredentialKind::Admin,
        "anthropic" if key.starts_with("sk-ant-oat") || key.starts_with("sk-ant-sid") => {
            CredentialKind::OAuth
        }
        _ if key.is_empty() => CredentialKind::Unknown,
        _ => CredentialKind::Standard,
    }
}

pub async fn validate_specialized(
    http: &reqwest::Client,
    credential: &Credential,
    provider: &str,
) -> anyhow::Result<Option<SpecializedValidation>> {
    let kind = classify_credential(provider, &credential.apikey);
    let routed_apiurl = routed_apiurl(credential, provider);
    let base = routed_apiurl.trim_end_matches('/');
    let origin = base
        .split("/v1beta")
        .next()
        .unwrap_or(base)
        .trim_end_matches('/');
    let response = match provider {
        "anthropic" => Some(
            http.get(if base.ends_with("/v1") {
                format!("{base}/models")
            } else {
                format!("{base}/v1/models")
            })
            .header("x-api-key", &credential.apikey)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?,
        ),
        "gemini" => Some(
            http.get(format!("{origin}/v1beta/models"))
                .query(&[("key", &credential.apikey)])
                .send()
                .await?,
        ),
        "azure_openai" => Some(
            http.get(format!("{base}/openai/models?api-version=2024-10-21"))
                .header("api-key", &credential.apikey)
                .send()
                .await?,
        ),
        "openai" => Some(
            http.get(if base.ends_with("/v1") {
                format!("{base}/models")
            } else {
                format!("{base}/v1/models")
            })
            .bearer_auth(&credential.apikey)
            .send()
            .await?,
        ),
        "qoder" => Some(
            http.get(if base.ends_with("/api/v1") {
                format!("{base}/cloud/models")
            } else {
                format!("{base}/api/v1/cloud/models")
            })
            .bearer_auth(&credential.apikey)
            .send()
            .await?,
        ),
        "cursor" => Some(
            http.get(if base.ends_with("/v1") {
                format!("{base}/me")
            } else {
                format!("{base}/v1/me")
            })
            .bearer_auth(&credential.apikey)
            .send()
            .await?,
        ),
        "openrouter" => {
            let origin = base
                .split("/api")
                .next()
                .unwrap_or(base)
                .trim_end_matches('/');
            let response = http
                .get(format!("{origin}/api/v1/auth/key"))
                .bearer_auth(&credential.apikey)
                .send()
                .await?;
            if !response.status().is_success() {
                Some(response)
            } else {
                Some(
                    http.get(format!("{origin}/api/v1/models"))
                        .bearer_auth(&credential.apikey)
                        .send()
                        .await?,
                )
            }
        }
        "aws_bedrock" => Some(
            http.get(if base.ends_with("/foundation-models") {
                base.to_owned()
            } else {
                format!("{base}/foundation-models")
            })
            .bearer_auth(&credential.apikey)
            .send()
            .await?,
        ),
        "volcengine_ark" => Some(
            http.post(format!("{base}/chat/completions"))
                .bearer_auth(&credential.apikey)
                .json(&json!({
                    "model": ark_probe_model(base),
                    "messages":[{"role":"user","content":"ping"}],
                    "max_tokens": 8,
                    "stream": false,
                }))
                .send()
                .await?,
        ),
        "fofa" => Some(
            http.get(format!("{}/api/v1/info/my", base.trim_end_matches('/')))
                .query(&[("key", credential.apikey.as_str())])
                .send()
                .await?,
        ),
        "shodan" => Some(
            http.get(format!("{}/api-info", base.trim_end_matches('/')))
                .query(&[("key", credential.apikey.as_str())])
                .send()
                .await?,
        ),
        _ => None,
    };
    let Some(response) = response else {
        return Ok(None);
    };
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(json!({}));
    let models = extract_models(&body);
    let provider_evidence = valid_provider_evidence(provider, &body);
    let valid = status.is_success() && provider_evidence;
    Ok(Some(SpecializedValidation {
        valid,
        status_code: Some(status.as_u16()),
        credential_kind: format!("{kind:?}").to_ascii_lowercase(),
        scope: body
            .get("organization_id")
            .or_else(|| body.get("account_id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        tier_evidence: body
            .get("tier")
            .or_else(|| body.get("plan"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        models,
        error: if status.as_u16() == 401 || status.as_u16() == 403 {
            "unauthorized".into()
        } else if !status.is_success() {
            "read-failed".into()
        } else if !provider_evidence {
            "invalid-response-schema".into()
        } else {
            String::new()
        },
        evidence: body,
    }))
}
fn routed_apiurl(credential: &Credential, provider: &str) -> String {
    match provider {
        "cursor" if credential.apikey.starts_with("crsr_") => cursor_apiurl(&credential.apiurl),
        "openai" | "anthropic" | "gemini" | "openrouter" | "volcengine_ark"
            if provider_from_key(provider, &credential.apikey) =>
        {
            match provider {
                "openai" => "https://api.openai.com/v1",
                "anthropic" => "https://api.anthropic.com/v1",
                "gemini" => "https://generativelanguage.googleapis.com",
                "openrouter" => "https://openrouter.ai/api",
                "volcengine_ark" => "https://ark.cn-beijing.volces.com/api/v3",
                _ => unreachable!(),
            }
            .into()
        }
        "fofa" => {
            if aipocket_core::recon_host_provider(&credential.apiurl) == Some("fofa") {
                credential.apiurl.clone()
            } else {
                aipocket_core::FOFA_OFFICIAL_API_URL.into()
            }
        }
        "shodan" => {
            if aipocket_core::recon_host_provider(&credential.apiurl) == Some("shodan") {
                credential.apiurl.clone()
            } else {
                aipocket_core::SHODAN_OFFICIAL_API_URL.into()
            }
        }
        _ => credential.apiurl.clone(),
    }
}

/// Ark 没有公开的模型列表接口；Coding Plan 基座走 `ark-code-latest` 别名，
/// 常规 `/api/v3` 用一个便宜的已开通概率最高的 seed 系列模型探活。
fn ark_probe_model(base: &str) -> &'static str {
    if base.contains("/coding") {
        "ark-code-latest"
    } else {
        "doubao-seed-1-6-flash"
    }
}

fn cursor_apiurl(apiurl: &str) -> String {
    if let Ok(parsed) = Url::parse(apiurl)
        && parsed.host_str() == Some("api.cursor.com")
    {
        return apiurl.to_owned();
    }
    "https://api.cursor.com".into()
}

fn provider_from_key(provider: &str, key: &str) -> bool {
    match provider {
        "openai" => {
            key.starts_with("sk-proj-")
                || key.starts_with("sk-admin-")
                || key.starts_with("sk-svcacct-")
        }
        "anthropic" => key.starts_with("sk-ant-"),
        "gemini" => key.starts_with("AIza"),
        "openrouter" => key.starts_with("sk-or-"),
        "volcengine_ark" => key.starts_with("ark-"),
        _ => false,
    }
}

fn valid_provider_evidence(provider: &str, body: &Value) -> bool {
    match provider {
        "cursor" => {
            body.get("apiKeyName").and_then(Value::as_str).is_some()
                || body.get("userEmail").and_then(Value::as_str).is_some()
        }
        // 方舟没有 /models；探活是 chat/completions，证据是 choices/usage。
        "volcengine_ark" => {
            body.get("choices")
                .and_then(Value::as_array)
                .is_some_and(|choices| !choices.is_empty())
                || body.get("usage").is_some()
        }
        "fofa" => aipocket_core::fofa_info_valid(body),
        "shodan" => {
            body.get("plan").and_then(Value::as_str).is_some()
                || body.get("query_credits").is_some()
                || body.get("scan_credits").is_some()
        }
        _ => !extract_models(body).is_empty(),
    }
}

fn extract_models(value: &Value) -> Vec<String> {
    value
        .get("data")
        .or_else(|| value.get("models"))
        .or_else(|| value.get("modelSummaries"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.get("id")
                .or_else(|| item.get("name"))
                .or_else(|| item.get("modelId"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}
#[cfg(test)]
mod validation_tests {
    use super::*;
    use axum::{
        Json, Router,
        http::StatusCode,
        routing::{get, post},
    };

    #[tokio::test]
    async fn ark_validates_chat_completions_with_choices_evidence() {
        async fn chat() -> (StatusCode, Json<Value>) {
            (
                StatusCode::OK,
                Json(json!({
                    "id":"c-1",
                    "object":"chat.completion",
                    "choices":[{"message":{"role":"assistant","content":"pong"}}],
                    "usage":{"total_tokens":6}
                })),
            )
        }
        let app = Router::new().route("/api/v3/chat/completions", post(chat));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = validate_specialized(
            &reqwest::Client::new(),
            &Credential {
                apikey: "not-prefix-routed".into(),
                apiurl: format!("http://{address}/api/v3"),
                ..Default::default()
            },
            "volcengine_ark",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(result.valid);
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_empty());
        server.abort();
    }

    #[tokio::test]
    async fn ark_unauthorized_and_schemaless_success_are_not_valid() {
        async fn denied() -> (StatusCode, Json<Value>) {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":{"code":"InvalidApiKey"}})),
            )
        }
        let app = Router::new()
            .route("/v3/chat/completions", post(denied))
            .route(
                "/html/v3/chat/completions",
                post(|| async { (StatusCode::OK, "<html>ok</html>") }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let rejected = validate_specialized(
            &reqwest::Client::new(),
            &Credential {
                apikey: "not-prefix-routed".into(),
                apiurl: format!("http://{address}/v3"),
                ..Default::default()
            },
            "volcengine_ark",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!rejected.valid);
        assert_eq!(rejected.status_code, Some(401));
        assert_eq!(rejected.error, "unauthorized");
        let schemaless = validate_specialized(
            &reqwest::Client::new(),
            &Credential {
                apikey: "not-prefix-routed".into(),
                apiurl: format!("http://{address}/html/v3"),
                ..Default::default()
            },
            "volcengine_ark",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!schemaless.valid);
        assert_eq!(schemaless.error, "invalid-response-schema");
        server.abort();
    }

    #[tokio::test]
    async fn openrouter_requires_authenticated_key_before_models() {
        async fn denied() -> (StatusCode, Json<Value>) {
            (StatusCode::UNAUTHORIZED, Json(json!({"error":"denied"})))
        }
        let app = Router::new().route("/api/v1/auth/key", get(denied));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = validate_specialized(
            &reqwest::Client::new(),
            &Credential {
                apikey: "not-prefix-routed".into(),
                apiurl: format!("http://{address}/api"),
                ..Default::default()
            },
            "openrouter",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!result.valid);
        assert_eq!(result.status_code, Some(401));
        assert_eq!(result.error, "unauthorized");
        server.abort();
    }

    #[tokio::test]
    async fn rejects_success_without_provider_evidence() {
        let app = Router::new().route(
            "/v1/models",
            get(|| async { (StatusCode::OK, "<!doctype html><title>fallback</title>") }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let result = validate_specialized(
            &reqwest::Client::new(),
            &Credential {
                apikey: "not-prefix-routed".into(),
                apiurl: format!("http://{address}"),
                ..Default::default()
            },
            "openai",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!result.valid);
        assert_eq!(result.status_code, Some(200));
        assert_eq!(result.error, "invalid-response-schema");
        server.abort();
    }

    #[tokio::test]
    async fn cursor_me_requires_identity_evidence() {
        async fn me() -> (StatusCode, Json<Value>) {
            (
                StatusCode::OK,
                Json(json!({"apiKeyName":"scanner","userEmail":"dev@example.test"})),
            )
        }
        let app = Router::new().route("/v1/me", get(me));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve("api.cursor.com", address)
            .build()
            .unwrap();
        let result = validate_specialized(
            &http,
            &Credential {
                apikey: "crsr_abcdefghijklmnopqrstuvwxyz123456".into(),
                apiurl: format!("http://api.cursor.com:{}", address.port()),
                ..Default::default()
            },
            "cursor",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(result.valid);
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_empty());
        server.abort();
    }

    #[tokio::test]
    async fn cursor_auth_rejection_is_unauthorized() {
        async fn denied() -> (StatusCode, Json<Value>) {
            (StatusCode::UNAUTHORIZED, Json(json!({"error":"denied"})))
        }
        let app = Router::new().route("/v1/me", get(denied));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve("api.cursor.com", address)
            .build()
            .unwrap();
        let result = validate_specialized(
            &http,
            &Credential {
                apikey: "crsr_abcdefghijklmnopqrstuvwxyz123456".into(),
                apiurl: format!("http://api.cursor.com:{}", address.port()),
                ..Default::default()
            },
            "cursor",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!result.valid);
        assert_eq!(result.status_code, Some(401));
        assert_eq!(result.error, "unauthorized");
        server.abort();
    }

    #[tokio::test]
    async fn fofa_info_my_accepts_account_evidence() {
        async fn info() -> (StatusCode, Json<Value>) {
            (
                StatusCode::OK,
                Json(json!({"error":false,"email":"a@b.c","fcoin":8})),
            )
        }
        let app = Router::new().route("/api/v1/info/my", get(info));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve("fofoapi.com", address)
            .build()
            .unwrap();
        let result = validate_specialized(
            &http,
            &Credential {
                apikey: "ooigvvhdstmbnjd6zxiijxj8ij9exdd8".into(),
                apiurl: format!("http://fofoapi.com:{}", address.port()),
                product: "fofa".into(),
                ..Default::default()
            },
            "fofa",
        )
        .await
        .unwrap()
        .unwrap();
        assert!(result.valid);
        assert_eq!(result.status_code, Some(200));
        server.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_admin_and_oauth() {
        assert_eq!(
            classify_credential("openai", "sk-admin-x"),
            CredentialKind::Admin
        );
        assert_eq!(
            classify_credential("anthropic", "sk-ant-oat-x"),
            CredentialKind::OAuth
        );
    }

    #[test]
    fn cursor_keys_on_leak_hosts_route_to_official_api() {
        let leak = Credential {
            apikey: "crsr_abcdefghijklmnopqrstuvwxyz123456".into(),
            apiurl: "https://paste.example/v1".into(),
            ..Default::default()
        };
        assert_eq!(routed_apiurl(&leak, "cursor"), "https://api.cursor.com");
        let official = Credential {
            apikey: "crsr_abcdefghijklmnopqrstuvwxyz123456".into(),
            apiurl: "http://api.cursor.com:8080".into(),
            ..Default::default()
        };
        assert_eq!(
            routed_apiurl(&official, "cursor"),
            "http://api.cursor.com:8080"
        );
    }

    #[test]
    fn ark_keys_on_leak_hosts_route_to_official_v3() {
        let leak = Credential {
            apikey: "ark-cec4a1b2f3d94e5687a9b1c2".into(),
            apiurl: "https://paste.example/api/v3".into(),
            ..Default::default()
        };
        assert_eq!(
            routed_apiurl(&leak, "volcengine_ark"),
            "https://ark.cn-beijing.volces.com/api/v3"
        );
    }

    #[test]
    fn ark_probe_model_switches_on_coding_base() {
        assert_eq!(
            ark_probe_model("https://ark.cn-beijing.volces.com/api/coding"),
            "ark-code-latest"
        );
        assert_eq!(
            ark_probe_model("https://ark.cn-beijing.volces.com/api/v3"),
            "doubao-seed-1-6-flash"
        );
    }

    #[test]
    fn fofa_and_shodan_leak_hosts_route_to_official_api() {
        let fofa = Credential {
            apikey: "ooigvvhdstmbnjd6zxiijxj8ij9exdd8".into(),
            apiurl: "https://github.com/acme/app/blob/main/.env".into(),
            product: "fofa".into(),
            ..Default::default()
        };
        assert_eq!(routed_apiurl(&fofa, "fofa"), "https://fofoapi.com");
        let bait = Credential {
            apikey: "ooigvvhdstmbnjd6zxiijxj8ij9exdd8".into(),
            apiurl: "https://paste.example/view?next=https://fofoapi.com".into(),
            product: "fofa".into(),
            ..Default::default()
        };
        assert_eq!(routed_apiurl(&bait, "fofa"), "https://fofoapi.com");
        let official = Credential {
            apikey: "ooigvvhdstmbnjd6zxiijxj8ij9exdd8".into(),
            apiurl: "http://fofoapi.com:8080".into(),
            product: "fofa".into(),
            ..Default::default()
        };
        assert_eq!(routed_apiurl(&official, "fofa"), "http://fofoapi.com:8080");
        let shodan = Credential {
            apikey: "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456".into(),
            apiurl: "https://paste.example/shodan.io/leak".into(),
            product: "shodan".into(),
            ..Default::default()
        };
        assert_eq!(routed_apiurl(&shodan, "shodan"), "https://api.shodan.io");
    }

    #[test]
    fn fofa_evidence_requires_account_fields_not_null_data() {
        assert!(valid_provider_evidence(
            "fofa",
            &json!({"error":false,"email":"a@b.c","fcoin":12})
        ));
        assert!(valid_provider_evidence(
            "fofa",
            &json!({"error":"false","username":"lab","fcoin":8})
        ));
        assert!(valid_provider_evidence(
            "fofa",
            &json!({"error":0,"fcoin":3.5})
        ));
        assert!(!valid_provider_evidence(
            "fofa",
            &json!({"error":false,"fcoin":null})
        ));
        assert!(!valid_provider_evidence(
            "fofa",
            &json!({"error":"true","email":"a@b.c"})
        ));
        assert!(!valid_provider_evidence(
            "fofa",
            &json!({"error":true,"data":null,"errmsg":"busy"})
        ));
        assert!(!valid_provider_evidence("fofa", &json!({"data":null})));
        assert!(valid_provider_evidence(
            "shodan",
            &json!({"plan":"dev","query_credits":9})
        ));
    }
}

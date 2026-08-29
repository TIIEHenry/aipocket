use std::collections::BTreeMap;

/// FOFA/Shodan/GitHub 查询模板：`body="api.example.com" && body="sk-"` + 环境变量名 + 裸域名。
macro_rules! domain_sk_pack {
    (
        id: $id:literal,
        domains: [$($domain:literal),+],
        env_key: $env:literal
    ) => {
        ProviderPack {
            id: $id,
            fofa_queries: &[
                $(concat!("body=\"", $domain, "\" && body=\"sk-\""),)+
                concat!("body=\"", $env, "\""),
                $(concat!("body=\"", $domain, "\""),)+
            ],
            shodan_queries: &[
                $(concat!("http.html:", $domain, " http.html:sk-"),)+
                concat!("http.html:\"", $env, "\""),
                $(concat!("http.html:", $domain),)+
            ],
            github_terms: &[$env, $(concat!($domain, " sk-"),)+],
        }
    };
}

/// 非 `sk-` 密钥前缀的 API 域名 pack（如 Replicate `r8_`）。
macro_rules! domain_key_pack {
    (
        id: $id:literal,
        domains: [$($domain:literal),+],
        env_key: $env:literal,
        key_token: $key:literal
    ) => {
        ProviderPack {
            id: $id,
            fofa_queries: &[
                $(concat!("body=\"", $domain, "\" && body=\"", $key, "\""),)+
                concat!("body=\"", $env, "\""),
                $(concat!("body=\"", $domain, "\""),)+
            ],
            shodan_queries: &[
                $(concat!("http.html:", $domain, " http.html:", $key),)+
                concat!("http.html:\"", $env, "\""),
                $(concat!("http.html:", $domain),)+
            ],
            github_terms: &[$env, $(concat!($domain, " ", $key),)+],
        }
    };
}

#[derive(Clone, Debug)]
pub struct ProviderPack {
    pub id: &'static str,
    pub fofa_queries: &'static [&'static str],
    pub shodan_queries: &'static [&'static str],
    pub github_terms: &'static [&'static str],
}
pub const PACKS: &[ProviderPack] = &[
    ProviderPack {
        id: "openai",
        fofa_queries: &[
            "body=\"api.openai.com\" && body=\"sk-\"",
            "body=\"OPENAI_API_KEY\" && body=\"sk-\"",
            "body=\"sk-proj-\"",
        ],
        shodan_queries: &["http.html:sk-"],
        github_terms: &["sk- filename:.env"],
    },
    ProviderPack {
        id: "anthropic",
        fofa_queries: &["body=\"sk-ant-\""],
        shodan_queries: &["http.html:sk-ant-"],
        github_terms: &["sk-ant- filename:.env"],
    },
    ProviderPack {
        id: "gemini",
        fofa_queries: &[
            "body=\"GEMINI_API_KEY\"",
            "body=\"GOOGLE_API_KEY\" && body=\"AIza\"",
            "body=\"generativelanguage.googleapis.com\"",
        ],
        shodan_queries: &[
            "http.html:\"GEMINI_API_KEY\"",
            "http.html:\"GOOGLE_API_KEY\" http.html:AIza",
            "http.html:generativelanguage.googleapis.com",
        ],
        github_terms: &[
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY AIza",
            "generativelanguage.googleapis.com AIza",
        ],
    },
    ProviderPack {
        id: "xai",
        fofa_queries: &[
            "body=\"XAI_API_KEY\"",
            "body=\"api.x.ai\"",
            "body=\"grok-4.6\"",
            "body=\"grok-4.7\"",
        ],
        shodan_queries: &[
            "http.html:\"XAI_API_KEY\"",
            "http.html:api.x.ai",
            "http.html:\"grok-4.6\"",
            "http.html:\"grok-4.7\"",
        ],
        github_terms: &["XAI_API_KEY", "api.x.ai xai-", "grok-4.6", "grok-4.7"],
    },
    ProviderPack {
        id: "qoder",
        fofa_queries: &[
            "body=\"QODER_PAT\"",
            "body=\"QODER_PERSONAL_ACCESS_TOKEN\"",
            "body=\"api.qoder.com\"",
            "body=\"Cantus\" && body=\"Qoder\"",
        ],
        shodan_queries: &[
            "http.html:\"QODER_PAT\"",
            "http.html:\"QODER_PERSONAL_ACCESS_TOKEN\"",
            "http.html:api.qoder.com",
            "http.html:Cantus http.html:Qoder",
        ],
        github_terms: &[
            "QODER_PAT pt-",
            "QODER_PERSONAL_ACCESS_TOKEN pt-",
            "api.qoder.com",
            "Cantus Qoder",
        ],
    },
    ProviderPack {
        id: "kiro",
        fofa_queries: &["body=\"KIRO_API_KEY\"", "body=\"ksk_\" && body=\"kiro\""],
        shodan_queries: &[
            "http.html:\"KIRO_API_KEY\"",
            "http.html:ksk_ http.html:kiro",
        ],
        github_terms: &["KIRO_API_KEY", "ksk_ kiro"],
    },
    ProviderPack {
        id: "aws_bedrock",
        fofa_queries: &[
            "body=\"AWS_BEARER_TOKEN_BEDROCK\"",
            "body=\"bedrock-runtime\" && body=\"amazonaws.com\"",
        ],
        shodan_queries: &[
            "http.html:\"AWS_BEARER_TOKEN_BEDROCK\"",
            "http.html:bedrock-runtime http.html:amazonaws.com",
        ],
        github_terms: &[
            "AWS_BEARER_TOKEN_BEDROCK",
            "bedrock-runtime.amazonaws.com Authorization Bearer",
        ],
    },
    domain_key_pack! {
        id: "cursor",
        domains: ["api.cursor.com"],
        env_key: "CURSOR_API_KEY",
        key_token: "crsr_"
    },
    ProviderPack {
        id: "windsurf",
        fofa_queries: &[
            "body=\"WINDSURF_SERVICE_KEY\"",
            "body=\"CODEIUM_SERVICE_KEY\"",
            "body=\"GetTeamCreditBalance\" && body=\"service_key\"",
        ],
        shodan_queries: &[
            "http.html:\"WINDSURF_SERVICE_KEY\"",
            "http.html:\"CODEIUM_SERVICE_KEY\"",
            "http.html:GetTeamCreditBalance http.html:service_key",
        ],
        github_terms: &[
            "WINDSURF_SERVICE_KEY",
            "CODEIUM_SERVICE_KEY",
            "GetTeamCreditBalance service_key",
        ],
    },
    ProviderPack {
        id: "azure_openai",
        fofa_queries: &["body=\"openai.azure.com\""],
        shodan_queries: &["http.html:openai.azure.com"],
        github_terms: &["AZURE_OPENAI_API_KEY"],
    },
    ProviderPack {
        id: "cohere",
        fofa_queries: &[
            "body=\"api.cohere.com\" && body=\"sk-\"",
            "body=\"COHERE_API_KEY\"",
            "body=\"api.cohere.com\"",
        ],
        shodan_queries: &[
            "http.html:api.cohere.com http.html:sk-",
            "http.html:\"COHERE_API_KEY\"",
            "http.html:api.cohere.com",
        ],
        github_terms: &["COHERE_API_KEY", "api.cohere.com sk-"],
    },
    domain_sk_pack! {
        id: "deepseek",
        domains: ["api.deepseek.com"],
        env_key: "DEEPSEEK_API_KEY"
    },
    domain_sk_pack! {
        id: "fireworks",
        domains: ["api.fireworks.ai"],
        env_key: "FIREWORKS_API_KEY"
    },
    domain_sk_pack! {
        id: "glm",
        domains: ["open.bigmodel.cn"],
        env_key: "ZHIPUAI_API_KEY"
    },
    domain_sk_pack! {
        id: "kimi",
        domains: ["api.moonshot.cn", "api.moonshot.ai"],
        env_key: "MOONSHOT_API_KEY"
    },
    domain_sk_pack! {
        id: "longcat",
        domains: ["api.longcat.chat"],
        env_key: "LONGCAT_API_KEY"
    },
    domain_sk_pack! {
        id: "minimax",
        domains: ["api.minimax.io", "api.minimaxi.com"],
        env_key: "MINIMAX_API_KEY"
    },
    domain_sk_pack! {
        id: "qwen",
        domains: ["dashscope.aliyuncs.com", "dashscope-intl.aliyuncs.com"],
        env_key: "DASHSCOPE_API_KEY"
    },
    domain_key_pack! {
        id: "replicate",
        domains: ["api.replicate.com"],
        env_key: "REPLICATE_API_TOKEN",
        key_token: "r8_"
    },
    domain_sk_pack! {
        id: "together",
        domains: ["api.together.ai"],
        env_key: "TOGETHER_API_KEY"
    },
    domain_key_pack! {
        id: "volcengine_ark",
        domains: [
            "ark.cn-beijing.volces.com",
            "ark.cn-shanghai.volces.com",
            "ark.cn-guangzhou.volces.com"
        ],
        env_key: "ARK_API_KEY",
        key_token: "ark-"
    },
    ProviderPack {
        id: "fofa_leak",
        fofa_queries: &[
            "body=\"FOFA_API_KEY\"",
            "body=\"FOFA_KEY\"",
            "body=\"FOFO_API_KEY\"",
            "body=\"FOFA_TOKEN\"",
        ],
        shodan_queries: &[
            "http.html:\"FOFA_API_KEY\"",
            "http.html:\"FOFA_KEY\"",
            "http.html:\"FOFO_API_KEY\"",
        ],
        github_terms: &[
            "FOFA_API_KEY",
            "FOFA_KEY filename:.env",
            "FOFO_API_KEY",
            "FOFA_TOKEN",
        ],
    },
    ProviderPack {
        id: "shodan_leak",
        fofa_queries: &[
            "body=\"SHODAN_API_KEY\"",
            "body=\"SHODAN_KEY\"",
            "body=\"SHODAN_TOKEN\"",
        ],
        shodan_queries: &[
            "http.html:\"SHODAN_API_KEY\"",
            "http.html:\"SHODAN_KEY\"",
            "http.html:\"SHODAN_TOKEN\"",
        ],
        github_terms: &["SHODAN_API_KEY", "SHODAN_KEY filename:.env", "SHODAN_TOKEN"],
    },
];
pub fn registry() -> BTreeMap<&'static str, &'static ProviderPack> {
    PACKS.iter().map(|pack| (pack.id, pack)).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pack_ids_are_unique_and_new_families_cover_every_source() {
        assert_eq!(registry().len(), PACKS.len());
        for id in [
            "gemini",
            "xai",
            "qoder",
            "kiro",
            "aws_bedrock",
            "cursor",
            "windsurf",
            "deepseek",
            "kimi",
            "glm",
            "qwen",
            "minimax",
            "longcat",
            "volcengine_ark",
        ] {
            let pack = registry()[id];
            assert!(!pack.fofa_queries.is_empty(), "{id} FOFA queries");
            assert!(!pack.shodan_queries.is_empty(), "{id} Shodan queries");
            assert!(!pack.github_terms.is_empty(), "{id} GitHub queries");
        }
        assert!(registry()["xai"].github_terms.contains(&"grok-4.6"));
        assert!(registry()["xai"].github_terms.contains(&"grok-4.7"));
        assert!(registry()["qoder"].github_terms.contains(&"Cantus Qoder"));
        assert!(
            registry()["gemini"]
                .github_terms
                .iter()
                .any(|query| query.contains("GEMINI_API_KEY"))
        );
        assert!(
            registry()["kiro"]
                .github_terms
                .iter()
                .any(|query| query.contains("KIRO_API_KEY"))
        );
        assert!(
            registry()["aws_bedrock"]
                .github_terms
                .iter()
                .any(|query| query.contains("AWS_BEARER_TOKEN_BEDROCK"))
        );
        assert!(
            registry()["cursor"]
                .github_terms
                .iter()
                .any(|query| query.contains("CURSOR_API_KEY"))
        );
        assert!(
            registry()["cursor"]
                .fofa_queries
                .iter()
                .any(|query| query.contains("api.cursor.com") && query.contains("crsr_"))
        );
        assert!(
            registry()["windsurf"]
                .github_terms
                .iter()
                .any(|query| query.contains("WINDSURF_SERVICE_KEY"))
        );
        assert!(
            registry()["deepseek"]
                .fofa_queries
                .iter()
                .any(|query| query.contains("api.deepseek.com") && query.contains("sk-"))
        );
        assert!(
            registry()["deepseek"]
                .shodan_queries
                .iter()
                .any(|query| query.contains("api.deepseek.com") && query.contains("sk-"))
        );
        assert!(
            registry()["kimi"]
                .fofa_queries
                .iter()
                .any(|query| query.contains("api.moonshot.cn") && query.contains("sk-"))
        );
        assert!(
            registry()["qwen"]
                .fofa_queries
                .iter()
                .any(|query| query.contains("dashscope.aliyuncs.com") && query.contains("sk-"))
        );
        assert!(
            registry()["volcengine_ark"]
                .fofa_queries
                .iter()
                .any(|query| query.contains("ark.cn-beijing.volces.com") && query.contains("ark-"))
        );
        assert!(
            registry()["volcengine_ark"]
                .github_terms
                .iter()
                .any(|term| term.contains("ARK_API_KEY"))
        );
        for id in ["fofa_leak", "shodan_leak"] {
            let pack = registry()[id];
            assert!(!pack.fofa_queries.is_empty(), "{id} FOFA queries");
            assert!(!pack.shodan_queries.is_empty(), "{id} Shodan queries");
            assert!(!pack.github_terms.is_empty(), "{id} GitHub queries");
        }
        let mut ranked = crate::legacy_queries::fofa_queries();
        ranked.extend(
            registry()
                .values()
                .flat_map(|pack| pack.fofa_queries.iter().map(|query| (*query).to_string())),
        );
        crate::legacy_queries::prioritize_fofa_queries(&mut ranked);
        let fofa_pos = ranked
            .iter()
            .position(|query| query.contains("FOFA_API_KEY"))
            .expect("FOFA leak query");
        assert!(
            fofa_pos < 48,
            "incremental FOFA budget must include FOFA_API_KEY, got index {fofa_pos}"
        );
        let mut github: Vec<String> = registry()
            .values()
            .flat_map(|pack| pack.github_terms.iter().map(|term| (*term).to_string()))
            .collect();
        crate::legacy_queries::prioritize_fofa_queries(&mut github);
        let github_pos = github
            .iter()
            .position(|term| term.contains("FOFA_API_KEY"))
            .expect("FOFA leak GitHub term");
        assert!(
            github_pos < 12,
            "incremental GitHub budget must include FOFA_API_KEY, got index {github_pos}"
        );
    }
}

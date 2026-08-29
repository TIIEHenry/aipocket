pub const DIRECT_CREDENTIAL_QUERIES: &[&str] = &[
    "header=\"authorization: bearer sk-\"",
    "header=\"authorization: bearer sk-proj\"",
    "header=\"authorization: bearer sk-ant-\"",
    "header=\"x-api-key: sk-\"",
    "header=\"x-api-key: sk-ant-\"",
    "header=\"api-key: sk-\"",
    "header=\"apikey: sk-\"",
    "banner=\"authorization: bearer sk-\"",
    "banner=\"authorization: bearer sk-proj\"",
    "banner=\"authorization: bearer sk-ant-\"",
    "banner=\"OPENAI_API_KEY=sk-\"",
    "banner=\"ANTHROPIC_API_KEY=sk-ant-\"",
    "body=\"sk-proj-\"",
    "body=\"sk-ant-api\"",
    "body=\"OPENAI_API_KEY\" && body=\"sk-\"",
    "body=\"ANTHROPIC_API_KEY\" && body=\"sk-ant-\"",
    "body=\"DEEPSEEK_API_KEY\" && body=\"sk-\"",
    "body=\"api.deepseek.com\" && body=\"sk-\"",
    "body=\"api.moonshot.cn\" && body=\"sk-\"",
    "body=\"api.moonshot.ai\" && body=\"sk-\"",
    "body=\"open.bigmodel.cn\" && body=\"sk-\"",
    "body=\"dashscope.aliyuncs.com\" && body=\"sk-\"",
    "body=\"dashscope-intl.aliyuncs.com\" && body=\"sk-\"",
    "body=\"api.minimax.io\" && body=\"sk-\"",
    "body=\"api.longcat.chat\" && body=\"sk-\"",
    "body=\".env\" && body=\"sk-\"",
    "body=\"docker-compose\" && body=\"sk-\"",
    "body=\"api_key\" && body=\"sk-proj-\"",
    "body=\"moonshot\" && body=\"sk-\"",
    "body=\"deepseek\" && body=\"sk-\"",
    "body=\"master_key\" && body=\"sk-\"",
    "body=\"DANGEROUSLY_DISABLE_AUTH\" && body=\"sk-\"",
    "body=\"GEMINI_API_KEY\" && body=\"AIza\"",
    "body=\"XAI_API_KEY\" && body=\"xai-\"",
    "body=\"QODER_PAT\" && body=\"pt-\"",
    "body=\"KIRO_API_KEY\" && body=\"ksk_\"",
    "body=\"AWS_BEARER_TOKEN_BEDROCK\"",
    "header=\"authorization: bearer crsr_\"",
    "body=\"api.cursor.com\" && body=\"crsr_\"",
    "body=\"CURSOR_API_KEY\" && body=\"crsr_\"",
    "body=\"WINDSURF_SERVICE_KEY\" || body=\"CODEIUM_SERVICE_KEY\"",
];

pub const PRODUCT_QUERIES: &[(&str, &[&str])] = &[
    (
        "LiteLLM",
        &[
            "body=\"litellm\" && body=\"sk-\"",
            "body=\"litellm_proxy\" && body=\"api_key\"",
            "body=\"LiteLLM Proxy\" && body=\"master_key\"",
        ],
    ),
    (
        "Flowise",
        &[
            "body=\"Flowise\" && body=\"sk-\"",
            "body=\"flowise\" && body=\"apiKey\"",
        ],
    ),
    (
        "Dify",
        &[
            "body=\"dify\" && body=\"sk-\"",
            "body=\"dify\" && body=\"OPENAI_API_KEY\"",
            "body=\"dify\" && body=\"ANTHROPIC_API_KEY\"",
        ],
    ),
    (
        "LibreChat",
        &[
            "body=\"librechat\" && body=\"sk-\"",
            "body=\"librechat\" && body=\"OPENAI_API_KEY\"",
            "body=\"librechat\" && body=\"ANTHROPIC_API_KEY\"",
        ],
    ),
    (
        "OpenWebUI",
        &[
            "body=\"Open WebUI\" && body=\"sk-\"",
            "body=\"open-webui\" && body=\"api_key\"",
        ],
    ),
    (
        "Langflow",
        &[
            "body=\"langflow\" && body=\"sk-\"",
            "body=\"langflow\" && body=\"OPENAI_API_KEY\"",
        ],
    ),
    (
        "MLflow",
        &[
            "body=\"mlflow\" && body=\"sk-\"",
            "body=\"mlflow\" && body=\"api_key\"",
        ],
    ),
    (
        "Portkey AI Gateway",
        &[
            "body=\"portkey\" && body=\"sk-\"",
            "body=\"portkey\" && body=\"api_key\"",
        ],
    ),
    (
        "LangChain",
        &[
            "body=\"langchain\" && body=\"OPENAI_API_KEY\"",
            "body=\"langchain\" && body=\"sk-\"",
        ],
    ),
    ("PraisonAI", &["body=\"praisonai\" && body=\"sk-\""]),
    (
        "GitLab AI Gateway",
        &["body=\"ai-gateway\" && body=\"sk-\""],
    ),
    (
        "FastGPT",
        &[
            "body=\"fastgpt\" && body=\"sk-\"",
            "body=\"fastgpt\" && body=\"OPENAI_API_KEY\"",
        ],
    ),
    (
        "New-API",
        &[
            "body=\"new-api\" && body=\"sk-\"",
            "body=\"new-api\" && body=\"token\"",
        ],
    ),
    (
        "One-API",
        &[
            "body=\"one-api\" && body=\"sk-\"",
            "body=\"one-api\" && body=\"token\"",
            "body=\"oneapi\" && body=\"sk-\"",
        ],
    ),
    (
        "Sub2API",
        &[
            "body=\"sub2api\" && body=\"sk-\"",
            "body=\"Sub2API\" && body=\"api_key\"",
            "body=\"pincc.ai\" && body=\"sk-\"",
        ],
    ),
    (
        "CLIProxyAPI",
        &[
            "body=\"CLIProxyAPI\" && body=\"sk-\"",
            "body=\"CLI Proxy API\" && body=\"api-keys\"",
            "body=\"CLIProxyAPI\" && port=\"8317\"",
        ],
    ),
    (
        "AnythingLLM",
        &[
            "body=\"anythingllm\" && body=\"sk-\"",
            "body=\"anythingllm\" && body=\"OPENAI_API_KEY\"",
        ],
    ),
    (
        "ChatGPT-Next-Web",
        &[
            "body=\"nextchat\" && body=\"sk-\"",
            "body=\"chatgpt-next-web\" && body=\"OPENAI_API_KEY\"",
        ],
    ),
    (
        "OpenRouter",
        &[
            "body=\"openrouter\" && body=\"sk-or-\"",
            "body=\"openrouter\" && body=\"sk-\"",
            "body=\"OpenRouter\" && body=\"api_key\"",
        ],
    ),
    (
        "vLLM",
        &[
            "body=\"vllm\" && body=\"sk-\"",
            "body=\"vllm\" && body=\"api_key\"",
        ],
    ),
    ("Ollama", &["body=\"ollama\" && body=\"sk-\""]),
    ("LocalAI", &["body=\"localai\" && body=\"sk-\""]),
    (
        "Text-Generation-WebUI",
        &["body=\"text-generation-webui\" && body=\"sk-\""],
    ),
    (
        "LobeChat",
        &[
            "body=\"lobe-chat\" && body=\"sk-\"",
            "body=\"lobechat\" && body=\"OPENAI_API_KEY\"",
        ],
    ),
    ("Jan", &["body=\"jan.ai\" && body=\"sk-\""]),
    (
        "Claude",
        &[
            "body=\"claude\" && body=\"sk-ant-\"",
            "body=\"ANTHROPIC_API_KEY\" && body=\"sk-ant-\"",
            "body=\"anthropic\" && body=\"api_key\" && body=\"sk-\"",
        ],
    ),
    ("Codex CLI", &["body=\"codex\" && body=\"OPENAI_API_KEY\""]),
];

pub fn product_for_query(query: &str) -> Option<&'static str> {
    PRODUCT_QUERIES.iter().find_map(|(product, queries)| {
        queries
            .iter()
            .map(|value| format!("{value} && status_code=\"200\""))
            .any(|candidate| candidate == query)
            .then(|| canonical_product(product))
    })
}

pub fn shodan_product_queries() -> Vec<String> {
    let mut out = PRODUCT_QUERIES
        .iter()
        .flat_map(|(_, queries)| queries.iter())
        .map(|query| {
            query
                .replace("body=\"", "http.html:\"")
                .replace(" && ", " ")
                .replace(" || ", " OR ")
        })
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

pub fn product_for_shodan_query(query: &str) -> Option<&'static str> {
    PRODUCT_QUERIES.iter().find_map(|(product, queries)| {
        queries
            .iter()
            .map(|query| {
                query
                    .replace("body=\"", "http.html:\"")
                    .replace(" && ", " ")
                    .replace(" || ", " OR ")
            })
            .any(|candidate| candidate == query)
            .then(|| canonical_product(product))
    })
}

fn canonical_product(product: &str) -> &'static str {
    match product {
        "LiteLLM" => "litellm",
        "Flowise" => "flowise",
        "Dify" => "dify",
        "LibreChat" => "librechat",
        "OpenWebUI" => "openwebui",
        "Langflow" => "langflow",
        "MLflow" => "mlflow",
        "Portkey AI Gateway" => "portkey",
        "FastGPT" => "fastgpt",
        "New-API" => "newapi",
        "One-API" => "oneapi",
        "Sub2API" => "sub2api_panel",
        "CLIProxyAPI" => "cliproxyapi",
        "AnythingLLM" => "anythingllm",
        "ChatGPT-Next-Web" => "chatgpt_next_web",
        "OpenRouter" => "openrouter",
        "LobeChat" => "lobechat",
        _ => "generic",
    }
}

/// Higher is better. Incremental `FOFA_QUERY_BUDGET` takes from the front.
pub fn query_priority(query: &str) -> u16 {
    if query.starts_with("header=") {
        return 1000;
    }
    if query.starts_with("banner=") {
        return 900;
    }
    let lowered = query.to_ascii_lowercase();
    if [
        "fofa_api_key",
        "fofo_api_key",
        "fofa_key",
        "fofa_token",
        "shodan_api_key",
        "shodan_key",
        "shodan_token",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
    {
        return 850;
    }
    let is_relay_panel = [
        "new-api",
        "one-api",
        "oneapi",
        "litellm",
        "sub2api",
        "pincc.ai",
        "cliproxyapi",
        "cli proxy api",
        "cli-proxy-api",
    ]
    .iter()
    .any(|marker| lowered.contains(marker));
    if is_relay_panel {
        return 800;
    }
    let has_key_token = [
        "sk-", "sk-ant", "sk-proj", "xai-", "ksk_", "crsr_", "pt-", "AIza", "r8_", "nvapi-", "ark-",
    ]
    .iter()
    .any(|token| query.contains(token));
    let has_official_host = query.contains("api.")
        || query.contains("dashscope")
        || query.contains("bigmodel")
        || query.contains("openai.azure")
        || query.contains("generativelanguage")
        || query.contains("volces.com");
    if has_key_token && has_official_host {
        return 800;
    }
    if has_key_token
        && (query.contains("API_KEY")
            || query.contains("SERVICE_KEY")
            || query.contains("TOKEN")
            || query.contains("QODER_PAT"))
    {
        return 700;
    }
    if query.contains("DANGEROUSLY_DISABLE_AUTH") {
        return 650;
    }
    if has_key_token && query.contains("status_code") {
        return 400;
    }
    if has_key_token {
        return 300;
    }
    100
}

pub fn prioritize_fofa_queries(queries: &mut Vec<String>) {
    queries.sort();
    queries.dedup();
    queries.sort_by(|left, right| {
        query_priority(right)
            .cmp(&query_priority(left))
            .then_with(|| left.cmp(right))
    });
}

pub fn fofa_queries() -> Vec<String> {
    let mut out: Vec<String> = DIRECT_CREDENTIAL_QUERIES
        .iter()
        .map(|value| (*value).into())
        .collect();
    for (_, queries) in PRODUCT_QUERIES {
        out.extend(
            queries
                .iter()
                .map(|value| format!("{value} && status_code=\"200\"")),
        );
    }
    prioritize_fofa_queries(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_covers_runtime_queries() {
        assert_eq!(DIRECT_CREDENTIAL_QUERIES.len(), 41);
        assert_eq!(PRODUCT_QUERIES.len(), 27);
        assert!(fofa_queries().len() > 60);
        let ranked = fofa_queries();
        assert!(
            ranked[0].starts_with("header="),
            "incremental budget must spend on header leaks first, got {}",
            ranked[0]
        );
        let dify_pos = ranked
            .iter()
            .position(|query| query.contains("dify"))
            .expect("dify query");
        let sub2api_pos = ranked
            .iter()
            .position(|query| query.to_ascii_lowercase().contains("sub2api"))
            .expect("sub2api query");
        let cpa_pos = ranked
            .iter()
            .position(|query| query.contains("CLIProxyAPI"))
            .expect("CLIProxyAPI query");
        assert!(sub2api_pos < dify_pos);
        assert!(cpa_pos < dify_pos);
        let sk_pos = ranked
            .iter()
            .position(|query| query == "body=\"sk-\"")
            .unwrap_or(ranked.len());
        let domain_sk_pos = ranked
            .iter()
            .position(|query| query.contains("api.deepseek.com") && query.contains("sk-"))
            .expect("deepseek domain+sk query");
        assert!(domain_sk_pos < sk_pos);
        let cursor_pos = ranked
            .iter()
            .position(|query| query.contains("api.cursor.com") && query.contains("crsr_"))
            .expect("cursor domain+crsr query");
        assert!(cursor_pos < sk_pos);
        assert!(DIRECT_CREDENTIAL_QUERIES.contains(&"header=\"authorization: bearer crsr_\""));
    }
    #[test]
    fn product_query_attribution_is_stable() {
        assert_eq!(
            product_for_query("body=\"litellm\" && body=\"sk-\" && status_code=\"200\""),
            Some("litellm")
        );
        assert_eq!(product_for_query("body=\"sk-\""), None);
        assert!(
            shodan_product_queries()
                .iter()
                .any(|query| query.contains("litellm"))
        );
        assert_eq!(
            product_for_shodan_query("http.html:\"dify\" http.html:\"sk-\""),
            Some("dify")
        );
        assert_eq!(
            product_for_query("body=\"sub2api\" && body=\"sk-\" && status_code=\"200\""),
            Some("sub2api_panel")
        );
        assert_eq!(
            product_for_query("body=\"CLIProxyAPI\" && port=\"8317\" && status_code=\"200\""),
            Some("cliproxyapi")
        );
        assert_eq!(query_priority("body=\"sub2api\" && body=\"sk-\""), 800);
        assert_eq!(query_priority("body=\"new-api\" && body=\"sk-\""), 800);
        assert_eq!(query_priority(r#"body="FOFA_API_KEY""#), 850);
        assert_eq!(query_priority(r#"http.html:"SHODAN_API_KEY""#), 850);
        assert!(
            query_priority(r#"body="FOFA_API_KEY""#)
                > query_priority(r#"body="api.deepseek.com" && body="sk-""#)
        );
    }
}

use aipocket_core::recon_keys::recon_product;
use std::collections::HashMap;
use url::Url;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolFamily {
    OpenAiCompatible,
    Anthropic,
    Gemini,
    Vertex,
    AwsBedrock,
    ClashSubscription,
}
#[derive(Clone, Debug)]
pub struct ProviderSpec {
    pub name: &'static str,
    pub category: &'static str,
    pub domain_suffixes: &'static [&'static str],
    pub key_prefixes: &'static [&'static str],
    pub protocol: ProtocolFamily,
    pub models: &'static [&'static str],
    pub official_api_url: &'static str,
}
#[derive(Clone, Debug)]
pub struct ProviderResolution {
    pub spec: &'static ProviderSpec,
    pub reason: &'static str,
}
static PROXY_SUB: ProviderSpec = ProviderSpec {
    name: "proxy_sub",
    category: "proxy_sub",
    domain_suffixes: &[],
    key_prefixes: &[],
    protocol: ProtocolFamily::ClashSubscription,
    models: &[],
    official_api_url: "",
};
static UNKNOWN: ProviderSpec = ProviderSpec {
    name: "unknown",
    category: "unknown",
    domain_suffixes: &[],
    key_prefixes: &[],
    protocol: ProtocolFamily::OpenAiCompatible,
    models: &["gpt-4o-mini"],
    official_api_url: "",
};
static SPECS: &[ProviderSpec] = &[
    ProviderSpec {
        name: "openai",
        category: "international",
        domain_suffixes: &["openai.com", "oaiusercontent.com"],
        key_prefixes: &["sk-proj-", "sk-admin-", "sk-svcacct-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["gpt-4o-mini"],
        official_api_url: "https://api.openai.com/v1",
    },
    ProviderSpec {
        name: "anthropic",
        category: "international",
        domain_suffixes: &["anthropic.com"],
        key_prefixes: &["sk-ant-"],
        protocol: ProtocolFamily::Anthropic,
        models: &["claude-opus-4-8"],
        official_api_url: "https://api.anthropic.com/v1",
    },
    ProviderSpec {
        name: "deepseek",
        category: "domestic",
        domain_suffixes: &["deepseek.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["deepseek-chat"],
        official_api_url: "https://api.deepseek.com",
    },
    ProviderSpec {
        name: "kimi",
        category: "domestic",
        domain_suffixes: &["moonshot.cn", "moonshot.ai"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["moonshot-v1-8k"],
        official_api_url: "https://api.moonshot.cn/v1",
    },
    ProviderSpec {
        name: "glm",
        category: "domestic",
        domain_suffixes: &["bigmodel.cn", "zhipuai.cn", "zhipuai.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["glm-4-flash"],
        official_api_url: "https://open.bigmodel.cn/api/paas/v4",
    },
    ProviderSpec {
        name: "minimax",
        category: "domestic",
        domain_suffixes: &["minimax.io", "minimaxi.com", "minimax.chat"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["MiniMax-M2.5"],
        official_api_url: "https://api.minimax.io/v1",
    },
    ProviderSpec {
        name: "nvidia",
        category: "international",
        domain_suffixes: &["integrate.api.nvidia.com"],
        key_prefixes: &["nvapi-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["meta/llama-3.3-70b-instruct"],
        official_api_url: "https://integrate.api.nvidia.com/v1",
    },
    ProviderSpec {
        name: "ksyun",
        category: "domestic",
        domain_suffixes: &["kspmas.ksyun.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["deepseek-v3"],
        official_api_url: "https://kspmas.ksyun.com/v1",
    },
    ProviderSpec {
        name: "longcat",
        category: "domestic",
        domain_suffixes: &["api.longcat.chat"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["LongCat-2.0"],
        official_api_url: "https://api.longcat.chat/openai",
    },
    ProviderSpec {
        name: "qwen",
        category: "domestic",
        domain_suffixes: &[
            "dashscope.aliyuncs.com",
            "dashscope-intl.aliyuncs.com",
            "dashscope-us.aliyuncs.com",
        ],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["qwen-turbo"],
        official_api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    },
    ProviderSpec {
        name: "siliconflow",
        category: "gateway",
        domain_suffixes: &["siliconflow.cn"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["deepseek-ai/DeepSeek-V3"],
        official_api_url: "https://api.siliconflow.cn/v1",
    },
    ProviderSpec {
        name: "cohere",
        category: "international",
        domain_suffixes: &["cohere.com", "cohere.ai"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["command-r"],
        official_api_url: "https://api.cohere.com/v1",
    },
    ProviderSpec {
        name: "replicate",
        category: "gateway",
        domain_suffixes: &["replicate.com"],
        key_prefixes: &["r8_"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["meta/meta-llama-3.1-405b-instruct"],
        official_api_url: "https://api.replicate.com/v1",
    },
    ProviderSpec {
        name: "together",
        category: "gateway",
        domain_suffixes: &["together.xyz", "together.ai"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["meta-llama/Llama-3.3-70B-Instruct-Turbo"],
        official_api_url: "https://api.together.ai/v1",
    },
    ProviderSpec {
        name: "fireworks",
        category: "gateway",
        domain_suffixes: &["fireworks.ai"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["accounts/fireworks/models/llama-v3p3-70b-instruct"],
        official_api_url: "https://api.fireworks.ai/inference/v1",
    },
    ProviderSpec {
        name: "groq",
        category: "international",
        domain_suffixes: &["groq.com"],
        key_prefixes: &["gsk_"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["llama-3.3-70b-versatile"],
        official_api_url: "https://api.groq.com/openai/v1",
    },
    ProviderSpec {
        name: "openrouter",
        category: "gateway",
        domain_suffixes: &["openrouter.ai"],
        key_prefixes: &["sk-or-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["openai/gpt-4o-mini"],
        official_api_url: "https://openrouter.ai/api",
    },
    ProviderSpec {
        name: "xai",
        category: "international",
        domain_suffixes: &["api.x.ai", "x.ai"],
        key_prefixes: &["xai-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["grok-4.6", "grok-4.7"],
        official_api_url: "https://api.x.ai/v1",
    },
    ProviderSpec {
        name: "qoder",
        category: "coding_agent",
        domain_suffixes: &["api.qoder.com", "qoder.com"],
        key_prefixes: &["pt-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["cantus"],
        official_api_url: "https://api.qoder.com",
    },
    ProviderSpec {
        name: "kiro",
        category: "coding_agent",
        domain_suffixes: &["app.kiro.dev", "kiro.dev"],
        key_prefixes: &["ksk_"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "https://app.kiro.dev",
    },
    ProviderSpec {
        name: "aws_bedrock",
        category: "cloud",
        domain_suffixes: &[
            "bedrock.amazonaws.com",
            "bedrock-runtime.amazonaws.com",
            "bedrock-mantle.amazonaws.com",
        ],
        key_prefixes: &["ABSK"],
        protocol: ProtocolFamily::AwsBedrock,
        models: &["amazon.nova-lite-v1:0"],
        official_api_url: "https://bedrock.us-east-1.amazonaws.com",
    },
    ProviderSpec {
        name: "cursor",
        category: "coding_agent",
        domain_suffixes: &["api.cursor.com", "cursor.com"],
        key_prefixes: &["crsr_"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "https://api.cursor.com",
    },
    ProviderSpec {
        name: "windsurf",
        category: "coding_agent",
        domain_suffixes: &["server.codeium.com", "windsurf.com", "codeium.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "https://server.codeium.com/api/v1",
    },
    ProviderSpec {
        name: "gemini",
        category: "international",
        domain_suffixes: &["generativelanguage.googleapis.com"],
        key_prefixes: &["AIza"],
        protocol: ProtocolFamily::Gemini,
        models: &["gemini-2.0-flash"],
        official_api_url: "https://generativelanguage.googleapis.com",
    },
    ProviderSpec {
        name: "azure_openai",
        category: "international",
        domain_suffixes: &["openai.azure.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "",
    },
    ProviderSpec {
        name: "volcengine_ark",
        category: "domestic",
        domain_suffixes: &[],
        key_prefixes: &["ark-"],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &["doubao-seed-1-6-flash", "ark-code-latest"],
        official_api_url: "https://ark.cn-beijing.volces.com/api/v3",
    },
    ProviderSpec {
        name: "fofa",
        category: "recon",
        domain_suffixes: &["fofa.info", "fofoapi.com"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "https://fofoapi.com",
    },
    ProviderSpec {
        name: "shodan",
        category: "recon",
        domain_suffixes: &["shodan.io", "api.shodan.io"],
        key_prefixes: &[],
        protocol: ProtocolFamily::OpenAiCompatible,
        models: &[],
        official_api_url: "https://api.shodan.io",
    },
];
fn is_ark_host(host: &str) -> bool {
    host.starts_with("ark.") && (host.ends_with(".volces.com") || host.ends_with(".bytepluses.com"))
}
fn domain_matches(spec: &ProviderSpec, host: &str) -> bool {
    if spec.name == "aws_bedrock" {
        return host.ends_with(".amazonaws.com")
            && (host.starts_with("bedrock.") || host.starts_with("bedrock-"));
    }
    if spec.name == "volcengine_ark" {
        return is_ark_host(host);
    }
    spec.domain_suffixes
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
}

#[derive(Default)]
pub struct ProviderRegistry;
impl ProviderRegistry {
    pub fn resolve(&self, apiurl: &str, apikey: &str) -> ProviderResolution {
        self.resolve_with_product(apiurl, apikey, "")
    }

    pub fn resolve_with_product(
        &self,
        apiurl: &str,
        apikey: &str,
        product: &str,
    ) -> ProviderResolution {
        if is_proxy_subscription(apiurl, product) {
            return ProviderResolution {
                spec: &PROXY_SUB,
                reason: "proxy_sub_path",
            };
        }
        if let Some(name) = recon_product(product)
            && let Some(spec) = SPECS.iter().find(|spec| spec.name == name)
        {
            return ProviderResolution {
                spec,
                reason: "product",
            };
        }
        let host = Url::parse(apiurl)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_default();
        if let Some(spec) = SPECS.iter().find(|spec| domain_matches(spec, &host)) {
            return ProviderResolution {
                spec,
                reason: "domain",
            };
        }
        if let Some(spec) = SPECS.iter().find(|spec| {
            spec.key_prefixes
                .iter()
                .any(|prefix| apikey.starts_with(prefix))
        }) {
            return ProviderResolution {
                spec,
                reason: "key_prefix",
            };
        }
        ProviderResolution {
            spec: &UNKNOWN,
            reason: "unmatched",
        }
    }
    pub fn specs(&self) -> HashMap<&'static str, &'static ProviderSpec> {
        SPECS.iter().map(|spec| (spec.name, spec)).collect()
    }
}

fn is_proxy_subscription(apiurl: &str, product: &str) -> bool {
    if product.starts_with("proxy_") {
        return true;
    }
    let lower = apiurl.to_ascii_lowercase();
    lower.contains("/client/subscribe")
        || lower.contains("/client/sub?")
        || lower.contains("/link/")
        || lower.contains("/sub/")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_provider_domains_and_key_prefixes() {
        for (url, key, expected) in [
            ("", "sk-ant-test", "anthropic"),
            ("", "xai-abcdefghijklmnop", "xai"),
            ("", "pt-abcdefghijklmnop", "qoder"),
            ("", "ksk_abcdefghijklmnop", "kiro"),
            ("", "crsr_abcdefghijklmnopqrstuvwxyz123456", "cursor"),
            (
                "https://bedrock-runtime.us-east-1.amazonaws.com",
                "token",
                "aws_bedrock",
            ),
            ("https://server.codeium.com", "service-key", "windsurf"),
            ("", "ark-cec4a1b2f3d94e5687a9", "volcengine_ark"),
            (
                "https://ark.cn-shanghai.volces.com/api/v3",
                "token",
                "volcengine_ark",
            ),
        ] {
            assert_eq!(ProviderRegistry.resolve(url, key).spec.name, expected);
        }
        assert!(
            !ProviderRegistry
                .resolve("https://console.volces.com", "t")
                .spec
                .name
                .eq("volcengine_ark")
        );
        let ark = ProviderRegistry.specs()["volcengine_ark"];
        assert!(ark.models.contains(&"ark-code-latest"));
        let xai = ProviderRegistry.specs()["xai"];
        assert!(xai.models.contains(&"grok-4.6"));
        assert!(xai.models.contains(&"grok-4.7"));
        assert_eq!(ProviderRegistry.specs()["qoder"].models, &["cantus"]);
        assert_eq!(
            ProviderRegistry
                .resolve_with_product(
                    "https://github.com/acme/app/blob/main/.env",
                    "ooigvvhdstmbnjd6zxiijxj8ij9exdd8",
                    "fofa"
                )
                .spec
                .name,
            "fofa"
        );
        assert_eq!(
            ProviderRegistry
                .resolve_with_product("https://paste.example", "token", "shodan")
                .spec
                .name,
            "shodan"
        );
    }
}

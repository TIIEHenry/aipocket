use std::sync::Arc;

use aipocket_clients::{FofaClient, GithubClient, ShodanClient};
use aipocket_core::{Settings, SkippedSource};
use aipocket_discovery::{
    DiscoverySource, compose_proxy_queries, compose_queries,
    legacy_queries::prioritize_fofa_queries,
    merge_proxy_queries, packs,
    proxy_packs::{selected_proxy_packs},
    sources::{FofaSource, GithubSource, ManualEnrichSource, ManualSource, ShodanSource},
};

pub struct AssembleParams {
    pub requested: Vec<String>,
    pub github_pack_ids: Vec<String>,
    pub manual_enrich: Vec<String>,
    pub resume_run_id: String,
    pub manual_targets: Vec<String>,
}

pub struct ScanPlan {
    pub sources: Vec<Arc<dyn DiscoverySource>>,
    pub skipped: Vec<SkippedSource>,
}

pub fn assemble_sources(
    settings: &Settings,
    http: &reqwest::Client,
    params: &AssembleParams,
) -> ScanPlan {
    let wants = |name: &str| params.requested.iter().any(|v| v == "all" || v == name);
    let ai_registry = packs::registry();
    let selected_ai: Vec<_> =
        if params.github_pack_ids.is_empty() || params.github_pack_ids.iter().any(|v| v == "all") {
            ai_registry.values().copied().collect()
        } else {
            params
                .github_pack_ids
                .iter()
                .filter_map(|id| ai_registry.get(id.as_str()).copied())
                .collect()
        };
    let mut q = compose_queries(&selected_ai);
    if settings.proxy_sub_enabled {
        let proxy_selected = selected_proxy_packs(&settings.proxy_sub_extra_pack_list());
        let q_proxy = compose_proxy_queries(&proxy_selected);
        merge_proxy_queries(&mut q, q_proxy, settings.proxy_sub_query_budget);
    }

    let mut sources: Vec<Arc<dyn DiscoverySource>> = Vec::new();
    let mut skipped: Vec<SkippedSource> = Vec::new();
    let skip = |src: &str, reason: &str, skipped: &mut Vec<SkippedSource>| {
        skipped.push(SkippedSource {
            source: src.into(),
            reason: reason.into(),
        });
    };

    if wants("fofa") {
        if settings.fofa_key_list().is_empty() {
            skip("fofa", "missing FOFA_KEYS", &mut skipped);
        } else {
            sources.push(Arc::new(FofaSource {
                client: FofaClient::new(http.clone(), settings),
                queries: q.fofa.clone(),
                page_size: settings.fofa_page_size,
                max_pages: settings.fofa_max_pages,
                page_delay: settings.fofa_page_delay,
            }));
        }
    }
    if wants("shodan") {
        if settings.shodan_key_list().is_empty() {
            skip("shodan", "missing SHODAN_KEYS", &mut skipped);
        } else {
            sources.push(Arc::new(ShodanSource {
                client: ShodanClient::new(http.clone(), settings),
                queries: q.shodan.clone(),
                max_pages: settings.shodan_max_pages,
                page_delay: settings.shodan_page_delay,
            }));
        }
    }
    if wants("github") {
        if settings.github_token_list().is_empty() {
            skip("github", "missing GITHUB_TOKENS", &mut skipped);
        } else if !settings.pg_enabled() {
            skip("github", "requires DATABASE_URL", &mut skipped);
        } else {
            let mut queries = q.github.clone();
            prioritize_fofa_queries(&mut queries);
            sources.push(Arc::new(GithubSource {
                client: GithubClient::new(http.clone(), settings),
                queries,
                per_page: settings.github_search_page_size,
                run_id: params.resume_run_id.clone(),
                pack_id: if params.github_pack_ids.len() == 1 {
                    params.github_pack_ids[0].clone()
                } else {
                    String::new()
                },
            }));
        }
    }
    if params.requested.iter().any(|v| v == "manual") {
        if params.manual_targets.is_empty() {
            skip("manual", "no manual targets", &mut skipped);
        } else {
            sources.push(Arc::new(ManualSource {
                targets: params.manual_targets.clone(),
            }));
            let engines: Vec<String> = params
                .manual_enrich
                .iter()
                .map(|e| e.trim().to_ascii_lowercase())
                .filter(|e| e == "fofa" || e == "shodan")
                .collect();
            if !engines.is_empty() {
                sources.push(Arc::new(ManualEnrichSource {
                    targets: params.manual_targets.clone(),
                    engines,
                    fofa: FofaClient::new(http.clone(), settings),
                    shodan: ShodanClient::new(http.clone(), settings),
                }));
            }
        }
    }
    ScanPlan { sources, skipped }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipocket_core::Settings;

    fn http() -> reqwest::Client {
        reqwest::Client::new()
    }

    fn params(requested: &[&str]) -> AssembleParams {
        AssembleParams {
            requested: requested.iter().map(|s| s.to_string()).collect(),
            github_pack_ids: vec![],
            manual_enrich: vec![],
            resume_run_id: String::new(),
            manual_targets: vec![],
        }
    }

    #[test]
    fn github_skipped_without_token_and_pg() {
        let settings = Settings::default();
        let plan = assemble_sources(&settings, &http(), &params(&["github"]));
        assert!(plan.sources.iter().all(|s| s.name() != "github"));
        assert!(plan.skipped.iter().any(|s| s.source == "github"));
    }

    #[test]
    fn fofa_skipped_without_key() {
        let settings = Settings::default();
        let plan = assemble_sources(&settings, &http(), &params(&["fofa"]));
        assert!(plan.sources.iter().all(|s| s.name() != "fofa"));
        assert!(
            plan.skipped
                .iter()
                .any(|s| s.source == "fofa" && s.reason.contains("FOFA_KEYS"))
        );
    }

    #[test]
    fn fofa_present_with_key() {
        let settings = Settings {
            fofa_keys: "k1".into(),
            ..Default::default()
        };
        let plan = assemble_sources(&settings, &http(), &params(&["fofa"]));
        assert!(plan.sources.iter().any(|s| s.name() == "fofa"));
        assert!(plan.skipped.iter().all(|s| s.source != "fofa"));
    }

    #[test]
    fn proxy_queries_prepended_when_enabled() {
        let settings = Settings {
            fofa_keys: "k1".into(),
            proxy_sub_enabled: true,
            proxy_sub_query_budget: 2,
            ..Default::default()
        };
        let plan = assemble_sources(&settings, &http(), &params(&["fofa"]));
        let fofa = plan
            .sources
            .iter()
            .find(|s| s.name() == "fofa")
            .expect("fofa source");
        let queries = fofa.query_ids();
        assert!(
            queries
                .first()
                .is_some_and(|q| q.contains("subscribe?token="))
        );
    }
}

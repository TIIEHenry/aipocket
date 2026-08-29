use crate::legacy_queries::{fofa_queries, prioritize_fofa_queries, shodan_product_queries};
use crate::packs::ProviderPack;

#[derive(Clone, Debug, Default)]
pub struct ComposedQueries {
    pub fofa: Vec<String>,
    pub shodan: Vec<String>,
    pub github: Vec<String>,
}

pub fn compose_queries(selected_packs: &[&ProviderPack]) -> ComposedQueries {
    let mut fofa = fofa_queries();
    fofa.extend(
        selected_packs
            .iter()
            .flat_map(|pack| pack.fofa_queries)
            .map(|q| q.to_string()),
    );
    let mut shodan = selected_packs
        .iter()
        .flat_map(|pack| pack.shodan_queries)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    shodan.extend(shodan_product_queries());
    let mut github = selected_packs
        .iter()
        .flat_map(|pack| pack.github_terms)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    prioritize_fofa_queries(&mut fofa);
    prioritize_fofa_queries(&mut shodan);
    prioritize_fofa_queries(&mut github);
    ComposedQueries {
        fofa,
        shodan,
        github,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs;

    #[test]
    fn compose_matches_web_full_scan_shape() {
        let registry = packs::registry();
        let selected = registry.values().copied().collect::<Vec<_>>();
        let q = compose_queries(&selected);
        assert!(q.fofa.len() > 60);
        assert!(
            q.fofa
                .iter()
                .any(|s| s.contains("api.deepseek.com") && s.contains("sk-"))
        );
        assert!(q.fofa.first().is_some_and(|s| s.starts_with("header=")));
        assert!(q.shodan.iter().any(|s| s == "http.html:sk-"));
        assert!(!q.github.is_empty());
    }
}

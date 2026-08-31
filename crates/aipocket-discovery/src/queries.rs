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

pub fn compose_proxy_queries(proxy_packs: &[&ProviderPack]) -> ComposedQueries {
    let mut fofa = proxy_packs
        .iter()
        .flat_map(|pack| pack.fofa_queries)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    let mut shodan = proxy_packs
        .iter()
        .flat_map(|pack| pack.shodan_queries)
        .map(|q| q.to_string())
        .collect::<Vec<_>>();
    let mut github = proxy_packs
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

pub fn merge_proxy_queries(base: &mut ComposedQueries, proxy: ComposedQueries, budget: usize) {
    let take = |mut items: Vec<String>| -> Vec<String> {
        items.truncate(budget);
        items
    };
    let proxy_fofa = take(proxy.fofa);
    let proxy_shodan = take(proxy.shodan);
    let proxy_github = take(proxy.github);
    base.fofa = proxy_fofa.into_iter().chain(base.fofa.drain(..)).collect();
    base.shodan = proxy_shodan
        .into_iter()
        .chain(base.shodan.drain(..))
        .collect();
    base.github = proxy_github
        .into_iter()
        .chain(base.github.drain(..))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs;
    use crate::proxy_packs::PROXY_PACKS;

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

    #[test]
    fn merge_proxy_prepends_budgeted_queries() {
        let ai = compose_queries(&[packs::registry()["openai"]]);
        let mut base = ai.clone();
        let proxy = compose_proxy_queries(&[&PROXY_PACKS[0]]);
        merge_proxy_queries(&mut base, proxy, 2);
        assert!(base.fofa.len() > ai.fofa.len());
        assert!(
            base.fofa
                .first()
                .is_some_and(|q| q.contains("subscribe?token="))
        );
    }
}

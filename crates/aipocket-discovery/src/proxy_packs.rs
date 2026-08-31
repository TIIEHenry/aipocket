use crate::packs::ProviderPack;

pub const PROXY_PACKS: &[ProviderPack] = &[
    ProviderPack {
        id: "proxy_v2board",
        fofa_queries: &[
            r#"banner="subscribe?token=" && banner="api/v1/client/subscribe""#,
            r#"banner="/api/v1/client/subscribe?token=""#,
            r#"header="/api/v1/client/subscribe?token=""#,
        ],
        shodan_queries: &[
            r#"http.html:"/api/v1/client/subscribe?token=""#,
            r#"http.html:"subscribe?token=""#,
        ],
        github_terms: &[
            "subscribe?token=",
            "SUBSCRIBE_URL filename:.env",
            "CLASH_SUB_URL filename:.env",
        ],
    },
    ProviderPack {
        id: "proxy_sspanel",
        fofa_queries: &[
            r#"banner="/link/" && banner="token""#,
            r#"banner="SUBSCRIBE_URL=""#,
        ],
        shodan_queries: &[r#"http.html:"/link/""#, r#"http.html:"SUBSCRIBE_URL=""#],
        github_terms: &["SUBSCRIBE_URL", "/link/ filename:.env"],
    },
    ProviderPack {
        id: "proxy_marzban",
        fofa_queries: &[
            r#"banner="/sub/" && banner="token""#,
            r#"header="/sub/""#,
        ],
        shodan_queries: &[r#"http.html:"/sub/""#],
        github_terms: &["/sub/ filename:.env", "CLASH_SUB_URL"],
    },
    ProviderPack {
        id: "proxy_3xui",
        fofa_queries: &[
            r#"banner="/sub/" && banner="3x-ui""#,
            r#"banner="subscribe?token=" && banner="x-ui""#,
        ],
        shodan_queries: &[
            r#"http.html:"3x-ui" "subscribe?token=""#,
            r#"http.html:"/sub/" "vmess""#,
        ],
        github_terms: &["3x-ui filename:.env", "x-ui subscribe"],
    },
    ProviderPack {
        id: "proxy_clash_env",
        fofa_queries: &[r#"banner="CLASH_SUB_URL=""#, r#"banner="SUBSCRIBE_URL=""#],
        shodan_queries: &[
            r#"http.html:"CLASH_SUB_URL=""#,
            r#"http.html:"SUBSCRIBE_URL=""#,
        ],
        github_terms: &[
            "CLASH_SUB_URL filename:.env",
            "SUBSCRIBE_URL filename:.env",
            "clash subscription url",
        ],
    },
    ProviderPack {
        id: "proxy_subconverter",
        fofa_queries: &[
            r#"banner="subconverter" && banner="subscribe""#,
            r#"banner="clash" && banner="config=""#,
        ],
        shodan_queries: &[r#"http.html:"subconverter""#, r#"http.html:"clash" "config=""#],
        github_terms: &["subconverter", "CLASH_SUB_URL", "subscribe?token="],
    },
];

pub const EXPERIMENTAL_PROXY_PACKS: &[ProviderPack] = &[
    ProviderPack {
        id: "proxy_nezha",
        fofa_queries: &[
            r#"banner="nezha" && banner="subscribe""#,
            r#"title="Nezha" && banner="token""#,
        ],
        shodan_queries: &[r#"http.html:"nezha" "subscribe""#],
        github_terms: &["nezha subscribe", "CLASH_SUB_URL nezha"],
    },
    ProviderPack {
        id: "proxy_wings",
        fofa_queries: &[r#"banner="wings" && banner="subscribe""#],
        shodan_queries: &[r#"http.html:"wings" "subscribe""#],
        github_terms: &["wings subscribe", "wings panel"],
    },
    ProviderPack {
        id: "proxy_mqpanel",
        fofa_queries: &[r#"banner="mqpanel" && banner="subscribe""#],
        shodan_queries: &[r#"http.html:"mqpanel""#],
        github_terms: &["mqpanel", "SUBSCRIBE_URL mqpanel"],
    },
    ProviderPack {
        id: "proxy_node_uri",
        fofa_queries: &[
            r#"banner="vmess://" && banner="uuid""#,
            r#"banner="trojan://" && banner="password""#,
        ],
        shodan_queries: &[
            r#"http.html:"vmess://""#,
            r#"http.html:"trojan://""#,
        ],
        github_terms: &["vmess://", "trojan:// filename:.env"],
    },
];

pub fn selected_proxy_packs(extra_pack_ids: &[&str]) -> Vec<&'static ProviderPack> {
    let mut out: Vec<&'static ProviderPack> = PROXY_PACKS.iter().collect();
    for id in extra_pack_ids {
        if let Some(pack) = EXPERIMENTAL_PROXY_PACKS.iter().find(|pack| pack.id == *id)
            && !out.iter().any(|existing| existing.id == pack.id)
        {
            out.push(pack);
        }
    }
    out
}

pub fn proxy_registry() -> std::collections::BTreeMap<&'static str, &'static ProviderPack> {
    PROXY_PACKS
        .iter()
        .chain(EXPERIMENTAL_PROXY_PACKS.iter())
        .map(|pack| (pack.id, pack))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_pack_ids_unique() {
        let mut ids = std::collections::HashSet::new();
        for pack in PROXY_PACKS.iter().chain(EXPERIMENTAL_PROXY_PACKS.iter()) {
            assert!(ids.insert(pack.id), "duplicate {}", pack.id);
            assert!(pack.id.starts_with("proxy_"));
        }
    }

    #[test]
    fn selected_proxy_packs_merges_extras() {
        let packs = selected_proxy_packs(&["proxy_nezha", "proxy_unknown"]);
        assert!(packs.iter().any(|pack| pack.id == "proxy_v2board"));
        assert!(packs.iter().any(|pack| pack.id == "proxy_nezha"));
        assert!(!packs.iter().any(|pack| pack.id == "proxy_unknown"));
    }
}

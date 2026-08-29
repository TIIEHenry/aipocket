use aipocket_api::settings::{SettingsUpdate, SettingsView, persist_env};
use aipocket_core::Settings;
use std::fs;

#[test]
fn settings_mask_and_env_round_trip_preserve_unrelated_lines() {
    let view = SettingsView::from_settings(&Settings {
        fofa_keys: "secret-one,secret-two".into(),
        tavily_key: "tvly-secret-12345".into(),
        ..Settings::default()
    });
    assert!(view.fofa_keys.contains("****"));
    assert!(view.tavily_key.contains("****"));
    let masked: SettingsUpdate =
        serde_json::from_value(serde_json::json!({"fofa_keys":view.fofa_keys,"tavily_key":view.tavily_key,"fofa_page_size":42,"tavily_base_url":"https://api.tavily.com"}))
            .unwrap();
    let updates = masked.env_updates();
    assert!(!updates.contains_key("FOFA_KEYS"));
    assert!(!updates.contains_key("TAVILY_KEY"));
    assert_eq!(updates["FOFA_PAGE_SIZE"], "42");
    assert_eq!(updates["TAVILY_BASE_URL"], "https://api.tavily.com");
    let budget: SettingsUpdate =
        serde_json::from_value(serde_json::json!({"fofa_query_budget":80,"fofa_max_pages":2}))
            .unwrap();
    let budget_updates = budget.env_updates();
    assert_eq!(budget_updates["FOFA_QUERY_BUDGET"], "80");
    assert_eq!(budget_updates["FOFA_MAX_PAGES"], "2");
    let path = std::env::temp_dir().join(format!("aipocket-settings-test-{}", std::process::id()));
    fs::write(&path, "# keep\nUNKNOWN=value\nFOFA_PAGE_SIZE=100\n").unwrap();
    persist_env(&path, &updates).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("# keep"));
    assert!(content.contains("UNKNOWN=value"));
    assert!(content.contains("FOFA_PAGE_SIZE=42"));
    let _ = fs::remove_file(path);
}

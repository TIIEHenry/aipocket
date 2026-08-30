use serde::Deserialize;

pub(crate) fn valid_kind() -> String {
    "valid".into()
}

pub(crate) fn all_source() -> String {
    "all".into()
}

#[derive(Default, Deserialize)]
pub(crate) struct Since {
    #[serde(default)]
    pub(crate) since: u64,
    pub(crate) token: Option<String>,
}

#[derive(Default, Deserialize)]
pub(crate) struct PageQuery {
    #[serde(default)]
    pub(crate) q: String,
    pub(crate) source: Option<String>,
    pub(crate) enabled_only: Option<bool>,
    pub(crate) limit: Option<i64>,
    pub(crate) offset: Option<i64>,
}

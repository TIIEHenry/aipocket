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

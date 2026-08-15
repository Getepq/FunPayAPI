use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SessionContext {
    #[serde(rename = "csrf-token")]
    pub(crate) csrf_token: Option<String>,
}

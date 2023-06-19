use std::collections::HashMap;

use serde::Deserialize;

/// Environment specific config.
#[derive(Deserialize)]
pub struct EnvConfig {
    /// Kafka config.
    pub kafka: HashMap<String, String>,

    /// Allowlist of programs to publish.
    /// If empty, all accounts are published.
    /// If not empty, only accounts owned by programs in this list are published.
    #[serde(default)]
    pub program_allowlist: Vec<String>,

    /// Allowlist from http url.
    /// If empty, all accounts are published.
    /// If not empty, only accounts owned by programs in this list are published.
    #[serde(default)]
    pub program_allowlist_url: String,

    /// Allowlist Authorization header value.
    /// If provided the request to the program_allowlist_url will add an
    /// 'Authorization: <value>' header.
    /// A sample auth header value would be 'Bearer my_long_secret_token'.
    #[serde(default)]
    pub program_allowlist_auth: String,

    /// Update iterval for allowlist from http url.
    #[serde(default)]
    pub program_allowlist_expiry_sec: u64,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            kafka: HashMap::new(),
            program_allowlist: Vec::new(),
            program_allowlist_url: "".to_owned(),
            program_allowlist_auth: "".to_owned(),
            program_allowlist_expiry_sec: 60,
        }
    }
}

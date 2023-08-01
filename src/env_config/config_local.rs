use serde::Deserialize;

/// Environment specific config for local development.
#[derive(Deserialize)]
pub struct EnvConfigLocal {
    /// Name of the environment
    #[serde(default)]
    pub name: String,

    /// Allowlist of programs to publish.
    /// If empty, all accounts are published since we are using this locally.
    /// If not empty, only accounts owned by programs in this list are published.
    #[serde(default)]
    pub program_allowlist: Vec<String>,

    /// URL to publish to.
    pub url: String,
}

impl Default for EnvConfigLocal {
    fn default() -> Self {
        Self {
            name: Default::default(),
            program_allowlist: Default::default(),
            url: Default::default(),
        }
    }
}

use serde::Deserialize;

/// Environment specific config.
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
}

impl Default for EnvConfigLocal {
    fn default() -> Self {
        Self {
            name: Default::default(),
            program_allowlist: Default::default(),
        }
    }
}

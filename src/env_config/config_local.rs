use serde::Deserialize;

/// Environment specific config for local development.
#[derive(Deserialize, Default)]
pub struct EnvConfigLocal {
    /// Name of the environment
    #[serde(default)]
    pub name: String,

    /// Allowlist of programs to publish.
    /// If empty, all accounts are published since we are using this locally.
    /// If not empty, only accounts owned by programs in this list are published including system
    /// program accounts unless [include_system_accounts] is `false`
    #[serde(default)]
    pub program_allowlist: Vec<String>,

    /// URL to publish to.
    pub url: String,

    /// If `true` then all system accounts are included when no [program_allowlist] is set
    /// Otherwise the following are ignored:
    /// - System Program: 11111111111111111111111111111111
    /// - BPF Loader:     BPFLoaderUpgradeab1e11111111111111111111111
    /// - Vote Program:   Vote111111111111111111111111111111111111111
    /// - Config Program: Config1111111111111111111111111111111111111
    #[serde(default)]
    pub include_system_accounts: bool,
}

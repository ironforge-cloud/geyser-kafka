mod config_kafka;
mod config_local;
pub use config_kafka::EnvConfigKafka;
pub use config_local::EnvConfigLocal;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum EnvConfig {
    Kafka(EnvConfigKafka),
    Local(EnvConfigLocal),
}

impl EnvConfig {
    pub fn program_allowlist(&self) -> &[String] {
        match self {
            EnvConfig::Kafka(c) => &c.program_allowlist,
            EnvConfig::Local(c) => &c.program_allowlist,
        }
    }
}

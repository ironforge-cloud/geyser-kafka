use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Cluster {
    #[default]
    Mainnet,
    Devnet,
    Testnet,
    Custom(String),
}

impl From<&str> for Cluster {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "mainnet" => Self::Mainnet,
            "devnet" => Self::Devnet,
            "testnet" => Self::Testnet,
            _ => Self::Custom(value.to_string()),
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Devnet => write!(f, "devnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Custom(value) => write!(f, "Custom({value})"),
        }
    }
}

impl Cluster {
    /// Derives a key to be used for Kafka events from the provided [cluster] and [program_id].
    /// This schema to derive keys is concistently used across all Ironforge services when
    /// sending/receiving Kafka events.
    pub fn key(&self, program_id: &str) -> String {
        format!("{self}:{program_id}")
    }
}

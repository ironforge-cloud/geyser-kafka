// Copyright 2022 Blockdaemon Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::str::FromStr;

use solana_program::pubkey::Pubkey;

use crate::EnvConfig;

use {
    crate::PrometheusService,
    rdkafka::producer::{DefaultProducerContext, ThreadedProducer},
    serde::Deserialize,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPluginError, Result as PluginResult,
    },
    std::{
        collections::{HashMap, HashSet},
        fs::File,
        io::Result as IoResult,
        net::SocketAddr,
        path::Path,
    },
};

/// Plugin config.
#[derive(Deserialize)]
pub struct Config {
    /// Time the plugin is given to flush out all messages to Kafka
    /// and gracefully shutdown upon exit request.
    #[serde(default)]
    pub shutdown_timeout_ms: u64,
    /// Kafka topic to send account updates to. Omit to disable.
    #[serde(default)]
    pub update_account_topic: String,
    /// Kafka topic overrides to send specific account updates to. Omit to disable.
    /// The keys are the alternate topics and the value is a collection of program
    /// addresses. If an account's owner matches one of those addresses its updates
    /// are sent to the alternative topic instead of [update_account_topic].
    ///
    /// ### Example
    /// ```json
    /// {
    ///   "update_account_topic_overrides": {
    ///      "geyser.mainnet.spl.account_update": [
    ///         "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    ///         "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
    ///      ]
    ///    }
    /// }
    /// ```
    #[serde(default)]
    pub update_account_topic_overrides: HashMap<String, HashSet<String>>,

    /// Kafka topic to send slot status updates to. Omit to disable.
    #[serde(default)]
    pub slot_status_topic: String,
    /// Kafka topic to send transaction updates to. Omit to disable.
    #[serde(default)]
    pub transaction_topic: String,
    /// Publish all accounts on startup. Omit to disable.
    #[serde(default)]
    pub publish_all_accounts: bool,
    /// Publishes account updates even if the txn_signature is not present.
    /// This will include account updates that occur without a corresponding
    /// transaction, i.e. caused by validator book-keeping. Omit to disable.
    #[serde(default)]
    pub publish_accounts_without_signature: bool,
    /// Wrap all messages in a unified wrapper object. Omit to disable.
    #[serde(default)]
    pub wrap_messages: bool,
    #[serde(default)]
    /// Kafka cluster and allow list configs for different environments. See [EnvConfig].
    pub environments: Vec<EnvConfig>,
    /// Prometheus endpoint.
    #[serde(default)]
    pub prometheus: Option<SocketAddr>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shutdown_timeout_ms: 30_000,
            update_account_topic: Default::default(),
            update_account_topic_overrides: Default::default(),
            slot_status_topic: Default::default(),
            transaction_topic: Default::default(),
            publish_all_accounts: Default::default(),
            publish_accounts_without_signature: Default::default(),
            wrap_messages: Default::default(),
            environments: Default::default(),
            prometheus: None,
        }
    }
}

impl Config {
    /// Read plugin from JSON file.
    pub fn read_from<P: AsRef<Path>>(config_path: P) -> PluginResult<Self> {
        let file = File::open(config_path)?;
        let mut this: Self = serde_json::from_reader(file)
            .map_err(|e| GeyserPluginError::ConfigFileReadError { msg: e.to_string() })?;
        for env_config in this.environments.iter_mut() {
            if let EnvConfig::Kafka(env_config) = env_config {
                env_config.fill_defaults();
            }
        }
        Ok(this)
    }

    pub fn update_topic_overrides_by_account(&self) -> HashMap<Vec<u8>, String> {
        let mut map = HashMap::new();
        for (topic, accounts) in &self.update_account_topic_overrides {
            for address in accounts {
                let pubkey = Pubkey::from_str(address)
                    .unwrap_or_else(|_| panic!("Invalid pubkey {address}"))
                    .to_bytes()
                    .to_vec();
                map.insert(pubkey, topic.clone());
            }
        }
        map
    }

    pub fn create_prometheus(&self) -> IoResult<Option<PrometheusService>> {
        self.prometheus.map(PrometheusService::new).transpose()
    }
}

pub type Producer = ThreadedProducer<DefaultProducerContext>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_env_topic_overrides() {
        let config = Config::read_from("test/fixtures/configs/single-env-topic-overrides.json")
            .expect("should deserialize config");
        let expected_overrides = {
            let mut map = HashMap::new();
            let set: HashSet<String> = vec![
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
            map.insert("geyser.mainnet.spl.account_update".to_string(), set);
            map
        };

        let expected_overrides_by_account = {
            let mut map = HashMap::new();
            map.insert(
                Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
                "geyser.mainnet.spl.account_update".to_string(),
            );
            map.insert(
                Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                    .unwrap()
                    .to_bytes()
                    .to_vec(),
                "geyser.mainnet.spl.account_update".to_string(),
            );
            map
        };
        assert_eq!(config.update_account_topic, "geyser.mainnet.account_update",);
        assert_eq!(config.update_account_topic_overrides, expected_overrides);
        assert_eq!(config.environments.len(), 1);
        assert_eq!(
            config.update_topic_overrides_by_account(),
            expected_overrides_by_account
        );
    }
}

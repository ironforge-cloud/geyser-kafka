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

use crate::EnvConfig;

use {
    rdkafka::producer::{DefaultProducerContext, ThreadedProducer},
    serde::Deserialize,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPluginError, Result as PluginResult,
    },
    std::{fs::File, path::Path},
};

/// Plugin config.
#[derive(Deserialize)]
pub struct Config {
    /// Graceful shutdown timeout.
    #[serde(default)]
    pub shutdown_timeout_ms: u64,
    /// Kafka topic to send account updates to.
    #[serde(default)]
    pub update_account_topic: String,
    /// Kafka topic to send slot status updates to.
    #[serde(default)]
    pub slot_status_topic: String,
    /// Kafka topic to send transaction to.
    #[serde(default)]
    pub transaction_topic: String,
    /// Publish all accounts on startup.
    #[serde(default)]
    pub publish_all_accounts: bool,
    /// Publishes account updates even if the txn_signature is not present.
    /// This will include account updates that occur without a corresponding
    /// transaction, i.e. caused by validator book-keeping.
    #[serde(default)]
    pub publish_accounts_without_signature: bool,
    /// Wrap all event message in a single message type.
    #[serde(default)]
    pub wrap_messages: bool,

    #[serde(default)]
    /// Kafka cluster and allow list configs for different environments
    pub environments: Vec<EnvConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shutdown_timeout_ms: 30_000,
            update_account_topic: Default::default(),
            slot_status_topic: Default::default(),
            transaction_topic: Default::default(),
            publish_all_accounts: Default::default(),
            publish_accounts_without_signature: Default::default(),
            wrap_messages: Default::default(),
            environments: Default::default(),
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
            env_config.fill_defaults();
        }
        Ok(this)
    }
}

pub type Producer = ThreadedProducer<DefaultProducerContext>;

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

use {
    rdkafka::{
        config::FromClientConfig,
        error::KafkaResult,
        producer::{DefaultProducerContext, ThreadedProducer},
        ClientConfig,
    },
    serde::Deserialize,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPluginError, Result as PluginResult,
    },
    std::{collections::HashMap, fs::File, path::Path},
};

/// Plugin config.
#[derive(Deserialize)]
pub struct Config {
    /// Kafka config.
    pub kafka: HashMap<String, String>,
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
    /// Wrap all event message in a single message type.
    #[serde(default)]
    pub wrap_messages: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kafka: HashMap::new(),
            shutdown_timeout_ms: 30_000,
            update_account_topic: "".to_owned(),
            slot_status_topic: "".to_owned(),
            transaction_topic: "".to_owned(),
            publish_all_accounts: false,
            publish_accounts_without_signature: false,
            program_allowlist: Vec::new(),
            program_allowlist_url: "".to_owned(),
            program_allowlist_auth: "".to_owned(),
            program_allowlist_expiry_sec: 60,
            wrap_messages: false,
        }
    }
}

impl Config {
    /// Read plugin from JSON file.
    pub fn read_from<P: AsRef<Path>>(config_path: P) -> PluginResult<Self> {
        let file = File::open(config_path)?;
        let mut this: Self = serde_json::from_reader(file)
            .map_err(|e| GeyserPluginError::ConfigFileReadError { msg: e.to_string() })?;
        this.fill_defaults();
        Ok(this)
    }

    /// Create rdkafka::FutureProducer from config.
    pub fn producer(&self) -> KafkaResult<Producer> {
        let mut config = ClientConfig::new();
        for (k, v) in self.kafka.iter() {
            config.set(k, v);
        }
        ThreadedProducer::from_config(&config)
    }

    fn set_default(&mut self, k: &'static str, v: &'static str) {
        if !self.kafka.contains_key(k) {
            self.kafka.insert(k.to_owned(), v.to_owned());
        }
    }

    fn fill_defaults(&mut self) {
        self.set_default("request.required.acks", "1");
        self.set_default("message.timeout.ms", "30000");
        self.set_default("compression.type", "lz4");
        self.set_default("partitioner", "murmur2_random");
    }
}

pub type Producer = ThreadedProducer<DefaultProducerContext>;

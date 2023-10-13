use std::collections::HashMap;

use rdkafka::{
    config::FromClientConfigAndContext, error::KafkaResult, producer::ThreadedProducer,
    ClientConfig,
};
use serde::Deserialize;

use crate::prom::StatsThreadedProducerContext;

/// Environment specific config.
#[derive(Deserialize)]
pub struct EnvConfigKafka {
    /// Name of the environment
    #[serde(default)]
    pub name: String,

    /// Kafka [`librdkafka` config options](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md).
    pub kafka: HashMap<String, String>,

    /// Allowlist of programs to publish.
    /// If empty, no accounts are published.
    /// If not empty, only accounts owned by programs in this list are published.
    #[serde(default)]
    pub program_allowlist: Vec<String>,

    /// URL to fetch allowlist updates from
    /// The file must be json, and with the following schema:
    /// ```json
    /// {
    ///   "result": [
    ///       "11111111111111111111111111111111",
    ///       "22222222222222222222222222222222"
    ///   ]
    /// }
    /// ```
    #[serde(default)]
    pub program_allowlist_url: String,

    /// Allowlist Authorization header value.
    /// If provided the request to the program_allowlist_url will add an
    /// 'Authorization: <value>' header.
    /// A sample auth header value would be 'Bearer my_long_secret_token'.
    #[serde(default)]
    pub program_allowlist_auth: String,

    /// Slots interval which determines how many slots to wait before updating the allowlist.
    #[serde(default)]
    pub program_allowlist_slot_interval: u64,
}

impl Default for EnvConfigKafka {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kafka: Default::default(),
            program_allowlist: Default::default(),
            program_allowlist_url: Default::default(),
            program_allowlist_auth: Default::default(),
            program_allowlist_slot_interval: 150, // roughly 60 secs
        }
    }
}

impl EnvConfigKafka {
    /// Create rdkafka::FutureProducer from config.
    pub fn producer(&self) -> KafkaResult<ThreadedProducer<StatsThreadedProducerContext>> {
        let mut config = ClientConfig::new();
        for (k, v) in self.kafka.iter() {
            config.set(k, v);
        }
        ThreadedProducer::from_config_and_context(&config, StatsThreadedProducerContext::default())
    }

    fn set_default(&mut self, k: &'static str, v: &'static str) {
        if !self.kafka.contains_key(k) {
            self.kafka.insert(k.to_owned(), v.to_owned());
        }
    }

    pub(crate) fn fill_defaults(&mut self) {
        self.set_default("request.required.acks", "1");
        self.set_default("message.timeout.ms", "30000");
        self.set_default("compression.type", "lz4");
        self.set_default("partitioner", "murmur2_random");
    }
}

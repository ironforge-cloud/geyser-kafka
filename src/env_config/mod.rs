mod config_kafka;
pub use config_kafka::EnvConfigKafka;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum EnvConfig {
    Kafka(EnvConfigKafka),
}

use std::array::TryFromSliceError;
use thiserror::Error;

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("TryFromSliceError ({0})")]
    TryFromSliceError(#[from] Box<TryFromSliceError>),

    #[error("SerdeJsonError ({0})")]
    SerdeJsonError(#[from] Box<serde_json::Error>),

    #[error("KafkaError ({0})")]
    KafkaError(#[from] Box<rdkafka::error::KafkaError>),

    #[error("UreqError ({0})")]
    UreqError(#[from] Box<ureq::Error>),
}

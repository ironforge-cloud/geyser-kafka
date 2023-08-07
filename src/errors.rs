use std::array::TryFromSliceError;
use thiserror::Error;

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("TryFromSliceError ({0})")]
    TryFromSliceError(#[from] TryFromSliceError),

    #[error("SerdeJsonError ({0})")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("KafkaError ({0})")]
    KafkaError(#[from] rdkafka::error::KafkaError),

    #[error("UreqError ({0})")]
    UreqError(#[from] ureq::Error),
}

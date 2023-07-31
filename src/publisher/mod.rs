mod filtering_publisher;
pub mod kafka_publisher;
pub use filtering_publisher::FilteringPublisher;
use rdkafka::error::KafkaError;

use crate::{allowlist::Allowlist, SlotStatusEvent, TransactionEvent, UpdateAccountEvent};

pub enum Publisher {
    FilteringPublisher(FilteringPublisher),
}

impl Publisher {
    // -----------------
    // Filter
    // -----------------
    pub fn get_allowlist(&self) -> Allowlist {
        match self {
            Publisher::FilteringPublisher(p) => p.get_allowlist(),
        }
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_account_key(account_key),
        }
    }

    // -----------------
    // Publisher
    // -----------------
    pub fn env(&self) -> &str {
        match self {
            Publisher::FilteringPublisher(p) => p.env(),
        }
    }

    pub fn wants_update_account(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_update_account(),
        }
    }

    pub fn wants_slot_status(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_slot_status(),
        }
    }

    pub fn wants_transaction(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_transaction(),
        }
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> Result<(), KafkaError> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_account(ev),
        }
    }

    pub fn update_slot_status(&self, ev: SlotStatusEvent) -> Result<(), KafkaError> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_slot_status(ev),
        }
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> Result<(), KafkaError> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_transaction(ev),
        }
    }
}

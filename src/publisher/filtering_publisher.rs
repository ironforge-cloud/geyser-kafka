use rdkafka::error::KafkaError;

use crate::{allowlist::Allowlist, Filter, SlotStatusEvent, TransactionEvent, UpdateAccountEvent};

use super::kafka_publisher::KafkaPublisher;

pub struct FilteringPublisher {
    publisher: KafkaPublisher,
    filter: Filter,
}

impl FilteringPublisher {
    pub fn new(publisher: KafkaPublisher, filter: Filter) -> Self {
        Self { publisher, filter }
    }

    // -----------------
    // Filter
    // -----------------
    pub fn get_allowlist(&self) -> Allowlist {
        self.filter.get_allowlist()
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
        self.filter.wants_account_key(account_key)
    }

    // -----------------
    // Publisher
    // -----------------
    pub fn env(&self) -> &str {
        &self.publisher.env
    }

    pub fn wants_update_account(&self) -> bool {
        self.publisher.wants_update_account()
    }

    pub fn wants_slot_status(&self) -> bool {
        self.publisher.wants_slot_status()
    }

    pub fn wants_transaction(&self) -> bool {
        self.publisher.wants_transaction()
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> Result<(), KafkaError> {
        self.publisher.update_account(ev)
    }

    pub fn update_slot_status(&self, ev: SlotStatusEvent) -> Result<(), KafkaError> {
        self.publisher.update_slot_status(ev)
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> Result<(), KafkaError> {
        self.publisher.update_transaction(ev)
    }
}

use crate::{allowlist::Allowlist, Filter, SlotStatusEvent, TransactionEvent, UpdateAccountEvent};

use rdkafka::error::KafkaError;

pub struct LocalPublisher {
    filter: Filter,
}

impl LocalPublisher {
    #[allow(unused)]
    pub fn new(filter: Filter) -> Self {
        Self { filter }
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
        todo!()
    }

    pub fn wants_update_account(&self) -> bool {
        todo!()
    }

    pub fn wants_slot_status(&self) -> bool {
        todo!()
    }

    pub fn wants_transaction(&self) -> bool {
        todo!()
    }

    pub fn update_account(&self, _ev: UpdateAccountEvent) -> Result<(), KafkaError> {
        todo!()
    }

    pub fn update_slot_status(&self, _ev: SlotStatusEvent) -> Result<(), KafkaError> {
        todo!()
    }

    pub fn update_transaction(&self, _ev: TransactionEvent) -> Result<(), KafkaError> {
        todo!()
    }
}

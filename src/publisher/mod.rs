mod filtering_publisher;
pub mod kafka_publisher;
mod local_publisher;
mod serializable_events;

use crate::{
    allowlist::Allowlist, PluginResult, SlotStatusEvent, TransactionEvent, UpdateAccountEvent,
};
pub use filtering_publisher::FilteringPublisher;
pub use local_publisher::LocalPublisher;

pub enum Publisher {
    FilteringPublisher(FilteringPublisher),
    #[allow(unused)]
    LocalPublisher(LocalPublisher),
}

impl Publisher {
    // -----------------
    // Filter
    // -----------------
    pub fn get_allowlist(&self) -> Allowlist {
        match self {
            Publisher::FilteringPublisher(p) => p.get_allowlist(),
            Publisher::LocalPublisher(p) => p.get_allowlist(),
        }
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_account_key(account_key),
            Publisher::LocalPublisher(p) => p.wants_account_key(account_key),
        }
    }

    // -----------------
    // Publisher
    // -----------------
    pub fn env(&self) -> &str {
        match self {
            Publisher::FilteringPublisher(p) => p.env(),
            Publisher::LocalPublisher(p) => p.env(),
        }
    }

    pub fn wants_update_account(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_update_account(),
            Publisher::LocalPublisher(p) => p.wants_update_account(),
        }
    }

    pub fn wants_slot_status(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_slot_status(),
            Publisher::LocalPublisher(p) => p.wants_slot_status(),
        }
    }

    pub fn wants_transaction(&self) -> bool {
        match self {
            Publisher::FilteringPublisher(p) => p.wants_transaction(),
            Publisher::LocalPublisher(p) => p.wants_transaction(),
        }
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> PluginResult<()> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_account(ev).map_err(Box::new)?,
            Publisher::LocalPublisher(p) => p.update_account(ev)?,
        }
        Ok(())
    }

    pub fn update_slot_status(&self, ev: SlotStatusEvent) -> PluginResult<()> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_slot_status(ev).map_err(Box::new)?,
            Publisher::LocalPublisher(p) => p.update_slot_status(ev)?,
        }
        Ok(())
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> PluginResult<()> {
        match self {
            Publisher::FilteringPublisher(p) => p.update_transaction(ev).map_err(Box::new)?,
            Publisher::LocalPublisher(p) => p.update_transaction(ev)?,
        }
        Ok(())
    }
}

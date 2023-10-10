use crate::{
    allowlist::Allowlist, Config, Filter, PluginResult, SlotStatusEvent, TransactionEvent,
    UpdateAccountEvent, SYSTEM_PROGRAMS,
};
use solana_program::pubkey::Pubkey;
use std::{collections::HashSet, str::FromStr};

use log::debug;
use serde::Serialize;

use super::serializable_events::{
    SerializableSlotStatusEvent, SerializableTransactionEvent, SerializableUpdateAccountEvent,
};

// -----------------
// LocalPublisher
// -----------------
pub struct LocalPublisher {
    pub(crate) env: String,
    filter: Filter,
    update_account_path: String,
    update_slot_status_path: String,
    update_transaction_path: String,
    root_url: String,
    include_system_accounts: bool,
    system_programs: HashSet<[u8; 32]>,
}

impl LocalPublisher {
    pub fn new(
        filter: Filter,
        config: &Config,
        env: String,
        root_url: String,
        include_system_accounts: bool,
    ) -> Self {
        let system_programs = SYSTEM_PROGRAMS
            .iter()
            .map(|s| Pubkey::from_str(s).unwrap().to_bytes())
            .collect::<HashSet<_>>();
        Self {
            env,
            update_account_path: config.update_account_topic.clone(),
            update_slot_status_path: config.slot_status_topic.clone(),
            update_transaction_path: config.transaction_topic.clone(),
            root_url,
            filter,
            include_system_accounts,
            system_programs,
        }
    }

    // -----------------
    // Filter
    // -----------------
    pub fn get_allowlist(&self) -> Allowlist {
        self.filter.get_allowlist()
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
        if self.filter.allow_list_is_empty() && !self.include_system_accounts {
            let slice: &[u8; 32] = account_key[0..32].try_into().unwrap();
            return !self.system_programs.contains(slice);
        }
        self.filter.wants_account_key(account_key, true)
    }

    // -----------------
    // Publisher
    // -----------------
    pub fn env(&self) -> &str {
        self.env.as_str()
    }

    pub fn wants_update_account(&self) -> bool {
        !self.update_account_path.is_empty()
    }

    pub fn wants_slot_status(&self) -> bool {
        !self.update_slot_status_path.is_empty()
    }

    pub fn wants_transaction(&self) -> bool {
        !self.update_transaction_path.is_empty()
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> PluginResult<()> {
        self.publish_event(
            &self.update_account_path,
            &SerializableUpdateAccountEvent::from(ev),
        )
    }

    pub fn update_slot_status(&self, ev: SlotStatusEvent) -> PluginResult<()> {
        self.publish_event(
            &self.update_slot_status_path,
            &SerializableSlotStatusEvent::from(ev),
        )
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> PluginResult<()> {
        self.publish_event(
            &self.update_transaction_path,
            &SerializableTransactionEvent::from(ev),
        )
    }

    fn publish_event<T: Serialize>(&self, path: &str, ev: &T) -> PluginResult<()> {
        let payload = serde_json::to_vec(ev).map_err(Box::new)?;
        let uri = format!("{}/{}", self.root_url, path);
        ureq::post(&uri)
            .set("Content-Type", "application/json")
            .send_bytes(&payload)
            .map_err(Box::new)?;
        debug!("Published event to {}", uri);
        Ok(())
    }
}

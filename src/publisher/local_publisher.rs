use crate::{
    allowlist::Allowlist, Config, Filter, PluginResult, SlotStatusEvent, TransactionEvent,
    UpdateAccountEvent,
};

use log::info;
use serde::Serialize;

// -----------------
// Serializable Events
// -----------------
#[derive(Serialize)]
pub struct SerializableUpdateAccountEvent {
    slot: u64,
    pubkey: Vec<u8>,
    lamports: u64,
    owner: Vec<u8>,
    executable: bool,
    rent_epoch: u64,
    data: Vec<u8>,
    write_version: u64,
    txn_signature: Option<Vec<u8>>,
}

impl From<UpdateAccountEvent> for SerializableUpdateAccountEvent {
    fn from(ev: UpdateAccountEvent) -> Self {
        Self {
            slot: ev.slot,
            pubkey: ev.pubkey,
            lamports: ev.lamports,
            owner: ev.owner,
            executable: ev.executable,
            rent_epoch: ev.rent_epoch,
            data: ev.data,
            write_version: ev.write_version,
            txn_signature: ev.txn_signature,
        }
    }
}

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
}

impl LocalPublisher {
    #[allow(unused)]
    pub fn new(filter: Filter, config: &Config, env: String, root_url: String) -> Self {
        Self {
            env,
            update_account_path: config.update_account_topic.clone(),
            update_slot_status_path: config.slot_status_topic.clone(),
            update_transaction_path: config.transaction_topic.clone(),
            root_url,
            filter,
        }
    }

    // -----------------
    // Filter
    // -----------------
    pub fn get_allowlist(&self) -> Allowlist {
        self.filter.get_allowlist()
    }

    pub fn wants_account_key(&self, account_key: &[u8]) -> bool {
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

    pub fn update_slot_status(&self, _ev: SlotStatusEvent) -> PluginResult<()> {
        todo!()
        // self.publish_event(&self.update_slot_status_path, &ev)
    }

    pub fn update_transaction(&self, _ev: TransactionEvent) -> PluginResult<()> {
        todo!()
        // self.publish_event(&self.update_transaction_path, &ev)
    }

    fn publish_event<T: Serialize>(&self, path: &str, ev: &T) -> PluginResult<()> {
        let payload = serde_json::to_vec(ev)?;
        let uri = format!("{}/{}", self.root_url, path);
        let res = ureq::post(&uri).send_bytes(&payload)?;
        info!("Published event to {}", uri);
        info!("res {res:#?}");
        Ok(())
    }
}

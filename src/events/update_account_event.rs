use solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use solana_program::{pubkey::Pubkey, slot_history::Slot};

use crate::{publisher::Publisher, PluginError, UpdateAccountEvent};

use super::replica_transaction_info::ReplicaTransactionInfo;

impl UpdateAccountEvent {
    pub fn for_account_deletion(
        account: Pubkey,
        owner: Vec<u8>,
        tx: &ReplicaTransactionInfo,
    ) -> Self {
        let signature: Vec<u8> = tx.signature().as_ref().to_vec();
        Self {
            pubkey: account.to_bytes().to_vec(),
            slot: tx.slot(),
            lamports: 0,
            owner,
            executable: false,
            rent_epoch: 0,
            data: vec![],
            // TODO(thlorenz): We do not know the actual write_version
            // This could lead to issues when an account is _deleted_ by draining its lamports
            // and then recreated, i.e. by airdropping money and reassigned to the previous owner
            // A way around this would be to track the current write_version for each slot
            write_version: 0,
            txn_signature: Some(signature),
        }
    }
}

pub fn publish_deleted_account_events(
    publishers: &[&Publisher],
    transaction: &ReplicaTransactionInfoV2,
    slot: Slot,
) -> Vec<PluginError> {
    let events = create_deleted_account_events(publishers, transaction, slot);
    let mut errors = vec![];
    for event in events.into_iter() {
        let owner = &event.owner;
        for publisher in publishers {
            if publisher.wants_account_key(owner) {
                match publisher.update_account(event.clone()) {
                    Ok(_) => {}
                    Err(err) => errors.push(err),
                }
            }
        }
    }
    errors
}

fn create_deleted_account_events(
    publishers: &[&Publisher],
    transaction: &ReplicaTransactionInfoV2,
    slot: Slot,
) -> Vec<UpdateAccountEvent> {
    let tx = ReplicaTransactionInfo::new(transaction, slot);
    let deleted_accounts = tx.account_addresses_with_zero_post_balance();
    if deleted_accounts.is_empty() {
        return vec![];
    }

    let account_keys = tx.account_keys();

    // We assume that one of the accounts (not deleted) is the program that owns the
    // deleted account and thus create an UpdateAccountEvent for each, given that a
    // publisher is interested in it.
    let programs_we_want = account_keys
        .iter()
        .filter(|key| !deleted_accounts.contains(key))
        .filter(|key| {
            publishers
                .iter()
                .any(|p| p.wants_account_key(&key.to_bytes()))
        })
        .collect::<Vec<_>>();

    deleted_accounts
        .into_iter()
        .flat_map(|deleted_account| {
            programs_we_want
                .iter()
                .map(|owner| {
                    UpdateAccountEvent::for_account_deletion(
                        deleted_account,
                        owner.to_bytes().to_vec(),
                        &tx,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

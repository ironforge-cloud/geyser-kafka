use std::ops::Deref;

use solana_geyser_plugin_interface::geyser_plugin_interface::ReplicaTransactionInfoV2;
use solana_program::{message::SanitizedMessage, pubkey::Pubkey, slot_history::Slot};
use solana_sdk::signature::Signature;

pub struct ReplicaTransactionInfo<'a> {
    info: &'a ReplicaTransactionInfoV2<'a>,
    slot: Slot,
}

impl<'a> Deref for ReplicaTransactionInfo<'a> {
    type Target = ReplicaTransactionInfoV2<'a>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl<'a> ReplicaTransactionInfo<'a> {
    pub fn new(info: &'a ReplicaTransactionInfoV2<'_>, slot: u64) -> Self {
        Self { info, slot }
    }

    pub fn account_addresses_with_zero_post_balance(&self) -> Vec<Pubkey> {
        if self.is_vote {
            return vec![];
        }
        let zero_balance_indexes = self
            .transaction_status_meta
            .post_balances
            .iter()
            .enumerate()
            .filter(|(idx, balance)| {
                **balance == 0 && self.transaction_status_meta.pre_balances[*idx] > 0
            })
            .map(|(idx, _)| idx)
            .collect::<Vec<usize>>();

        if zero_balance_indexes.is_empty() {
            return vec![];
        }

        let account_keys = self.account_keys();
        zero_balance_indexes
            .iter()
            .map(|&idx| account_keys[idx].clone())
            .collect::<Vec<_>>()
    }

    pub(crate) fn account_keys(&self) -> &Vec<Pubkey> {
        match self.transaction.message() {
            SanitizedMessage::Legacy(legacy) => &legacy.message.account_keys,
            SanitizedMessage::V0(v0) => &v0.message.account_keys,
        }
    }

    pub(crate) fn slot(&self) -> u64 {
        self.slot
    }

    pub(crate) fn signature(&self) -> &Signature {
        self.info.transaction.signature()
    }
}

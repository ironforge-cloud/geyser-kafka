use std::ops::Deref;

use log::debug;
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
        self.info
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
            .into_iter()
            .flat_map(|idx| {
                let key = account_keys.get(idx);
                // Even though this shouldn't happen, we see a balances array that doesn't match with
                // the accout_keys array at times. We warn here and exclude this zero_balance
                // account instead.
                // However at that point there is not telling if the zero_balances which aren't out
                // of bounds are even matching up with the account keys at all
                if key.is_none() {
                    debug!(
                        "Prebalance idx {idx} is out of bounds for account_keys {}",
                        account_keys.len()
                    );
                }
                key
            })
            .cloned()
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

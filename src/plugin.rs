// Copyright 2022 Blockdaemon Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use {
    crate::*,
    log::{debug, info, log_enabled},
    rdkafka::util::get_rdkafka_version,
    simple_error::simple_error,
    solana_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError as PluginError, ReplicaAccountInfoV2,
        ReplicaAccountInfoVersions, ReplicaTransactionInfoV2, ReplicaTransactionInfoVersions,
        Result as PluginResult, SlotStatus as PluginSlotStatus,
    },
    solana_program::pubkey::Pubkey,
    std::fmt::{Debug, Formatter},
};

pub struct FilteredPublisher {
    publisher: Publisher,
    filter: Filter,
}

#[derive(Default)]
pub struct KafkaPlugin {
    publishers: Option<Vec<FilteredPublisher>>,
    publish_all_accounts: bool,
    publish_accounts_without_signature: bool,
}

impl Debug for KafkaPlugin {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl GeyserPlugin for KafkaPlugin {
    fn name(&self) -> &'static str {
        "KafkaPlugin"
    }

    fn on_load(&mut self, config_file: &str) -> PluginResult<()> {
        if self.publishers.is_some() {
            let err = simple_error!("plugin already loaded");
            return Err(PluginError::Custom(Box::new(err)));
        }

        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );
        let config = Config::read_from(config_file)?;
        self.publish_all_accounts = config.publish_all_accounts;
        self.publish_accounts_without_signature = config.publish_accounts_without_signature;

        let (version_n, version_s) = get_rdkafka_version();
        info!("rd_kafka_version: {:#08x}, {}", version_n, version_s);

        let publishers = Vec::new();
        for env_config in config.environments {
            let producer = env_config
                .producer()
                .map_err(|e| PluginError::Custom(Box::new(e)))?;
            info!("Created rdkafka::FutureProducer");

            let publisher = Publisher::new(producer, &config);
            let filter = Filter::new(&env_config);
            publishers.push(FilteredPublisher { publisher, filter })
        }
        self.publishers = Some(publishers);
        info!("Spawned producers");

        Ok(())
    }

    fn on_unload(&mut self) {
        self.publishers = None;
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> PluginResult<()> {
        if is_startup && !self.publish_all_accounts {
            return Ok(());
        }

        let info = Self::unwrap_update_account(account);
        if !self.publish_accounts_without_signature && info.txn_signature.is_none() {
            return Ok(());
        }
        if !self.unwrap_filters().wants_account_key(info.owner) {
            Self::log_ignore_account_update(info);
            return Ok(());
        }

        // Trigger an update of the remote allowlist
        // but don't wait for it to complete.
        self.unwrap_filters()
            .get_allowlist()
            .update_from_http_if_needed_async();

        let event = UpdateAccountEvent {
            slot,
            pubkey: info.pubkey.to_vec(),
            lamports: info.lamports,
            owner: info.owner.to_vec(),
            executable: info.executable,
            rent_epoch: info.rent_epoch,
            data: info.data.to_vec(),
            write_version: info.write_version,
            txn_signature: info.txn_signature.map(|sig| sig.as_ref().to_owned()),
        };

        let publishers = self.unwrap_publishers();
        let mut errors = Vec::new();
        for publisher in publishers {
            if let Err(err) = publisher.update_account(event) {
                errors.push(err.to_string());
            }
        }
        if !errors.is_empty() {
            // TODO(thlorenz): think about naming environments and including that info here
            Err(PluginError::AccountsUpdateError {
                msg: errors.join(" | "),
            })
        } else {
            Ok(())
        }
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: PluginSlotStatus,
    ) -> PluginResult<()> {
        let publishers = self.unwrap_publishers();

        let mut errors = Vec::new();
        for publisher in publishers {
            if !publisher.wants_slot_status() {
                continue;
            }

            let event = SlotStatusEvent {
                slot,
                parent: parent.unwrap_or(0),
                status: SlotStatus::from(status).into(),
            };

            if let Err(err) = publisher.update_slot_status(event) {
                errors.push(err.to_string());
            }
        }

        if !errors.is_empty() {
            Err(PluginError::SlotStatusUpdateError {
                msg: errors.join(" | "),
            })
        } else {
            Ok(())
        }
    }

    fn notify_transaction(
        &mut self,
        transaction: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> PluginResult<()> {
        let publishers = self.unwrap_publishers();
        let mut errors = Vec::new();
        for publisher in publishers {
            if !publisher.wants_transaction() {
                continue;
            }

            let info = Self::unwrap_transaction(transaction);
            let maybe_ignored = info
                .transaction
                .message()
                .account_keys()
                .iter()
                .find(|key| !self.unwrap_filters().wants_account_key(&key.to_bytes()));
            if maybe_ignored.is_some() {
                debug!(
                    "Ignoring transaction {:?} due to account key: {:?}",
                    info.signature,
                    &maybe_ignored.unwrap()
                );
                return Ok(());
            }

            let event = Self::build_transaction_event(slot, info);

            if let Err(err) = publisher.update_transaction(event) {
                errors.push(err.to_string());
            }
        }
        if !errors.is_empty() {
            Err(PluginError::TransactionUpdateError {
                msg: errors.join(" | "),
            })
        } else {
            Ok(())
        }
    }

    fn account_data_notifications_enabled(&self) -> bool {
        self.unwrap_publishers()
            .iter()
            .any(|p| p.wants_update_account())
    }

    fn transaction_notifications_enabled(&self) -> bool {
        self.unwrap_publishers()
            .iter()
            .any(|p| p.wants_transaction())
    }
}

impl KafkaPlugin {
    pub fn new() -> Self {
        Default::default()
    }

    fn unwrap_publishers(&self) -> Vec<&Publisher> {
        self.publishers
            .as_ref()
            .expect("filtered publishers are unavailable")
            .iter()
            .map(|x| &x.publisher)
            .collect::<Vec<_>>()
    }

    fn unwrap_filters(&self) -> Vec<&Filter> {
        self.publishers
            .as_ref()
            .expect("filtered publishers are unavailable")
            .iter()
            .map(|x| &x.filter)
            .collect::<Vec<_>>()
    }

    fn unwrap_update_account(account: ReplicaAccountInfoVersions) -> &ReplicaAccountInfoV2 {
        match account {
            ReplicaAccountInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaAccountInfoVersions::V0_0_1 unsupported, please upgrade your Solana node.");
            }
            ReplicaAccountInfoVersions::V0_0_2(info) => info,
        }
    }

    fn unwrap_transaction(
        transaction: ReplicaTransactionInfoVersions,
    ) -> &ReplicaTransactionInfoV2 {
        match transaction {
            ReplicaTransactionInfoVersions::V0_0_1(_info) => {
                panic!("ReplicaTransactionInfoVersions::V0_0_1 unsupported, please upgrade your Solana node.");
            }
            ReplicaTransactionInfoVersions::V0_0_2(info) => info,
        }
    }

    fn build_compiled_instruction(
        ix: &solana_program::instruction::CompiledInstruction,
    ) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index: ix.program_id_index as u32,
            accounts: ix.clone().accounts.into_iter().map(|v| v as u32).collect(),
            data: ix.data.clone(),
        }
    }

    fn build_message_header(header: &solana_program::message::MessageHeader) -> MessageHeader {
        MessageHeader {
            num_required_signatures: header.num_required_signatures as u32,
            num_readonly_signed_accounts: header.num_readonly_signed_accounts as u32,
            num_readonly_unsigned_accounts: header.num_readonly_unsigned_accounts as u32,
        }
    }

    fn build_transaction_token_balance(
        transaction_token_account_balance: solana_transaction_status::TransactionTokenBalance,
    ) -> TransactionTokenBalance {
        TransactionTokenBalance {
            account_index: transaction_token_account_balance.account_index as u32,
            ui_token_account: Some(UiTokenAmount {
                ui_amount: transaction_token_account_balance.ui_token_amount.ui_amount,
                decimals: transaction_token_account_balance.ui_token_amount.decimals as u32,
                amount: transaction_token_account_balance.ui_token_amount.amount,
                ui_amount_string: transaction_token_account_balance
                    .ui_token_amount
                    .ui_amount_string,
            }),
            mint: transaction_token_account_balance.mint,
            owner: transaction_token_account_balance.owner,
        }
    }

    fn build_transaction_event(
        slot: u64,
        transaction: &ReplicaTransactionInfoV2,
    ) -> TransactionEvent {
        let transaction_status_meta = transaction.transaction_status_meta;
        let signature = transaction.signature;
        let is_vote = transaction.is_vote;
        let transaction = transaction.transaction;
        TransactionEvent {
            is_vote,
            slot,
            signature: signature.as_ref().into(),
            transaction_status_meta: Some(TransactionStatusMeta {
                is_status_err: transaction_status_meta.status.is_err(),
                error_info: match &transaction_status_meta.status {
                    Err(e) => e.to_string(),
                    Ok(_) => "".to_owned(),
                },
                rewards: transaction_status_meta
                    .rewards
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(|x| Reward {
                        pubkey: x.pubkey,
                        lamports: x.lamports,
                        post_balance: x.post_balance,
                        reward_type: match x.reward_type {
                            Some(r) => r as i32,
                            None => 0,
                        },
                        commission: match x.commission {
                            Some(v) => v as u32,
                            None => 0,
                        },
                    })
                    .collect(),
                fee: transaction_status_meta.fee,
                log_messages: match &transaction_status_meta.log_messages {
                    Some(v) => v.to_owned(),
                    None => vec![],
                },
                inner_instructions: match &transaction_status_meta.inner_instructions {
                    None => vec![],
                    Some(inners) => inners
                        .clone()
                        .into_iter()
                        .map(|inner| InnerInstruction {
                            index: inner.index as u32,
                            instructions: inner
                                .instructions
                                .iter()
                                .map(Self::build_compiled_instruction)
                                .collect(),
                        })
                        .collect(),
                },
                pre_balances: transaction_status_meta.pre_balances.clone(),
                post_balances: transaction_status_meta.post_balances.clone(),
                pre_token_balances: match &transaction_status_meta.pre_token_balances {
                    Some(v) => v
                        .clone()
                        .into_iter()
                        .map(Self::build_transaction_token_balance)
                        .collect(),
                    None => vec![],
                },
                post_token_balances: match &transaction_status_meta.post_token_balances {
                    Some(v) => v
                        .clone()
                        .into_iter()
                        .map(Self::build_transaction_token_balance)
                        .collect(),
                    None => vec![],
                },
            }),
            transaction: Some(SanitizedTransaction {
                message_hash: transaction.message_hash().to_bytes().into(),
                is_simple_vote_transaction: transaction.is_simple_vote_transaction(),
                message: Some(SanitizedMessage {
                    message_payload: Some(match transaction.message() {
                        solana_program::message::SanitizedMessage::Legacy(lv) => {
                            sanitized_message::MessagePayload::Legacy(LegacyLoadedMessage {
                                message: Some(LegacyMessage {
                                    header: Some(Self::build_message_header(&lv.message.header)),
                                    account_keys: lv
                                        .message
                                        .account_keys
                                        .clone()
                                        .into_iter()
                                        .map(|k| k.as_ref().into())
                                        .collect(),
                                    instructions: lv
                                        .message
                                        .instructions
                                        .iter()
                                        .map(Self::build_compiled_instruction)
                                        .collect(),
                                    recent_block_hash: lv.message.recent_blockhash.as_ref().into(),
                                }),
                                is_writable_account_cache: (0..(lv.account_keys().len() - 1))
                                    .map(|i: usize| lv.is_writable(i))
                                    .collect(),
                            })
                        }
                        solana_program::message::SanitizedMessage::V0(v0) => {
                            sanitized_message::MessagePayload::V0(V0LoadedMessage {
                                message: Some(V0Message {
                                    header: Some(Self::build_message_header(&v0.message.header)),
                                    account_keys: v0
                                        .message
                                        .account_keys
                                        .clone()
                                        .into_iter()
                                        .map(|k| k.as_ref().into())
                                        .collect(),
                                    recent_block_hash: v0.message.recent_blockhash.as_ref().into(),
                                    instructions: v0
                                        .message
                                        .instructions
                                        .iter()
                                        .map(Self::build_compiled_instruction)
                                        .collect(),
                                    address_table_lookup: v0
                                        .message
                                        .address_table_lookups
                                        .clone()
                                        .into_iter()
                                        .map(|vf| MessageAddressTableLookup {
                                            account_key: vf.account_key.as_ref().into(),
                                            writable_indexes: vf
                                                .writable_indexes
                                                .iter()
                                                .map(|x| *x as u32)
                                                .collect(),
                                            readonly_indexes: vf
                                                .readonly_indexes
                                                .iter()
                                                .map(|x| *x as u32)
                                                .collect(),
                                        })
                                        .collect(),
                                }),
                                loaded_adresses: Some(LoadedAddresses {
                                    writable: v0
                                        .loaded_addresses
                                        .writable
                                        .clone()
                                        .into_iter()
                                        .map(|x| x.as_ref().into())
                                        .collect(),
                                    readonly: v0
                                        .loaded_addresses
                                        .readonly
                                        .clone()
                                        .into_iter()
                                        .map(|x| x.as_ref().into())
                                        .collect(),
                                }),
                                is_writable_account_cache: (0..(v0.account_keys().len() - 1))
                                    .map(|i: usize| v0.is_writable(i))
                                    .collect(),
                            })
                        }
                    }),
                }),
                signatures: transaction
                    .signatures()
                    .iter()
                    .copied()
                    .map(|x| x.as_ref().into())
                    .collect(),
            }),
        }
    }

    fn log_ignore_account_update(info: &ReplicaAccountInfoV2) {
        if log_enabled!(::log::Level::Debug) {
            match <&[u8; 32]>::try_from(info.owner) {
                Ok(key) => debug!(
                    "Ignoring update for account key: {:?}",
                    Pubkey::new_from_array(*key)
                ),
                // Err should never happen because wants_account_key only returns false if the input is &[u8; 32]
                Err(_err) => debug!("Ignoring update for account key: {:?}", info.owner),
            };
        }
    }
}

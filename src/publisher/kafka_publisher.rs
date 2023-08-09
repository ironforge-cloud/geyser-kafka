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

use std::collections::HashMap;

use solana_program::pubkey::Pubkey;

use {
    crate::{
        message_wrapper::EventMessage::{self, Account, Slot, Transaction},
        prom::{
            StatsThreadedProducerContext, UPLOAD_ACCOUNTS_TOTAL, UPLOAD_SLOTS_TOTAL,
            UPLOAD_TRANSACTIONS_TOTAL,
        },
        Cluster, Config, MessageWrapper, SlotStatusEvent, TransactionEvent, UpdateAccountEvent,
    },
    log::error,
    prost::Message,
    rdkafka::{
        error::KafkaError,
        producer::{BaseRecord, Producer, ThreadedProducer},
    },
    std::time::Duration,
};

pub struct KafkaPublisher {
    pub(crate) env: String,
    producer: ThreadedProducer<StatsThreadedProducerContext>,
    cluster: Cluster,
    shutdown_timeout: Duration,

    update_account_topic: String,
    update_account_topic_overrides: HashMap<Vec<u8>, String>,
    slot_status_topic: String,
    transaction_topic: String,

    wrap_messages: bool,
}

impl KafkaPublisher {
    pub fn new(
        producer: ThreadedProducer<StatsThreadedProducerContext>,
        config: &Config,
        env: String,
    ) -> Self {
        Self {
            env,
            cluster: config.cluster.clone(),
            producer,
            shutdown_timeout: Duration::from_millis(config.shutdown_timeout_ms),
            update_account_topic: config.update_account_topic.clone(),
            update_account_topic_overrides: config.update_topic_overrides_by_account(),
            slot_status_topic: config.slot_status_topic.clone(),
            transaction_topic: config.transaction_topic.clone(),
            wrap_messages: config.wrap_messages,
        }
    }

    pub fn update_account(&self, ev: UpdateAccountEvent) -> Result<(), KafkaError> {
        let topic = self
            .update_account_topic_overrides
            .get(&ev.owner)
            .unwrap_or(&self.update_account_topic);

        let (key, buf) = Self::account_update_key_and_data(ev, &self.cluster, self.wrap_messages);
        let record = BaseRecord::<Vec<u8>, _>::to(topic).key(&key).payload(&buf);
        let result = self.producer.send(record).map(|_| ()).map_err(|(e, _)| e);
        UPLOAD_ACCOUNTS_TOTAL
            .with_label_values(&[if result.is_ok() { "success" } else { "failed" }])
            .inc();
        result
    }

    pub fn update_slot_status(&self, ev: SlotStatusEvent) -> Result<(), KafkaError> {
        let temp_key;
        let (key, buf) = if self.wrap_messages {
            temp_key = Self::copy_and_prepend(&ev.slot.to_le_bytes(), 83u8);
            (&temp_key, Self::encode_with_wrapper(Slot(Box::new(ev))))
        } else {
            temp_key = ev.slot.to_le_bytes().to_vec();
            (&temp_key, ev.encode_to_vec())
        };
        let record = BaseRecord::<Vec<u8>, _>::to(&self.slot_status_topic)
            .key(key)
            .payload(&buf);
        let result = self.producer.send(record).map(|_| ()).map_err(|(e, _)| e);
        UPLOAD_SLOTS_TOTAL
            .with_label_values(&[if result.is_ok() { "success" } else { "failed" }])
            .inc();
        result
    }

    pub fn update_transaction(&self, ev: TransactionEvent) -> Result<(), KafkaError> {
        let temp_key;
        let (key, buf) = if self.wrap_messages {
            temp_key = Self::copy_and_prepend(ev.signature.as_slice(), 84u8);
            (
                &temp_key,
                Self::encode_with_wrapper(Transaction(Box::new(ev))),
            )
        } else {
            (&ev.signature, ev.encode_to_vec())
        };
        let record = BaseRecord::<Vec<u8>, _>::to(&self.transaction_topic)
            .key(key)
            .payload(&buf);
        let result = self.producer.send(record).map(|_| ()).map_err(|(e, _)| e);
        UPLOAD_TRANSACTIONS_TOTAL
            .with_label_values(&[if result.is_ok() { "success" } else { "failed" }])
            .inc();
        result
    }

    pub fn wants_update_account(&self) -> bool {
        !self.update_account_topic.is_empty()
    }

    pub fn wants_slot_status(&self) -> bool {
        !self.slot_status_topic.is_empty()
    }

    pub fn wants_transaction(&self) -> bool {
        !self.transaction_topic.is_empty()
    }

    fn encode_with_wrapper(message: EventMessage) -> Vec<u8> {
        MessageWrapper {
            event_message: Some(message),
        }
        .encode_to_vec()
    }

    fn account_update_key_and_data(
        ev: UpdateAccountEvent,
        cluster: &Cluster,
        wrap_messages: bool,
    ) -> (Vec<u8>, Vec<u8>) {
        if wrap_messages {
            let key = Self::account_update_key(cluster, &ev.owner);
            let key = Self::copy_and_prepend(key.as_bytes(), 65u8);
            let data = Self::encode_with_wrapper(Account(Box::new(ev)));
            (key, data)
        } else {
            let key = Self::account_update_key(cluster, &ev.owner);
            let key = key.as_bytes().to_vec();
            let data = ev.encode_to_vec();
            (key, data)
        }
    }

    fn copy_and_prepend(data: &[u8], prefix: u8) -> Vec<u8> {
        let mut temp_key = Vec::with_capacity(data.len() + 1);
        temp_key.push(prefix);
        temp_key.extend_from_slice(data);
        temp_key
    }

    fn account_update_key(cluster: &Cluster, owner: &[u8]) -> String {
        // SAFETY: we don't expect the RPC to provide us invalid pubkeys ever
        format!("{}:{}", cluster, Pubkey::try_from(owner).unwrap())
    }
}

impl Drop for KafkaPublisher {
    fn drop(&mut self) {
        if let Err(e) = self.producer.flush(self.shutdown_timeout) {
            error!("Failed to flush producer: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const PK: &str = "A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz";
    fn event_with_owner(owner: &Pubkey) -> UpdateAccountEvent {
        UpdateAccountEvent {
            slot: 9,
            pubkey: Pubkey::new_unique().to_bytes().to_vec(),
            lamports: 100,
            owner: owner.to_bytes().to_vec(),
            executable: false,
            rent_epoch: 0,
            data: "account data".as_bytes().to_vec(),
            write_version: 1,
            txn_signature: None,
        }
    }

    #[test]
    fn account_update_key_and_data_no_wrap() {
        fn check(cluster: Cluster, expected_key: &str) {
            let owner = Pubkey::from_str(PK).unwrap();
            let ev = event_with_owner(&owner);
            let (key, data) =
                KafkaPublisher::account_update_key_and_data(ev.clone(), &cluster, false);
            let key = String::from_utf8_lossy(key.as_slice());

            let mut bytes = data.as_slice();
            let decoded = UpdateAccountEvent::decode(&mut bytes).unwrap();

            assert_eq!(key, expected_key);
            assert_eq!(decoded, ev);
        }

        check(
            Cluster::Mainnet,
            "mainnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
        check(
            Cluster::Devnet,
            "devnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
        check(
            Cluster::Testnet,
            "testnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
    }

    #[test]
    fn account_update_key_and_data_wrap() {
        fn check(cluster: Cluster, expected_key: &str) {
            let owner = Pubkey::from_str(PK).unwrap();
            let ev = event_with_owner(&owner);
            let wrapped = MessageWrapper {
                event_message: Some(EventMessage::Account(Box::new(ev.clone()))),
            };

            let (key, data) = KafkaPublisher::account_update_key_and_data(ev, &cluster, true);

            assert_eq!(key[0], 65u8);
            let key = key.into_iter().skip(1).collect::<Vec<_>>();
            let key = String::from_utf8_lossy(key.as_slice());

            let mut bytes = data.as_slice();
            let decoded = MessageWrapper::decode(&mut bytes).unwrap();

            assert_eq!(key, expected_key);
            assert_eq!(decoded, wrapped);
        }

        check(
            Cluster::Mainnet,
            "mainnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
        check(
            Cluster::Devnet,
            "devnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
        check(
            Cluster::Testnet,
            "testnet:A15Y2eoMNGeX4516TYTaaMErwabCrf9AB9mrzFohdQJz",
        );
    }
}

use crate::{
    sanitized_message::MessagePayload, CompiledInstruction, InnerInstruction, InnerInstructions,
    LegacyLoadedMessage, LegacyMessage, LoadedAddresses, MessageAddressTableLookup, MessageHeader,
    Reward, SanitizedMessage, SanitizedTransaction, SlotStatusEvent, TransactionEvent,
    TransactionStatusMeta, TransactionTokenBalance, UiTokenAmount, UpdateAccountEvent,
    V0LoadedMessage, V0Message,
};
use serde::Serialize;

// -----------------
// UpdateAccountEvent
// -----------------
#[derive(Debug, Serialize)]
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
// SlotStatusEvent
// -----------------
#[derive(Debug, Serialize)]
pub enum SerializableSlotStatus {
    Processed,
    Rooted,
    Confirmed,
}

impl From<i32> for SerializableSlotStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Processed,
            1 => Self::Rooted,
            2 => Self::Confirmed,
            _ => panic!("Invalid slot status"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableSlotStatusEvent {
    slot: u64,
    parent: u64,
    status: SerializableSlotStatus,
}

impl From<SlotStatusEvent> for SerializableSlotStatusEvent {
    fn from(ev: SlotStatusEvent) -> Self {
        Self {
            slot: ev.slot,
            parent: ev.parent,
            status: SerializableSlotStatus::from(ev.status),
        }
    }
}

// -----------------
// TransactionEvent
// -----------------
#[derive(Debug, Serialize)]
pub struct SerializableMessageHeader {
    pub num_required_signatures: u32,
    pub num_readonly_signed_accounts: u32,
    pub num_readonly_unsigned_accounts: u32,
}

impl From<MessageHeader> for SerializableMessageHeader {
    fn from(x: MessageHeader) -> Self {
        Self {
            num_required_signatures: x.num_required_signatures,
            num_readonly_signed_accounts: x.num_readonly_signed_accounts,
            num_readonly_unsigned_accounts: x.num_readonly_unsigned_accounts,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableCompiledInstruction {
    pub program_id_index: u32,
    pub accounts: Vec<u32>,
    pub data: Vec<u8>,
}

impl From<CompiledInstruction> for SerializableCompiledInstruction {
    fn from(x: CompiledInstruction) -> Self {
        Self {
            program_id_index: x.program_id_index,
            accounts: x.accounts,
            data: x.data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableLoadedAddresses {
    pub writable: Vec<Vec<u8>>,
    pub readonly: Vec<Vec<u8>>,
}

impl From<LoadedAddresses> for SerializableLoadedAddresses {
    fn from(x: LoadedAddresses) -> Self {
        Self {
            writable: x.writable,
            readonly: x.readonly,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableMessageAddressTableLookup {
    pub account_key: Vec<u8>,
    pub writable_indexes: Vec<u32>,
    pub readonly_indexes: Vec<u32>,
}

impl From<MessageAddressTableLookup> for SerializableMessageAddressTableLookup {
    fn from(x: MessageAddressTableLookup) -> Self {
        Self {
            account_key: x.account_key,
            writable_indexes: x.writable_indexes,
            readonly_indexes: x.readonly_indexes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableV0Message {
    pub header: Option<SerializableMessageHeader>,
    pub account_keys: Vec<Vec<u8>>,
    pub recent_block_hash: Vec<u8>,
    pub instructions: Vec<SerializableCompiledInstruction>,
    pub address_table_lookup: Vec<SerializableMessageAddressTableLookup>,
}

impl From<V0Message> for SerializableV0Message {
    fn from(x: V0Message) -> Self {
        Self {
            header: x.header.map(SerializableMessageHeader::from),
            account_keys: x.account_keys,
            recent_block_hash: x.recent_block_hash,
            instructions: x
                .instructions
                .into_iter()
                .map(SerializableCompiledInstruction::from)
                .collect(),
            address_table_lookup: x
                .address_table_lookup
                .into_iter()
                .map(SerializableMessageAddressTableLookup::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableV0LoadedMessage {
    pub message: Option<SerializableV0Message>,
    pub loaded_adresses: Option<SerializableLoadedAddresses>,
    pub is_writable_account_cache: Vec<bool>,
}

impl From<V0LoadedMessage> for SerializableV0LoadedMessage {
    fn from(x: V0LoadedMessage) -> Self {
        Self {
            message: x.message.map(SerializableV0Message::from),
            loaded_adresses: x.loaded_adresses.map(SerializableLoadedAddresses::from),
            is_writable_account_cache: x.is_writable_account_cache,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableLegacyMessage {
    pub header: Option<SerializableMessageHeader>,
    pub account_keys: Vec<Vec<u8>>,
    pub recent_block_hash: Vec<u8>,
    pub instructions: Vec<SerializableCompiledInstruction>,
}

impl From<LegacyMessage> for SerializableLegacyMessage {
    fn from(x: LegacyMessage) -> Self {
        Self {
            header: x.header.map(SerializableMessageHeader::from),
            account_keys: x.account_keys,
            recent_block_hash: x.recent_block_hash,
            instructions: x
                .instructions
                .into_iter()
                .map(SerializableCompiledInstruction::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableLegacyLoadedMessage {
    pub message: Option<SerializableLegacyMessage>,
    pub is_writable_account_cache: Vec<bool>,
}

impl From<LegacyLoadedMessage> for SerializableLegacyLoadedMessage {
    fn from(x: LegacyLoadedMessage) -> Self {
        Self {
            message: x.message.map(SerializableLegacyMessage::from),
            is_writable_account_cache: x.is_writable_account_cache,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum SerializableMessagePayload {
    Legacy(SerializableLegacyLoadedMessage),
    V0(SerializableV0LoadedMessage),
}

impl From<MessagePayload> for SerializableMessagePayload {
    fn from(x: MessagePayload) -> Self {
        match x {
            MessagePayload::Legacy(x) => Self::Legacy(SerializableLegacyLoadedMessage::from(x)),
            MessagePayload::V0(x) => Self::V0(SerializableV0LoadedMessage::from(x)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableSanitizedMessage {
    pub message_payload: Option<SerializableMessagePayload>,
}

impl From<SanitizedMessage> for SerializableSanitizedMessage {
    fn from(x: SanitizedMessage) -> Self {
        Self {
            message_payload: x.message_payload.map(SerializableMessagePayload::from),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableSanitizedTransaction {
    pub message: Option<SerializableSanitizedMessage>,
    pub message_hash: Vec<u8>,
    pub is_simple_vote_transaction: bool,
    pub signatures: Vec<Vec<u8>>,
}

impl From<SanitizedTransaction> for SerializableSanitizedTransaction {
    fn from(x: SanitizedTransaction) -> Self {
        Self {
            message: x.message.map(SerializableSanitizedMessage::from),
            message_hash: x.message_hash,
            is_simple_vote_transaction: x.is_simple_vote_transaction,
            signatures: x.signatures,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableInnerInstruction {
    pub instruction: Option<SerializableCompiledInstruction>,
    pub stack_height: Option<u32>,
}

impl From<InnerInstruction> for SerializableInnerInstruction {
    fn from(x: InnerInstruction) -> Self {
        Self {
            instruction: x.instruction.map(SerializableCompiledInstruction::from),
            stack_height: x.stack_height,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableInnerInstructions {
    pub index: u32,
    pub instructions: Vec<SerializableInnerInstruction>,
}

impl From<InnerInstructions> for SerializableInnerInstructions {
    fn from(x: InnerInstructions) -> Self {
        Self {
            index: x.index,
            instructions: x
                .instructions
                .into_iter()
                .map(SerializableInnerInstruction::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableUiTokenAmount {
    pub ui_amount: Option<f64>,
    pub decimals: u32,
    pub amount: String,
    pub ui_amount_string: String,
}

impl From<UiTokenAmount> for SerializableUiTokenAmount {
    fn from(x: UiTokenAmount) -> Self {
        Self {
            ui_amount: x.ui_amount,
            decimals: x.decimals,
            amount: x.amount,
            ui_amount_string: x.ui_amount_string,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableTransactionTokenBalance {
    pub account_index: u32,
    pub mint: String,
    pub ui_token_account: Option<SerializableUiTokenAmount>,
    pub owner: String,
}

impl From<TransactionTokenBalance> for SerializableTransactionTokenBalance {
    fn from(x: TransactionTokenBalance) -> Self {
        Self {
            account_index: x.account_index,
            mint: x.mint,
            ui_token_account: x.ui_token_account.map(SerializableUiTokenAmount::from),
            owner: x.owner,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableReward {
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: i32,
    pub commission: u32,
}

impl From<Reward> for SerializableReward {
    fn from(x: Reward) -> Self {
        Self {
            pubkey: x.pubkey,
            lamports: x.lamports,
            post_balance: x.post_balance,
            reward_type: x.reward_type,
            commission: x.commission,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableTransactionStatusMeta {
    pub is_status_err: bool,
    pub error_info: String,
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub inner_instructions: Vec<SerializableInnerInstructions>,
    pub log_messages: Vec<String>,
    pub pre_token_balances: Vec<SerializableTransactionTokenBalance>,
    pub post_token_balances: Vec<SerializableTransactionTokenBalance>,
    pub rewards: Vec<SerializableReward>,
}

impl From<TransactionStatusMeta> for SerializableTransactionStatusMeta {
    fn from(x: TransactionStatusMeta) -> Self {
        Self {
            is_status_err: x.is_status_err,
            error_info: x.error_info,
            fee: x.fee,
            pre_balances: x.pre_balances,
            post_balances: x.post_balances,
            inner_instructions: x
                .inner_instructions
                .into_iter()
                .map(SerializableInnerInstructions::from)
                .collect(),
            log_messages: x.log_messages,
            pre_token_balances: x
                .pre_token_balances
                .into_iter()
                .map(SerializableTransactionTokenBalance::from)
                .collect(),
            post_token_balances: x
                .post_token_balances
                .into_iter()
                .map(SerializableTransactionTokenBalance::from)
                .collect(),
            rewards: x
                .rewards
                .into_iter()
                .map(SerializableReward::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SerializableTransactionEvent {
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub transaction: Option<SerializableSanitizedTransaction>,
    pub transaction_status_meta: Option<SerializableTransactionStatusMeta>,
    pub slot: u64,
    pub index: u64,
}

impl From<TransactionEvent> for SerializableTransactionEvent {
    fn from(x: TransactionEvent) -> Self {
        Self {
            signature: x.signature,
            is_vote: x.is_vote,
            transaction: x.transaction.map(SerializableSanitizedTransaction::from),
            transaction_status_meta: x
                .transaction_status_meta
                .map(SerializableTransactionStatusMeta::from),
            slot: x.slot,
            index: x.index,
        }
    }
}

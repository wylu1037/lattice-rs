use serde::{Deserialize, Serialize};

/// 回执
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Receipt {
    #[serde(rename = "contractAddress")]
    contract_address: String,
    #[serde(rename = "contractRet")]
    contract_return: String,
    #[serde(rename = "dblockHash")]
    daemon_block_hash: String,
    #[serde(rename = "dblockNumber")]
    dblock_height: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<Event>>,
    #[serde(rename = "jouleUsed")]
    joule_used: u64,
    #[serde(rename = "receiptIndex")]
    receipt_index: i32,
    success: bool,
    #[serde(rename = "tblockHash")]
    tblock_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "confirmTime")]
    confirm_time: Option<u64>,
    version: u16,
}

/// 事件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    address: String, // address of the contract that generated the event
    topics: Vec<String>,// list of topics provided by the contract
    data: Vec<u8>, // supplied by the contract, usually ABI-encoded
    #[serde(rename = "logIndex")]
    index: u32, // index of the log in the block
    #[serde(rename = "dblockNumber")]
    daemon_block_height: u64,
    removed: bool,
    #[serde(rename = "dataHex")]
    data_hex: String,
}
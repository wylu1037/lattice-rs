use serde::{Deserialize, Serialize};

/// 回执
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Receipt {
    contract_address: String,
    contract_ret: String,
    dblock_hash: String,
    dblock_number: u64,
    events: Vec<Event>,
    joule_used: u64,
    receipt_index: i32,
    success: bool,
    tblock_hash: String,
    confirm_time: u64,
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
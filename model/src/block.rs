use serde::{Deserialize, Serialize};

/// 最新的账户区块和守护区块信息
pub struct CurrentTDBlock {
    current_dblock_hash: String,
    current_tblock_hash: String,
    current_tblock_number: u64,
}

/// 账户区块
pub struct TBlock {
    number: u64,
    parent_hash: String,
    daemon_hash: String,
}

/// 交易
pub struct Transaction {
    number: u64,
    parent_hash: String,
    daemon_hash: String,
    payload: String,
    hub: Vec<String>,
    timestamp: u64,
    tx_type: String,
    owner: String,
    linker: String,
    code: String,
    amount: String,
    joule: u64,
    sign: String,
    proof_of_work: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBlock {
    #[serde(rename = "parentHash")]
    parent_hash: String,
    #[serde(rename = "number")]
    height: u64,
    timestamp: u64,
    version: u8,
}
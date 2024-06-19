use serde::{Deserialize, Serialize};

/// 最新的账户区块和守护区块信息
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTDBlock {
    /// 最新的守护区块
    #[serde(rename = "currentDBlockHash")]
    pub current_dblock_hash: String,
    /// 最新的账户区块
    #[serde(rename = "currentTBlockHash")]
    pub current_tblock_hash: String,
    /// 最新的账户区块高度
    #[serde(rename = "currentTBlockNumber")]
    pub current_tblock_height: u64,
}

/// 账户区块
#[derive(Serialize, Deserialize, Debug)]
pub struct TBlock {
    #[serde(rename = "number")]
    height: u64,
    parent_hash: String,
    daemon_hash: String,
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
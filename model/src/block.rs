use serde::{Deserialize, Serialize};

/// 最新的账户区块和守护区块信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LatestBlock {
    /// 最新的账户区块高度
    #[serde(rename = "currentTBlockNumber")]
    pub height: u64,
    /// 最新的账户区块
    #[serde(rename = "currentTBlockHash")]
    pub hash: String,
    /// 最新的守护区块
    #[serde(rename = "currentDBlockHash")]
    pub daemon_hash: String,
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
    pub hash: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    #[serde(rename = "number")]
    pub height: u64,
    pub timestamp: u64,
    pub version: u8,
}
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
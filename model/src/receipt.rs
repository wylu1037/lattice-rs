/// 回执
pub struct Receipt {
    contract_address: String,
    contract_ret: String,
    dblock_hash: String,
    dblock_number: u64,
    events: Vec<Box<dyn std::any::Any>>,
    joule_used: u64,
    receipt_index: i32,
    success: bool,
    tblock_hash: String,
    confirm_time: u64,
}
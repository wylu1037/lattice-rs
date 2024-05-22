use std::any::Any;
use std::collections::HashMap;

/// 回执
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
pub struct Event {
    name: String,
    fields: HashMap<String, Box<dyn Any>>,
}
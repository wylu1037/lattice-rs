use rlp::RlpStream;
use serde::{Deserialize, Serialize};

use crate::common::Address;
use crate::constants::{ZERO_HASH_STRING, ZERO_ZLTC_ADDRESS};
use crate::convert::number_to_vec;
use crate::enums::Cryptography;
use crate::HexString;

/// 最新的账户区块和守护区块信息
pub struct CurrentTDBlock {
    current_dblock_hash: String,
    current_tblock_hash: String,
    current_tblock_number: u64,
}

/// 账户区块
#[derive(Serialize, Deserialize, Debug)]
pub struct TBlock {
    #[serde(rename = "number")]
    height: u64,
    parent_hash: String,
    daemon_hash: String,
}

/// 交易
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "number")]
    pub height: u64,
    pub parent_hash: String,
    pub daemon_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub: Option<Vec<String>>,
    pub timestamp: u64,
    pub tx_type: String,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joule: Option<u128>,
    pub sign: String,
    pub proof_of_work: String,
}

impl Transaction {
    fn rlp_encode(&self, code_hash: String, pow: String, chain_id: u32, cryptography: Cryptography, use_pow: bool, is_sign: bool) -> Vec<u8> {
        let mut rlp = RlpStream::new();
        rlp.begin_list(15 + if is_sign { 2 } else { 0 });

        let parent_hash = HexString::new(&self.parent_hash.as_str()).decode();
        let daemon_hash = HexString::new(&self.daemon_hash.as_str()).decode();
        let hub = match &self.hub {
            None => { vec![] }
            Some(v) => { v.to_vec() }
        };
        let hub_arr = hub
            .into_iter()
            .map(|s| HexString::new(s.as_str()).decode())
            .collect::<Vec<Vec<u8>>>();
        let owner_address = HexString::new(Address::new(&self.owner).to_ethereum_address().as_str()).decode();
        //let linker_address = HexString::new(Address::new(&self.linker).to_ethereum_address().as_str()).decode();
        let linker_address = match &self.linker {
            None => HexString::new(Address::new(ZERO_ZLTC_ADDRESS).to_zltc_address().as_str()).decode(),
            Some(v) => HexString::new(Address::new(v).to_ethereum_address().as_str()).decode()
        };
        let code_hash = match &self.code {
            None => ZERO_HASH_STRING[2..].to_string(),
            Some(v) => {
                let bytes = HexString::new(v).decode();
                ha
            }
        };

        rlp.append(&number_to_vec(self.height));
        rlp.out().to_vec()
    }
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
use std::ops::Shl;

use num_bigint::BigUint;
use rlp::RlpStream;
use serde::{Deserialize, Serialize, Serializer};

use model::{Cryptography, HexString};
use model::common::Address;
use model::constants::{ZERO_HASH_STRING, ZERO_ZLTC_ADDRESS};
use model::convert::{number_to_vec, option_number_to_vec};

use crate::hash::hash_message;
use crate::sign::KeyPair;

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
    #[serde(rename = "type")]
    pub tx_type: TxType,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(rename = "codeHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub joule: Option<u128>,
    pub sign: String,
    pub proof_of_work: String,
    pub version: TxVersion,
}

#[derive(Deserialize, Debug)]
pub enum TxType {
    Genesis,
    Create,
    Send,
    Receive,
    Contract,
    Execute,
    Update,
}

impl TxType {
    fn to_vec(&self) -> Vec<u8> {
        match &self {
            TxType::Genesis => vec![0x00],
            TxType::Create => vec![0x01],
            TxType::Send => vec![0x02],
            TxType::Receive => vec![0x03],
            TxType::Contract => vec![0x04],
            TxType::Execute => vec![0x05],
            TxType::Update => vec![0x06],
        }
    }

    pub fn name(&self) -> String {
        match &self {
            TxType::Genesis => "genesis".to_string(),
            TxType::Create => "create".to_string(),
            TxType::Send => "send".to_string(),
            TxType::Receive => "receive".to_string(),
            TxType::Contract => "contract".to_string(),
            TxType::Execute => "execute".to_string(),
            TxType::Update => "update".to_string(),
        }
    }
}

impl Serialize for TxType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = match &self {
            TxType::Genesis => "genesis",
            TxType::Create => "create",
            TxType::Send => "send",
            TxType::Receive => "receive",
            TxType::Contract => "contract",
            TxType::Execute => "execute",
            TxType::Update => "update",
        };
        serializer.serialize_str(s)
    }
}

#[derive(Deserialize, Debug)]
pub enum TxVersion {
    /// 混沌-0
    Chaos,
    /// 盘古-1
    PanGu,
    /// 女娲-2
    NuWa,
    /// 最新-3
    Latest,
}

impl TxVersion {
    pub fn ordinal(&self) -> u16 {
        match &self {
            Self::Chaos => 0,
            Self::PanGu => 1,
            Self::NuWa => 2,
            Self::Latest => 3,
        }
    }
}

impl Serialize for TxVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let ordinal = self.ordinal();
        serializer.serialize_u16(ordinal)
    }
}

const DIFFICULTY_BYTE_ARRAY: Vec<u8> = vec![];
const POW_BYTE_ARRAY: Vec<u8> = vec![];
const DIFFICULTY: usize = 12;

impl Transaction {
    /// # 创建空交易
    ///
    /// ## 入参
    ///
    /// ## 出参
    /// + `Transaction`
    pub fn empty_tx() -> Self {
        Transaction {
            height: 0,
            parent_hash: String::new(),
            daemon_hash: String::new(),
            payload: None,
            hub: None,
            timestamp: 0,
            tx_type: TxType::Genesis,
            owner: String::new(),
            linker: None,
            code: None,
            code_hash: None,
            amount: None,
            joule: None,
            sign: String::new(),
            proof_of_work: String::new(),
            version: TxVersion::Latest,
        }
    }

    /// # RLP编码
    /// ## 入参
    /// + `chain_id: u64`: 区块链id
    /// + `pow: String`
    /// + `cryptography: Cryptography`: Secp256k or Sm2p256v1
    /// + `use_pow: bool`
    /// + `is_sign: bool`
    ///
    /// ## 出参
    /// + `Vec<u8>`
    fn rlp_encode(&mut self, chain_id: u64, pow: String, cryptography: Cryptography, use_pow: bool, is_sign: bool) -> Vec<u8> {
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
                hash_message(&bytes, cryptography)
            }
        };
        self.set_code_hash(format!("0x{}", code_hash)); // update transaction code_hash
        let code_hash = HexString::new(code_hash.as_str()).decode();
        let payload = match &self.payload {
            None => vec![],
            Some(v) => HexString::new(v).decode()
        };

        rlp.append(&number_to_vec(self.height));
        rlp.append(&self.tx_type.to_vec());
        rlp.append(&parent_hash);
        rlp.append_list::<Vec<u8>, Vec<u8>>(&hub_arr);
        rlp.append(&daemon_hash);
        rlp.append(&code_hash);
        rlp.append(&owner_address);
        rlp.append(&linker_address);
        rlp.append(&option_number_to_vec(self.amount));
        rlp.append(&option_number_to_vec(self.joule));
        if use_pow {
            rlp.append(&HexString::new(pow.as_str()).decode());
        } else {
            rlp.append(&DIFFICULTY_BYTE_ARRAY);
            rlp.append(&POW_BYTE_ARRAY);
        }
        rlp.append(&payload);
        rlp.append(&number_to_vec(self.timestamp));
        rlp.append(&number_to_vec(chain_id));
        if is_sign {
            rlp.append(&vec![]);
            rlp.append(&vec![]);
        }

        rlp.out().to_vec()
    }

    /// # 计算pow
    /// ## 入参
    /// + `chain_id: u64`: 区块链id
    /// + `cryptography: Cryptography`: Secp256k or Sm2p256v1
    ///
    /// ## 出参
    /// + `BigUint`: pow
    #[allow(dead_code)]
    fn pow(&mut self, chain_id: u64, cryptography: Cryptography) -> BigUint {
        let mut i: u32 = 0;
        let min: BigUint = BigUint::from(1u32).shl(256 - DIFFICULTY);

        loop {
            i = i + 1;
            let pow = BigUint::from(i);
            let rlp = self.rlp_encode(chain_id, hex::encode(&pow.to_bytes_be()), cryptography, true, false);
            let hash = hash_message(&rlp, cryptography);
            let bytes = HexString::new(hash.as_str()).decode();
            let calculated = BigUint::from_bytes_be(&bytes);
            if calculated.le(&min) {
                return pow;
            }
        }
    }

    /// # encode
    /// ## 入参
    /// + `chain_id: u64`: 区块链id
    /// + `cryptography: Cryptography`: Secp256k or Sm2p256v1
    ///
    /// ## 出参
    /// + `BigUint`
    /// + `Vec<u8>`
    fn encode(&mut self, chain_id: u64, cryptography: Cryptography) -> (BigUint, Vec<u8>) {
        // let pow = self.pow(chain_id, cryptography);
        let pow = BigUint::from_bytes_be(HexString::new("0x00").decode().as_slice());
        let code = self.rlp_encode(chain_id, hex::encode(&pow.to_bytes_be()), cryptography, false, true);
        (pow, code)
    }

    /// # 签名交易
    /// ## 入参
    /// + `chain_id: u64`: 区块链id
    /// + `sk: &[u8]`: 私钥
    /// + `cryptography: Cryptography`: Secp256k or Sm2p256v1
    ///
    /// ## 出参
    /// + `BigUint`: pow
    /// + `String`: signature
    pub fn sign(&mut self, chain_id: u64, sk: &[u8], cryptography: Cryptography) -> (BigUint, String) {
        let key_pair = KeyPair::from_secret_key(sk, cryptography);

        let (pow, encoded) = self.encode(chain_id, cryptography);
        let hash = hash_message(&encoded, cryptography);
        let data = HexString::new(hash.as_str()).decode();
        let signature = key_pair.sign(&data);
        self.sign = signature;

        (pow, self.sign.to_string())
    }

    pub fn to_raw_tx(self) -> RawTransaction {
        RawTransaction {
            height: self.height,
            parent_hash: self.parent_hash,
            daemon_hash: self.daemon_hash,
            timestamp: self.timestamp,
            owner: self.owner,
            linker: self.linker.unwrap(),
            ty: self.tx_type.name(),
            hub: self.hub.unwrap_or(vec![]),
            code: self.code.unwrap_or(String::new()),
            code_hash: self.code_hash,
            payload: self.payload.unwrap_or(String::from("0x")),
            amount: self.amount.unwrap_or(0),
            joule: self.joule.unwrap_or(0),
            sign: self.sign,
            proof_of_work: self.proof_of_work,
            version: self.version.ordinal(),
            difficulty: 0,
        }
    }

    pub fn set_code_hash(&mut self, code_hash: String) {
        self.code_hash = Some(code_hash.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawTransaction {
    #[serde(rename = "number")]
    pub height: u64,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    #[serde(rename = "daemonHash")]
    pub daemon_hash: String,
    pub timestamp: u64,
    pub owner: String,
    pub linker: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub hub: Vec<String>,
    pub code: String,
    #[serde(rename = "codeHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_hash: Option<String>,
    pub payload: String,
    pub amount: u128,
    pub joule: u128,
    pub sign: String,
    #[serde(rename = "proofOfWork")]
    pub proof_of_work: String,
    pub version: u16,
    pub difficulty: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sign_tx() {}
}
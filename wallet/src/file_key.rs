use serde::{Deserialize, Serialize};

/// aes
#[derive(Serialize, Deserialize, Debug)]
pub struct Aes {
    /// 密码算法：aes-128-ctr
    pub cipher: String,
    // 初始化向量：1ad693b4d8089da0492b9c8c49bc60d3
    pub iv: String,
}

/// 密钥派生函数，Key Derivation Function。用于从原始密钥材料（如密码、随机数或其它原始数据）中生成一个或多个密钥的函数。
#[derive(Serialize, Deserialize, Debug)]
pub struct Kdf {
    /// scrypt, PBKDF2, bcrypt, HKDF
    pub kdf: String,
    #[serde(rename = "kdfParams")]
    pub kdf_params: KdfParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KdfParams {
    /// 生成的密钥长度，单位byte
    #[serde(rename = "DKLen")]
    pub dk_len: u32,
    /// CPU/内存成本因子，控制计算和内存的使用量。
    pub n: u32,
    /// 并行度因子，控制 scrypt 函数的并行度。
    pub p: u32,
    /// 块大小因子，影响内部工作状态和内存占用。
    pub r: u32,
    /// 盐值，在密钥派生过程中加入随机性。
    pub salt: String,
}
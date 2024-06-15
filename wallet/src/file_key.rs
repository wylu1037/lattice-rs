use rand::random;
use scrypt::{Params, Scrypt};
use scrypt::password_hash::{PasswordHasher, SaltString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crypto::aes;
use crypto::hash::hash_message;
use crypto::sign::KeyPair;
use model::Cryptography;
use model::HexString;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileKey {
    pub uuid: String,
    pub address: String,
    pub cipher: Cipher,
    #[serde(rename = "isGM")]
    pub is_gm: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cipher {
    pub aes: Aes,
    pub kdf: Kdf,
    #[serde(rename = "cipherText")]
    pub cipher_text: String,
    /// Message Authentication Code（消息认证码）
    pub mac: String,
}

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

impl FileKey {
    /// # 从私钥得到FileKey
    /// ## Parameters
    /// + `secret_key: &[u8]`: 私钥
    /// + `password: &[u8]`: 身份密码
    /// + `cryptography: Cryptography`: Secp256k1 or Sm2p256v1
    ///
    /// ## Returns
    /// + FileKey
    fn from_secret_key(secret_key: &[u8], password: &[u8], cryptography: Cryptography) -> Self {
        let key_pair = KeyPair::from_secret_key(secret_key, cryptography);
        FileKey {
            uuid: Uuid::new_v4().to_string(),
            address: key_pair.address(),
            cipher: gen_cipher(secret_key, password, cryptography),
            is_gm: matches!(cryptography, Cryptography::Sm2p256v1),
        }
    }
}

/// # 生成 Cipher
/// ## Parameters
/// + `secret_key: &[u8]`: 私钥
/// + `password: &[u8]`: 密码
/// + `cryptography: Cryptography`:
///
/// ## Returns
/// + `Cipher`: struct
fn gen_cipher(secret_key: &[u8], password: &[u8], cryptography: Cryptography) -> Cipher {
    let salt = hex::encode(random::<[u8; 32]>());
    let iv_bytes = random::<[u8; 16]>();
    let iv = hex::encode(iv_bytes);
    let key = scrypt_key(password, &salt);
    let aes_key = hex::decode(&key[0..32]).unwrap();
    let hash_key = hex::decode(&key[32..64]).unwrap();
    let cipher_text = aes::encrypt(&secret_key, &aes_key, &iv_bytes);
    let mac = compute_mac(&hash_key, &cipher_text, cryptography);
    Cipher {
        aes: Aes {
            cipher: "aes-128-ctr".to_string(),
            iv,
        },
        kdf: Kdf {
            kdf: "scrypt".to_string(),
            kdf_params: KdfParams {
                dk_len: 32,
                n: 262144,
                p: 1,
                r: 8,
                salt,
            },
        },
        cipher_text,
        mac,
    }
}

/// # 使用 Scrypt 算法生成一个基于输入密码和盐值的加密密钥
/// ## Parameters
/// + `password: &[u8]`: 密码
/// + `salt: &str`: 盐值
///
/// ## Returns
/// + `String`: 十六进制编码的 Scrypt 密钥
fn scrypt_key(password: &[u8], salt: &str) -> String {
    let h = HexString { hex_string: String::from(salt) };
    let salt_bytes = h.decode();
    let salt_str = SaltString::encode_b64(&salt_bytes).unwrap();
    let params = Params::new(18, 8, 1, 32).unwrap();
    let password_hash = Scrypt.hash_password_customized(password, None, None, params, &salt_str).unwrap();
    let scrypt_output = password_hash.hash.unwrap();
    hex::encode(scrypt_output.as_bytes())
}

/// # 计算Message Authentication Code（消息认证码）
/// ## Parameters
/// + `key: &[u8]`:
/// + `cipher_text: &str`:
/// + `cryptography: Cryptography`:
///
/// ## Returns
/// + String
fn compute_mac(key: &[u8], cipher_text: &str, cryptography: Cryptography) -> String {
    let h = HexString { hex_string: String::from(cipher_text) };
    let cipher_bytes = h.decode();
    let data = [key, &cipher_bytes].concat();
    hash_message(&data, cryptography)
}

#[cfg(test)]
mod tests {
    use model::Cryptography;
    use model::HexString;

    use crate::file_key::FileKey;

    #[test]
    fn test_gen_file_key() {
        let secret_key = HexString { hex_string: String::from("0x23d5b2a2eb0a9c8b86d62cbc3955cfd1fb26ec576ecc379f402d0f5d2b27a7bb") };
        let file_key = FileKey::from_secret_key(secret_key.decode().as_slice(), b"Root1234", Cryptography::Sm2p256v1);
        match serde_json::to_string(&file_key) {
            Ok(json_string) => println!("{}", json_string),
            Err(e) => eprintln!("{}", e)
        }
        let addr = "zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi";
        assert_eq!(file_key.address, addr)
    }
}
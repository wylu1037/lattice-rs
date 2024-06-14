use aes::Aes128Ctr;
use aes::cipher::{NewCipher, StreamCipher, StreamCipherSeek};

use model::HexString;

/// # Aes encrypt, 16 byte = 128 bit
/// ## Parameters
/// + `data: &[u8]`: 待对称加密的数据
/// + `key: &[u8]`: 密钥
/// + `iv: &[u8]`: 初始化向量
///
/// ## Returns
/// + String: cipher
pub fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> String {
    let mut cipher = Aes128Ctr::new_from_slices(key, iv).unwrap();
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    hex::encode(buffer)
}

/// # Aes decrypt,
/// ## Parameters
/// + `cipher_text: &str`
/// + `key: &[u8]`
/// + `iv: &[u8]`
///
/// ## Returns
/// + String: source
pub fn decrypt(cipher_text: &str, key: &[u8], iv: &[u8]) -> String {
    let mut cipher = Aes128Ctr::new_from_slices(key, iv).unwrap();
    let h = HexString { hex_string: String::from(cipher_text) };
    let mut buffer = h.decode();
    cipher.seek(0);
    cipher.apply_keystream(&mut buffer);
    hex::encode(buffer)
}
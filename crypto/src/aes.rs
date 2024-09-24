use aes::{Aes128, Aes128Ctr, BlockDecrypt, BlockEncrypt, NewBlockCipher};
use aes::cipher::{NewCipher, StreamCipher, StreamCipherSeek};
use aes::cipher::generic_array::GenericArray;

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

pub enum AesMode {
    CTR,
    ECB,
}

const AES_DEFAULT_IV: &[u8; 16] = b"0123456789abcdef";

/// # encrypt data with aes mode, and use pkcs7 padding,
/// # encryption key size fixed as 128 bit
///
/// ## 入参
/// + `mode: AesMode`: CTR or ECB
/// + `data &[u8]`: data that needs to be encrypted
/// + `key &[u8]`: secret key
/// + `iv: Option<&[u8]>`: initialization vector, is used by the CTR, and is useless for ECB.
///
/// ## 出参
/// + `String`: cipher
pub fn encrypt_with_mode(mode: AesMode, data: &[u8], key: &[u8], iv: Option<&[u8]>) -> String {
    match mode {
        AesMode::CTR => {
            let mut cipher = Aes128Ctr::new_from_slices(key, iv.unwrap_or(AES_DEFAULT_IV)).unwrap();
            let mut buffer = data.to_vec();
            cipher.apply_keystream(&mut buffer);
            hex::encode(buffer)
        }
        AesMode::ECB => {
            let cipher = Aes128::new_from_slice(key).unwrap();
            let mut buffer = data.to_vec();

            // Apply PKCS7 padding
            let pad_len = 16 - buffer.len() % 16;
            buffer.extend(vec![pad_len as u8; pad_len]);

            for chunk in buffer.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chunk);
                cipher.encrypt_block(block);
            }
            hex::encode(buffer)
        }
    }
}

/// # decrypt data with aes mode
///
/// ## 入参
/// + `mode: AesMode`: CTR or ECB
/// + `cipher_text: &str`: cipher
/// + `key: &[u8]`: secret key
/// + `iv: &[u8]`: initialization vector
///
/// ## 出参
/// + `String`: source
pub fn decrypt_with_mode(mode: AesMode, cipher_text: &str, key: &[u8], iv: Option<&[u8]>) -> String {
    match mode {
        AesMode::CTR => {
            let mut cipher = Aes128Ctr::new_from_slices(key, iv.unwrap_or(AES_DEFAULT_IV)).unwrap();
            let h = HexString { hex_string: String::from(cipher_text) };
            let mut buffer = h.decode();
            cipher.seek(0);
            cipher.apply_keystream(&mut buffer);
            hex::encode(buffer)
        }
        AesMode::ECB => {
            let cipher = Aes128::new_from_slice(key).unwrap();
            let h = HexString { hex_string: String::from(cipher_text) };
            let mut buffer = h.decode();

            for chuck in buffer.chunks_mut(16) {
                let block = GenericArray::from_mut_slice(chuck);
                cipher.decrypt_block(block);
            }

            // Remove PKCS7 padding
            let pad_len = *buffer.last().unwrap() as usize;
            if pad_len > 0 && pad_len <= 16 {
                buffer.truncate(buffer.len() - pad_len);
            }
            hex::encode(buffer)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encrypt_with_ecb() {
        let key = b"0123456789abcdef";
        let cipher_text = encrypt_with_mode(AesMode::ECB, b"hello world", key, None);
        let expected = "8169bed4ef49a8874559c5b200daade7";
        assert_eq!(expected, cipher_text);
    }

    #[test]
    fn decrypt_with_ecb() {
        let key = b"0123456789abcdef";
        let cipher_text = "8169bed4ef49a8874559c5b200daade7";
        let plain_text = decrypt_with_mode(AesMode::ECB, cipher_text, key, None);
        let expected = hex::encode(b"hello world");
        assert_eq!(expected, plain_text);
    }
}
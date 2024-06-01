use aes::Aes128Ctr;
use aes::cipher::{NewCipher, StreamCipher, StreamCipherSeek};

use crate::HexString;

pub fn encrypt(data: &[u8], key: &[u8], iv: &[u8]) -> String {
    let mut cipher = Aes128Ctr::new_from_slices(key, iv).unwrap();
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    hex::encode(buffer)
}

pub fn decrypt(cipher_text: &str, key: &[u8], iv: &[u8]) -> String {
    let mut cipher = Aes128Ctr::new_from_slices(key, iv).unwrap();
    let h = HexString { hex: String::from(cipher_text) };
    let mut buffer = h.decode();
    cipher.seek(0);
    cipher.apply_keystream(&mut buffer);
    hex::encode(buffer)
}
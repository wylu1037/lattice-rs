use libsm::sm3::hash::Sm3Hash;

use model::Curve;

/// # 哈希
/// ## 入参
/// + `message: &[u8]`: 消息
/// + `curve: Curve`: secp256k1时哈希算法为sha256, sm2p256v1时哈希算法为sm3
///
/// ## 出参
/// + `String`: 哈希字符串
pub fn hash_message(message: &[u8], curve: Curve) -> String {
    match curve {
        Curve::Secp256k1 => {
            sha256::digest(message)
        }
        Curve::Sm2p256v1 => {
            let mut hash = Sm3Hash::new(message);
            let digest = hash.get_hash().to_vec();
            hex::encode(digest)
        }
    }
}
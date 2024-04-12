use libsm::sm2::ecc::EccCtx;
use libsm::sm2::signature::SigCtx;
use libsm::sm3::hash::Sm3Hash;
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use secp256k1::{All, PublicKey, rand::rngs::OsRng, Secp256k1, SecretKey};

#[derive(Debug, Clone, Copy)]
pub enum Cryptography {
    /// 国际算法
    Secp256k1,
    /// 国密算法
    Sm2p256v1,
}

#[derive(Debug)]
pub struct KeyPair {
    /// 公钥，非压缩公钥，由1字节的前缀(标识y坐标的奇偶，0x02/0x03)+32字节的x坐标+32字节的y坐标
    pub public_key: Vec<u8>,
    /// 私钥，32字节
    pub secret_key: BigUint,
    /// 椭圆曲线，Secp256k1 or Sm2p256v1
    pub cryptography: Cryptography,
}

static CONTEXT_SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);
static CONTEXT_SM2P256V1: Lazy<SigCtx> = Lazy::new(SigCtx::new);
static CURVE_SM2P256V1: Lazy<EccCtx> = Lazy::new(EccCtx::new);

impl KeyPair {
    pub fn new_keypair(cryptography: Cryptography) -> KeyPair {
        match cryptography {
            Cryptography::Secp256k1 => {
                let mut rng = OsRng::default();
                let (secret_key, public_key) = CONTEXT_SECP256K1.generate_keypair(&mut rng);

                KeyPair {
                    public_key: public_key.serialize_uncompressed().to_vec(),
                    secret_key: BigUint::from_bytes_be(&secret_key.secret_bytes()),
                    cryptography,
                }
            }
            Cryptography::Sm2p256v1 => {
                let (public_key, secret_key) = CONTEXT_SM2P256V1.new_keypair().expect("new keypair failed.");

                KeyPair {
                    public_key: CURVE_SM2P256V1.point_to_bytes(&public_key, false).expect("convert point to bytes failed."),
                    secret_key,
                    cryptography,
                }
            }
        }
    }

    pub fn from_secret_key(bytes: &[u8], cryptography: Cryptography) -> KeyPair {
        match cryptography {
            Cryptography::Secp256k1 => {
                let secret_key = SecretKey::from_slice(bytes).unwrap();
                let public_key = PublicKey::from_secret_key(&CONTEXT_SECP256K1, &secret_key);

                KeyPair {
                    public_key: public_key.serialize_uncompressed().to_vec(),
                    secret_key: BigUint::from_bytes_be(&secret_key.secret_bytes()),
                    cryptography,
                }
            }
            Cryptography::Sm2p256v1 => {
                let secret_key = BigUint::from_bytes_be(bytes);
                let public_key = CONTEXT_SM2P256V1.pk_from_sk(&secret_key).unwrap();

                KeyPair {
                    public_key: CURVE_SM2P256V1.point_to_bytes(&public_key, false).unwrap(),
                    secret_key,
                    cryptography,
                }
            }
        }
    }
}

/// 哈希
pub fn hash_message(message: &[u8], cryptography: Cryptography) -> String {
    match cryptography {
        Cryptography::Secp256k1 => {
            sha256::digest(message)
        }
        Cryptography::Sm2p256v1 => {
            let mut hash = Sm3Hash::new(message);
            let digest = hash.get_hash().to_vec();
            hex::encode(digest)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keypair() {
        let keypair_sm2p256v1 = KeyPair::new_keypair(Cryptography::Sm2p256v1);
        let keypair_secp256k1 = KeyPair::new_keypair(Cryptography::Secp256k1);

        assert_eq!(keypair_sm2p256v1.public_key.len(), 65);
        assert_eq!(keypair_sm2p256v1.secret_key.to_str_radix(16).len(), 64);
        assert_eq!(keypair_secp256k1.public_key.len(), 65);
        assert_eq!(keypair_secp256k1.secret_key.to_bytes_be().len(), 32)
    }

    #[test]
    fn hash_message_sm2p256v1() {
        let expected = "becbbfaae6548b8bf0cfcad5a27183cd1be6093b1cceccc303d9c61d0a645268";
        let actual = hash_message(b"hello", Cryptography::Sm2p256v1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn hash_message_secp256k1() {
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let actual = hash_message(b"hello", Cryptography::Secp256k1);
        assert_eq!(actual, expected);
    }
}
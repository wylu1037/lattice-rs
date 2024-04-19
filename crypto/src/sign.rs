use libsm::sm2::ecc::EccCtx;
use libsm::sm2::signature::SigCtx;
use libsm::sm3::hash::Sm3Hash;
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use secp256k1::{All, Message, PublicKey, rand::rngs::OsRng, Secp256k1, SecretKey};

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

    /// 从私钥恢复密钥对
    /// bytes 私钥
    /// cryptography Secp256k1 or Sm2p256v1
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

    pub fn sign(&self, message: &[u8]) -> String {
        match self.cryptography {
            Cryptography::Secp256k1 => {
                let sk = SecretKey::from_slice(&self.secret_key.to_bytes_be()).unwrap();
                let msg = Message::from_digest_slice(&message).unwrap();
                let (recovery_id, sig) = CONTEXT_SECP256K1
                    .sign_ecdsa_recoverable(&msg, &sk).serialize_compact();
                let r: &[u8] = &sig[..32];
                let s: &[u8] = &sig[32..];
                format!(
                    "0x{}{}0{}",
                    hex::encode(r),
                    hex::encode(s),
                    BigUint::from(recovery_id.to_i32() as u32).to_str_radix(16),
                )
            }
            Cryptography::Sm2p256v1 => {
                let pk = CURVE_SM2P256V1.bytes_to_point(&self.public_key).unwrap();
                // Get the value "e", which is the hash of message and ID, EC parameters and public key
                let digest = CONTEXT_SM2P256V1.hash("1234567812345678", &pk, message).unwrap();
                let sig = CONTEXT_SM2P256V1.sign_raw(&digest[..], &self.secret_key).unwrap();
                format!(
                    "0x{}{}01{}",
                    sig.get_r().to_str_radix(16),
                    sig.get_s().to_str_radix(16),
                    hex::encode(digest),
                )
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

    #[test]
    fn sign_secp256k1() {
        let sk =
            hex::decode("c842e1ef9ece7e992a4021423a58d6e89c751881e43fd7dbebe70f932ad493e2").unwrap();

        let data =
            hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();

        let key_pair = KeyPair::from_secret_key(&sk, Cryptography::Secp256k1);

        let sig = KeyPair::sign(&key_pair, &data);
        assert_eq!(sig, "0xc8eced818b011433b5d486f9f0c97c8d0180a0df042bcaf1e75a7cd20d66920a5bbc4901bd90353fc62828ed2a821a801440f294779fc402033bf92c7657c30600");
    }

    #[test]
    fn sign_sm2p256v1() {
        let sk =
            hex::decode("29d63245990076b0bbb33f7482beef21855a8d2197c8d076c2356c49e2a06322").unwrap();

        let data =
            hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();

        let key_pair = KeyPair::from_secret_key(&sk, Cryptography::Sm2p256v1);

        let sig = KeyPair::sign(&key_pair, &data);
        assert_eq!(sig, "7ca02541f80886a0612ca05bf555a1092a888abe3a6e0674381fc6fbf5253cea24e4ea409d7833868176d8eb01c852f841024a728f35a697ec9d691d6b5e24dd015931ab5708c28403560b471e30e7f5c404bdeabea2e8e2d5d6cc4f1ca96ba4aa");
    }
}
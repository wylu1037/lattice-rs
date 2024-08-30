use libsm::sm2::ecc::EccCtx;
use libsm::sm2::signature::{SigCtx, Signature};
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use secp256k1::{All, Message, PublicKey, rand::rngs::OsRng, Secp256k1, SecretKey};
use secp256k1::ecdsa::Signature as SigNist;

use model::enums::Curve;

use crate::public_key_to_address;

#[derive(Debug)]
pub struct KeyPair {
    /// 公钥，非压缩公钥，由1字节的前缀(标识y坐标的奇偶，0x02/0x03)+32字节的x坐标+32字节的y坐标
    pub public_key: Vec<u8>,
    /// 私钥，32字节
    pub secret_key: BigUint,
    /// 椭圆曲线，Secp256k1 or Sm2p256v1
    pub curve: Curve,
}

pub static CONTEXT_SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(Secp256k1::new);
pub static CONTEXT_SM2P256V1: Lazy<SigCtx> = Lazy::new(SigCtx::new);
pub static CURVE_SM2P256V1: Lazy<EccCtx> = Lazy::new(EccCtx::new);

impl KeyPair {
    pub fn new_keypair(curve: Curve) -> KeyPair {
        match curve {
            Curve::Secp256k1 => {
                let mut rng = OsRng::default();
                let (secret_key, public_key) = CONTEXT_SECP256K1.generate_keypair(&mut rng);

                KeyPair {
                    public_key: public_key.serialize_uncompressed().to_vec(),
                    secret_key: BigUint::from_bytes_be(&secret_key.secret_bytes()),
                    curve,
                }
            }
            Curve::Sm2p256v1 => {
                let (public_key, secret_key) = CONTEXT_SM2P256V1.new_keypair().expect("new keypair failed.");

                KeyPair {
                    public_key: CURVE_SM2P256V1.point_to_bytes(&public_key, false).expect("convert point to bytes failed."),
                    secret_key,
                    curve,
                }
            }
        }
    }

    /// 从私钥恢复密钥对
    /// bytes 私钥
    /// curve Secp256k1 or Sm2p256v1
    pub fn from_secret_key(bytes: &[u8], curve: Curve) -> KeyPair {
        match curve {
            Curve::Secp256k1 => {
                let secret_key = SecretKey::from_slice(bytes).unwrap();
                let public_key = PublicKey::from_secret_key(&CONTEXT_SECP256K1, &secret_key);

                KeyPair {
                    public_key: public_key.serialize_uncompressed().to_vec(),
                    secret_key: BigUint::from_bytes_be(&secret_key.secret_bytes()),
                    curve,
                }
            }
            Curve::Sm2p256v1 => {
                let secret_key = BigUint::from_bytes_be(bytes);
                let public_key = CONTEXT_SM2P256V1.pk_from_sk(&secret_key).unwrap();

                KeyPair {
                    public_key: CURVE_SM2P256V1.point_to_bytes(&public_key, false).unwrap(),
                    secret_key,
                    curve,
                }
            }
        }
    }

    /// # 签名
    /// ## 入参
    /// + `message: &[u8]`: 待签名的消息
    ///
    /// ## 出参
    /// + `String`: signature 签名结果
    pub fn sign(&self, message: &[u8]) -> String {
        match self.curve {
            Curve::Secp256k1 => {
                let sk = SecretKey::from_slice(&self.secret_key.to_bytes_be()).unwrap();
                let msg = Message::from_digest_slice(&message).unwrap();
                let (recovery_id, sig) = CONTEXT_SECP256K1
                    .sign_ecdsa_recoverable(&msg, &sk).serialize_compact();
                let r: &[u8] = &sig[..32];
                let s: &[u8] = &sig[32..];
                let recovery_id = recovery_id.to_i32() as u32 + 27;
                format!(
                    "0x{}{}{}",
                    hex::encode(r),
                    hex::encode(s),
                    BigUint::from(recovery_id).to_str_radix(16),
                )
            }
            Curve::Sm2p256v1 => {
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

    /// # 验签
    pub fn verify(&self, message: &[u8], signature: &str) -> bool {
        match self.curve {
            Curve::Secp256k1 => {
                let msg = Message::from_digest_slice(&message).unwrap();
                let sk = SecretKey::from_slice(self.secret_key.to_bytes_be().as_slice()).unwrap();
                let mut pk = PublicKey::from_secret_key(&CONTEXT_SECP256K1, &sk).serialize_uncompressed();
                pk[0] = 4;
                let public_key = PublicKey::from_slice(&pk).unwrap();
                let signature = KeyPair::get_clean_signature_hex(&signature);
                let signature = hex::decode(signature).unwrap();
                let signature = SigNist::from_compact(signature.as_slice()).unwrap();
                CONTEXT_SECP256K1.verify_ecdsa(&msg, &signature, &public_key).is_ok()
            }
            Curve::Sm2p256v1 => {
                let sk = BigUint::from_bytes_be(self.secret_key.to_bytes_be().as_slice());
                let pk = CONTEXT_SM2P256V1.pk_from_sk(&sk).unwrap();
                let signature = KeyPair::get_clean_signature_hex(signature);
                let r = hex::decode(&(signature[0..64])).unwrap();
                let s = hex::decode(&(signature[64..])).unwrap();
                let signature = Signature::new(r.as_slice(), s.as_slice());
                CONTEXT_SM2P256V1.verify(&message, &pk, &signature).is_ok()
            }
        }
    }

    /// # 只获取签名中的r、s
    fn get_clean_signature_hex(signature: &str) -> &str {
        let hex_str = if signature.starts_with("0x") {
            &signature[2..]
        } else {
            signature
        };
        &hex_str[..hex_str.len().min(128)]
    }

    /// # 获取地址
    pub fn address(&self) -> String {
        let key_encode = &hex::encode(&self.public_key)[2..];
        let key_decode = hex::decode(key_encode).unwrap();
        public_key_to_address(&key_decode, self.curve)
    }
}

#[cfg(test)]
mod tests {
    use model::enums::Curve;
    use model::HexString;

    use crate::hash::hash_message;

    use super::*;

    #[test]
    fn new_keypair() {
        let keypair_sm2p256v1 = KeyPair::new_keypair(Curve::Sm2p256v1);
        let keypair_secp256k1 = KeyPair::new_keypair(Curve::Secp256k1);

        assert_eq!(keypair_sm2p256v1.public_key.len(), 65);
        assert_eq!(keypair_sm2p256v1.secret_key.to_str_radix(16).len(), 64);
        assert_eq!(keypair_secp256k1.public_key.len(), 65);
        assert_eq!(keypair_secp256k1.secret_key.to_bytes_be().len(), 32)
    }

    #[test]
    fn hash_message_sm2p256v1() {
        let expected = "becbbfaae6548b8bf0cfcad5a27183cd1be6093b1cceccc303d9c61d0a645268";
        let actual = hash_message(b"hello", Curve::Sm2p256v1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn hash_message_secp256k1() {
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let actual = hash_message(b"hello", Curve::Secp256k1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn sign_and_verify_secp256k1() {
        let sk = HexString::new("0xc842e1ef9ece7e992a4021423a58d6e89c751881e43fd7dbebe70f932ad493e2").decode();
        let message =
            hex::decode("0102030405060708010203040506070801020304050607080102030405060708").unwrap();
        let key_pair = KeyPair::from_secret_key(&sk, Curve::Secp256k1);
        let signature = key_pair.sign(&message);
        println!("{}", signature);
        let pass = key_pair.verify(&message, &signature);
        assert_eq!(pass, true)
    }

    #[test]
    fn verify_secp256k1() {
        let sk = hex::decode("c842e1ef9ece7e992a4021423a58d6e89c751881e43fd7dbebe70f932ad493e2").unwrap();
        let message = hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();
        let signature = String::from("0xc8eced818b011433b5d486f9f0c97c8d0180a0df042bcaf1e75a7cd20d66920a5bbc4901bd90353fc62828ed2a821a801440f294779fc402033bf92c7657c3061b");

        let keypair = KeyPair::from_secret_key(&sk, Curve::Secp256k1);
        let b = KeyPair::verify(&keypair, &message, &signature);
        assert_eq!(b, true);
    }

    #[test]
    fn verify_sm2p256v1() {
        let sk = hex::decode("ae96ce342785f0a2663098336a42598eae814a5020433f193aca6c08af71a6a6").unwrap();
        let message = hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();
        let signature = String::from("0xa7fd7d7675f3db3917dbf667ff6b981fc79fef75b51a2de6bd032fac4e06159e8cbf1fa9e84c8dc4fe6a5b9c01e45246b1bfb6a066c19f9e25d1185cba313374011bab3d01ceb5c070d2291bd15fa2087205cbce2cc68df51561d915956ed83ed5");

        let keypair = KeyPair::from_secret_key(&sk, Curve::Sm2p256v1);
        let b = keypair.verify(&message, &signature);
        assert_eq!(b, true);
    }

    #[test]
    fn verify_sm2p256v1_error() {
        let sk = hex::decode("ae96ce342785f0a2663098336a42598eae814a5020433f193aca6c08af71a6a6").unwrap();
        let message = hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();
        let signature = String::from("0xcbe07a7e27bf85586b152df99cf191163e666545720758b8f55e88b4478b00fa5756d1dd47ba0b7600e7f5b22c4495ae59e9e444d24152335b460c938f23741201d640e7d7f013c3559a14a0c7ec010bd2b25a177faffb6a9821659af43684233a");

        let keypair = KeyPair::from_secret_key(&sk, Curve::Sm2p256v1);
        let b = keypair.verify(&message, &signature);
        assert_eq!(b, true);
    }

    #[test]
    fn sign_sm2p256v1() {
        let sk =
            hex::decode("29d63245990076b0bbb33f7482beef21855a8d2197c8d076c2356c49e2a06322").unwrap();

        let data =
            hex::decode("790dcb1e43ac151998f8c2e59e0959072f9d476d19fb6f98d7a4e59ea5f8e59e").unwrap();

        let key_pair = KeyPair::from_secret_key(&sk, Curve::Sm2p256v1);

        let sig = KeyPair::sign(&key_pair, &data);
        assert_eq!(sig, "7ca02541f80886a0612ca05bf555a1092a888abe3a6e0674381fc6fbf5253cea24e4ea409d7833868176d8eb01c852f841024a728f35a697ec9d691d6b5e24dd015931ab5708c28403560b471e30e7f5c404bdeabea2e8e2d5d6cc4f1ca96ba4aa");
    }

    #[test]
    fn sign_and_verify_sm2p256v1() {
        let sk = HexString::new("0x29d63245990076b0bbb33f7482beef21855a8d2197c8d076c2356c49e2a06322").decode();
        let message = HexString::new("0x0102030405060708010203040506070801020304050607080102030405060708").decode();

        let key_pair = KeyPair::from_secret_key(&sk, Curve::Sm2p256v1);
        let signature = key_pair.sign(&message);
        println!("{}", signature);
        let pass = key_pair.verify(&message, &signature);
        assert_eq!(pass, true)
    }

    #[test]
    fn recovery_keypair() {
        let sk = HexString::new("0x72ffdd7245e0ad7cffd533ad99f54048bf3fa6358e071fba8c2d7783d992d997").decode();
        let keypair = KeyPair::from_secret_key(&sk, Curve::Sm2p256v1);
        println!("{}", hex::encode(keypair.public_key.clone()));
        let address = public_key_to_address(keypair.public_key.as_slice(), Curve::Sm2p256v1);
        print!("{:?}", address);
    }
}
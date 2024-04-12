use libsm::sm2::ecc::EccCtx;
use libsm::sm2::signature::SigCtx;
use num_bigint::BigUint;
use once_cell::sync::Lazy;
use secp256k1::{All, rand::rngs::OsRng, Secp256k1};

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
    pub fn new_keypair(crypto: Cryptography) -> KeyPair {
        match crypto {
            Cryptography::Secp256k1 => {
                let mut rng = OsRng::default();
                let (secret_key, public_key) = CONTEXT_SECP256K1.generate_keypair(&mut rng);

                KeyPair {
                    public_key: public_key.serialize_uncompressed().to_vec(),
                    secret_key: BigUint::from_bytes_be(&secret_key.secret_bytes()),
                    cryptography: crypto,
                }
            }
            Cryptography::Sm2p256v1 => {
                let (public_key, secret_key) = CONTEXT_SM2P256V1.new_keypair().expect("new keypair failed.");

                KeyPair {
                    public_key: CURVE_SM2P256V1.point_to_bytes(&public_key, false).expect("convert point to bytes failed."),
                    secret_key,
                    cryptography: crypto,
                }
            }
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
}
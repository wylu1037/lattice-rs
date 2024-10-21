use std::fmt;
use std::ops::Deref;

use hmac::{Hmac, Mac};
use memzero::Memzero;
use num_bigint::BigUint;
use secp256k1::{PublicKey, Scalar, SecretKey};
use sha2::Sha512;

use crypto::sign::{CONTEXT_SECP256K1, CONTEXT_SM2P256V1, CURVE_SM2P256V1};
use model::Curve;

use crate::bip44::{ChildNumber, IntoDerivationPath};
use crate::error::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct Protected(Memzero<[u8; 32]>);

impl<Data: AsRef<[u8]>> From<Data> for Protected {
    fn from(data: Data) -> Protected {
        let mut buf = [0u8; 32];

        buf.copy_from_slice(data.as_ref());

        Protected(Memzero::from(buf))
    }
}

impl Deref for Protected {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for Protected {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Protected")
    }
}

/// # 扩展私钥，包括私钥[0..32]、链码[32..64]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExtendedPrivateKey {
    /// 私钥，32 byte
    secret_key: BigUint,
    /// 链码，32 byte
    chain_code: Protected,
}

// Create alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

impl ExtendedPrivateKey {
    /// Attempts to derive an extended private key from a path.
    pub fn derive<Path>(seed: &[u8], path: Path, curve: Curve) -> Result<ExtendedPrivateKey, Error>
    where
        Path: IntoDerivationPath,
    {
        let mut hmac =
            HmacSha512::new_from_slice(b"Bitcoin seed").expect("seed is always correct; qed");
        hmac.update(seed);

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.as_slice().split_at(32);

        let mut sk = ExtendedPrivateKey {
            secret_key: BigUint::from_bytes_be(secret_key),
            chain_code: Protected::from(chain_code),
        };

        for child in path.into()?.as_ref() {
            sk = sk.child(*child, curve)?;
        }

        Ok(sk)
    }

    /// # padding zero in the top of sk when sk len less than 32
    /// ## 入参
    ///
    /// ## 出参
    /// + `[u8; 32]`: secret key byte array
    pub fn secret(&self) -> [u8; 32] {
        let bytes = self.secret_key.to_bytes_be();

        let mut secret = [0u8; 32];

        let start = 32 - bytes.len();
        secret[start..].copy_from_slice(&bytes);

        secret
    }

    pub fn child(&self, child: ChildNumber, curve: Curve) -> Result<ExtendedPrivateKey, Error> {
        let mut hmac =
            HmacSha512::new_from_slice(&self.chain_code).map_err(|_| Error::InvalidChildNumber)?;

        if child.is_normal() {
            match curve {
                Curve::Secp256k1 => {
                    let sk = SecretKey::from_slice(&self.secret_key.to_bytes_be()).unwrap();
                    hmac.update(
                        &PublicKey::from_secret_key(&CONTEXT_SECP256K1, &sk).serialize()[..],
                    );
                }
                Curve::Sm2p256v1 => {
                    let pk = CONTEXT_SM2P256V1.pk_from_sk(&self.secret_key).unwrap();
                    let pk_bytes = CURVE_SM2P256V1.point_to_bytes(&pk, true).unwrap();
                    hmac.update(&pk_bytes);
                }
            }
        } else {
            hmac.update(&[0]);
            let sk_bytes = &self.secret();
            hmac.update(&sk_bytes[..32]);
        }

        hmac.update(&child.to_bytes());

        let result = hmac.finalize().into_bytes();
        let (secret_key, chain_code) = result.split_at(32);

        let sk: BigUint;
        match curve {
            Curve::Secp256k1 => {
                let mut secret_key =
                    SecretKey::from_slice(&secret_key).map_err(Error::Secp256k1)?;
                // 对私钥进行加法微调
                let scalar = Scalar::from_be_bytes(self.secret()).unwrap();
                secret_key = secret_key.add_tweak(&scalar).map_err(Error::Secp256k1)?;
                sk = BigUint::from_bytes_be(secret_key.secret_bytes().as_slice());
            }
            Curve::Sm2p256v1 => {
                let secret_key = BigUint::from_bytes_be(secret_key);
                // 对私钥进行加法微调
                sk = (secret_key + &self.secret_key) % CURVE_SM2P256V1.get_n();
            }
        }

        Ok(ExtendedPrivateKey {
            secret_key: sk,
            chain_code: Protected::from(&chain_code),
        })
    }
}

/*impl FromStr for ExtendedPrivateKey {
    type Err = Error;

    fn from_str(xprv: &str) -> Result<ExtendedPrivateKey, Error> {
        let data = xprv.from_base58().map_err(|_| Error::InvalidExtendedPrivKey)?;

        if data.len() != 82 {
            return Err(Error::InvalidExtendedPrivKey);
        }

        Ok(ExtendedPrivateKey {
            chain_code: Protected::from(&data[13..45]),
            secret_key: BigUint::from_bytes_be(&data[56..78]),
        })
    }
}*/

#[cfg(test)]
mod tests {
    use crypto::sign::KeyPair;
    use model::common::Address;
    use model::Curve;

    use crate::bip32::ExtendedPrivateKey;
    use crate::bip39::Mnemonic;

    const WORDS: &str = "potato front rug inquiry old author dose little still apart below develop";

    #[test]
    fn test_secp256k1_derive() {
        let seed = Mnemonic::from(WORDS).to_seed("Root1234");
        let ext = ExtendedPrivateKey::derive(seed.as_slice(), "m/44'/60'/0'/0/0", Curve::Secp256k1)
            .unwrap();
        let excepted_sk = "dbd91293f324e5e49f040188720c6c9ae7e6cc2b4c5274120ee25808e8f4b6a7";
        assert_eq!(hex::encode(ext.secret_key.to_bytes_be()), excepted_sk)
    }

    #[test]
    fn test_sm2p256v1_derive() {
        let seed = Mnemonic::from(WORDS).to_seed("Root1234");
        let ext = ExtendedPrivateKey::derive(seed.as_slice(), "m/44'/60'/0'/0/0", Curve::Sm2p256v1)
            .unwrap();
        let excepted_sk = "24f5d48f3804af48d7d0f3f02b25bdf7b3f936d8c2c7b04eca415fa83cc02758";
        assert_eq!(hex::encode(ext.secret_key.to_bytes_be()), excepted_sk)
    }

    #[test]
    fn test_sm2p256v1_derive2() {
        let seed = Mnemonic::from(
            "medal shed task apart range accident ride matrix fire citizen motion ridge",
        )
        .to_seed("123");
        let ext = ExtendedPrivateKey::derive(seed.as_slice(), "m/44'/2'/3'/4/5", Curve::Sm2p256v1)
            .unwrap();
        let excepted_sk = "cd2e0330c22f7d8d38e22ad8df4d15824a7ba0ef7150f4dd777bf036fde64eed";
        let expected_address = "0x76bc156f9188b09d549117af9391ce9947d4f45b";
        assert_eq!(hex::encode(ext.secret_key.to_bytes_be()), excepted_sk);
        let key_pair =
            KeyPair::from_secret_key(ext.secret_key.to_bytes_be().as_slice(), Curve::Sm2p256v1);
        assert_eq!(
            Address::new(key_pair.address().as_str()).to_ethereum_address(),
            expected_address
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    Secp256k1(secp256k1::Error),
    Sm2p256v1,
    InvalidChildNumber,
    InvalidDerivationPath,
    InvalidExtendedPrivateKey,
}
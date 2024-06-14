#[derive(Debug, Clone, Copy)]
pub enum Cryptography {
    /// 国际算法
    Secp256k1,
    /// 国密算法
    Sm2p256v1,
}
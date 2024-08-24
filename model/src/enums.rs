/// 椭圆曲线
#[derive(Debug, Clone, Copy)]
pub enum Curve {
    /// 国际算法，NIST
    Secp256k1,
    /// 国密算法，SMC
    Sm2p256v1,
}
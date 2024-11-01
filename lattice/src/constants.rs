/// json-rpc version number, fixed to 2.0
pub const JSON_RPC_VERSION: &str = "2.0";
/// 私钥的正则表达式校验
pub(crate) const REGEX_PRIVATE_KEY: &str = r"^(0x)?[a-zA-Z0-9]{64}$";
/// ZLTC地址的正则表达式校验
#[allow(dead_code)]
pub(crate) const REGEX_ZLTC_ADDRESS: &str = r#"^zltc_[a-zA-Z0-9]{33}$"#;

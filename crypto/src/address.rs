use sha256::digest;

use crate::sign::{Cryptography, hash_message};

/// # 公钥转地址
/// ## 入参
/// + `public_key: &[u8]`: 公钥，`0xaaa53093e7fc18c3335876afc3aa604cf624cf7091685f42e09ee69cab3a6bcee8e0297eda17b6d8d3bfda8cc44945304ffb8bc40b5b7ff47e132c0c3fa0bd7f`
/// + `cryptography: Cryptography`:
///
/// ## 出参
/// + `String`: Lattice地址，示例：zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi
pub fn public_key_to_address(public_key: &[u8], cryptography: Cryptography) -> String {
    let key_hash = hash_message(public_key, cryptography);
    let eth = &hex::decode(key_hash.as_bytes()).unwrap()[12..];
    eth_to_lattice(eth)
}

/// # 以太坊地址转为Lattice地址
/// ## 入参
/// + `addr: &[u8]`: 以太坊地址，示例：`[95, 43, 233, 160, 43, 67, 247, 72, 238, 70, 11, 243, 110, 237, 36, 250, 250, 16, 153, 32]`
///
/// ## 出参
/// + `String`: Lattice地址，示例：`zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
pub fn eth_to_lattice(addr: &[u8]) -> String {
    let prefix = hex::decode("01").unwrap();
    let hash = [&prefix, addr].concat();
    let d1 = hex::decode(digest(&hash)).unwrap();
    let d2 = hex::decode(digest(&d1)).unwrap();
    let d3 = [&prefix, addr, &d2[0..4]].concat();
    let encoded = bs58::encode(d3).into_string();
    format!("zltc_{}", encoded)
}

/// # Lattice地址转为以太坊地址
/// ## 入参
/// + `addr: &str`: Lattice地址，示例：`zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
///
/// ## 出参
/// + `String`: 示例：`0x5f2be9a02b43f748ee460bf36eed24fafa109920`
pub fn lattice_to_eth(addr: &str) -> String {
    let addr = &addr[5..]; // remove prefix
    let decoded = bs58::decode(addr).into_vec().unwrap();
    let len = decoded.len() - 4;

    let data = &decoded[1..len];
    hex::encode(data)
}

#[cfg(test)]
mod tests {
    use crate::model::HexString;

    use super::*;

    #[test]
    fn test_public_key_to_address() {
        let pk = HexString { hex: String::from("0xaaa53093e7fc18c3335876afc3aa604cf624cf7091685f42e09ee69cab3a6bcee8e0297eda17b6d8d3bfda8cc44945304ffb8bc40b5b7ff47e132c0c3fa0bd7f") };
        let addr = public_key_to_address(pk.decode().as_slice(), Cryptography::Sm2p256v1);
        assert_eq!(String::from("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi"), addr)
    }

    #[test]
    fn test_lattice_to_eth() {
        let addr = String::from("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi");
        let eth_addr = lattice_to_eth(addr.as_str());
        assert_eq!(String::from("0x5f2be9a02b43f748ee460bf36eed24fafa109920"), format!("0x{}", eth_addr))
    }

    #[test]
    fn test_eth_to_lattice() {
        let addr = hex::decode(String::from("5f2be9a02b43f748ee460bf36eed24fafa109920")).unwrap();
        let lattice_addr = eth_to_lattice(&addr);
        assert_eq!(String::from("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi"), lattice_addr)
    }
}
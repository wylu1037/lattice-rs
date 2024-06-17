use sha256::digest;

use crate::constants::ADDRESS_TITLE;

/// hex字符串结构体
pub struct HexString {
    pub hex_string: String,
}

const HEX_PREFIX: &str = "0x";

impl HexString {
    /// # 接收一个hex字符串来初始化一个hex对象
    pub fn new(hex: &str) -> Self {
        HexString { hex_string: hex.to_string() }
    }

    /// # 接收一个`&[u8]`字符串来初始化一个hex对象
    pub fn from(bs: &[u8]) -> Self {
        let hex_string = hex::encode(bs);
        HexString { hex_string: format!("{}{}", HEX_PREFIX, hex_string) }
    }

    /// # 获取没有前缀0x的hex string
    pub fn clean_hex_string(&self) -> String {
        if let Some(res) = self.hex_string.strip_prefix(HEX_PREFIX) {
            res.to_string()
        } else {
            self.hex_string.clone()
        }
    }

    /// # decode hex string
    pub fn decode(&self) -> Vec<u8> {
        hex::decode(self.clean_hex_string()).unwrap()
    }
}

/// 地址结构体
pub struct Address {
    pub addr: String,
}

impl Address {
    /// # 初始化一个地址对象
    /// ## 入参
    /// + `addr: &str`: 可接收一个zltc地址或ethereum地址
    ///   + `zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
    ///   + `0x5f2be9a02b43f748ee460bf36eed24fafa109920`
    ///
    /// ## 出参
    /// + `Address`
    pub fn new(addr: &str) -> Self {
        Address { addr: addr.to_string() }
    }

    /// # Lattice地址转为以太坊地址
    /// ## 入参
    /// + `&self`: Lattice地址，示例：`zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
    ///
    /// ## 出参
    /// + `String`: 示例：`0x5f2be9a02b43f748ee460bf36eed24fafa109920`
    pub fn to_ethereum_address(&self) -> String {
        if self.addr.starts_with(ADDRESS_TITLE) {
            let addr = &self.addr[5..];
            let decoded = bs58::decode(addr).into_vec().unwrap();
            let len = decoded.len() - 4;

            let data = &decoded[1..len];

            format!("{}{}", HEX_PREFIX, hex::encode(data))
        } else {
            self.addr.clone()
        }
    }

    /// # 以太坊地址转为Lattice地址
    /// ## 入参
    /// + `&self`: 以太坊地址，示例：`0x5f2be9a02b43f748ee460bf36eed24fafa109920`
    ///
    /// ## 出参
    /// + `String`: Lattice地址，示例：`zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi`
    pub fn to_zltc_address(&self) -> String {
        if self.addr.starts_with(HEX_PREFIX) {
            let eth = HexString::new(&self.addr).decode();
            let prefix = hex::decode("01").unwrap();
            let hash = [&prefix, eth.as_slice()].concat();
            let d1 = hex::decode(digest(&hash)).unwrap();
            let d2 = hex::decode(digest(&d1)).unwrap();
            let d3 = [&prefix, eth.as_slice(), &d2[0..4]].concat();
            let encoded = bs58::encode(d3).into_string();
            format!("{}{}", ADDRESS_TITLE, encoded)
        } else {
            self.addr.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Address, HexString};

    #[test]
    fn test_new_hex_string() {
        let bs = HexString::new("0x5f2be9a02b43f748ee460bf36eed24fafa109920").decode();
        let excepted: [u8; 20] = [95, 43, 233, 160, 43, 67, 247, 72, 238, 70, 11, 243, 110, 237, 36, 250, 250, 16, 153, 32];
        assert_eq!(excepted, bs.as_slice())
    }

    #[test]
    fn test_new_text_string_from_bytes() {
        let bs: [u8; 20] = [95, 43, 233, 160, 43, 67, 247, 72, 238, 70, 11, 243, 110, 237, 36, 250, 250, 16, 153, 32];
        let hex_string = HexString::from(bs.as_slice()).hex_string;
        let expected = "0x5f2be9a02b43f748ee460bf36eed24fafa109920";
        assert_eq!(expected, hex_string)
    }

    #[test]
    fn zltc_address_to_ethereum_address() {
        let a = Address::new("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi");
        let ethereum_address = a.to_ethereum_address();
        assert_eq!("0x5f2be9a02b43f748ee460bf36eed24fafa109920", ethereum_address)
    }

    #[test]
    fn ethereum_address_to_zltc_address() {
        let a = Address::new("0x5f2be9a02b43f748ee460bf36eed24fafa109920");
        let zltc_address = a.to_zltc_address();
        assert_eq!("zltc_Z1pnS94bP4hQSYLs4aP4UwBP9pH8bEvhi", zltc_address)
    }
}
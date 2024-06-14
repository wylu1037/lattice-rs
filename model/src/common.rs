use sha256::digest;

use crate::constants::ADDRESS_TITLE;

pub struct HexString {
    pub hex_string: String,
}

const HEX_PREFIX: &str = "0x";

impl HexString {
    pub fn new(hex: &str) -> Self {
        HexString { hex_string: hex.to_string() }
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

pub struct Address {
    pub addr: String,
}

impl Address {
    pub fn new(addr: &str) -> Self {
        Address { addr: addr.to_string() }
    }


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
    fn test_hex_string() {
        let str = HexString::new("0x0102030405").clean_hex_string();
        println!("{}", str);
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
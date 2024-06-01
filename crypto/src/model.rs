pub struct HexString {
    pub hex: String,
}

const HEX_PREFIX: &str = "0x";

impl HexString {
    /// # 获取没有前缀0x的hex string
    pub fn clean_hex_string(&self) -> String {
        if let Some(res) = self.hex.strip_prefix(HEX_PREFIX) {
            res.to_string()
        } else {
            self.hex.clone()
        }
    }

    /// # decode hex string
    pub fn decode(&self) -> Vec<u8> {
        hex::decode(self.clean_hex_string()).unwrap()
    }
}
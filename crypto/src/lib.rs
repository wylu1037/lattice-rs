pub use address::public_key_to_address;
pub use model::HexString;

pub mod sign;
pub mod model;

pub mod address;

#[cfg(test)]
mod tests {
    #[test]
    fn base58_encode() {
        let input = "Hello";
        let out = bs58::encode(input.as_bytes()).into_string();
        assert_eq!("9Ajdvzr", out);
    }
}
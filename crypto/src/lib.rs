pub mod sign;
mod model;

#[cfg(test)]
mod tests {
    #[test]
    fn base58_encode() {
        let input = "Hello";
        let out = bs58::encode(input.as_bytes()).into_string();
        assert_eq!("9Ajdvzr", out);
    }
}
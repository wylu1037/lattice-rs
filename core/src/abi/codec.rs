use ethabi::Bytes;
use crate::abi::errors::AbiError;

/// Trait for ABI encoding
pub trait AbiEncode {
    /// ABI encode the type
    fn encode(self) -> Vec<u8>;

    /// Returns the encoded value as hex string, _with_ a `0x` prefix
    fn encode_hex(self) -> String
        where
            Self: Sized,
    {
        hex::encode_prefixed(self.encode())
    }
}

/// Trait for ABI Decoding
pub trait AbiDecode: Sized {
    /// Decodes the ABI encoded data
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError>;

    /// Decode hex encoded ABI encoded data
    ///
    /// Expects a hex encoded string, with optional `0x` prefix
    fn decode_hex(data: impl AsRef<str>) -> Result<Self, AbiError> {
        let bytes: Bytes = data.as_ref().parse()?;
        Self::decode(bytes)
    }
}
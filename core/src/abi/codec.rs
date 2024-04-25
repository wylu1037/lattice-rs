use crate::abi::Detokenize;
use crate::abi::error::AbiError;
use crate::types::{Bytes, H256, U128, U256};

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

macro_rules! impl_abi_codec {
    ($($name:ty),*) => {
        $(
            impl AbiEncode for $name {
                fn encode(self) -> Vec<u8> {
                    let token = self.into_token();
                    crate::abi::encode(&[token]).into()
                }
            }
            impl AbiDecode for $name {
                fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
                    let tokens = crate::abi::decode(
                        $[Self::param_type()], bytes.as_ref()
                    )?;
                    Ok(<Self as Detokenize>::from_tokens(tokens)?)
                }
            }
        )*
    };
}

impl_abi_codec!(
    Vec<u8>,
    Bytes,
    bytes::Bytes,
    // Address,
    bool,
    String,
    H256,
    U128,
    U256,
    // I256,
    // Uint8,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128
);
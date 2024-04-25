pub use error::{Error, Result};

pub mod rlp;
mod error;

mod header;
mod decode;

/// RLP prefix byte for 0-length string.
/// 0x90 = 128
pub const EMPTY_STRING_CODE: u8 = 0x80;

/// RLP prefix byte for a 0-length array.
/// 0xC0 = 192
pub const EMPTY_LIST_CODE: u8 = 0xC0;
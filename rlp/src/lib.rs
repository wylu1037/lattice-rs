pub use encode::{MaxEncodedLen, MaxEncodedLenAssoc};
pub use error::{Error, Result};
/// header
pub use header::Header;

pub mod rlp;
mod error;

mod header;

mod decode;
mod encode;

/// RLP prefix byte for 0-length string.
/// 0x80 = 128
pub const EMPTY_STRING_CODE: u8 = 0x80;

/// RLP prefix byte for a 0-length array.
/// 0xC0 = 192
pub const EMPTY_LIST_CODE: u8 = 0xC0;

#[cfg(test)]
mod tests {
    #[test]
    fn cal() {
        let a = 0x90;
        println!("{:?}", a)
    }
}
use std::hint::unreachable_unchecked;

use bytes::Buf;

use crate::{EMPTY_LIST_CODE, EMPTY_STRING_CODE, Error, Result};
use crate::decode::static_left_pad;

/// The header of an RLP item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    /// True if listed, false otherwise.
    pub list: bool,
    /// Length of the payload in bytes.
    pub payload_length: usize,
}

impl Header {
    /// Decodes an RLP header from the given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short or the header is invalid.
    pub fn decode(buf: &mut &[u8]) -> Result<Self> {
        let payload_length: usize;
        let mut list = false;
        match get_next_byte(buf)? {
            /// 0..=0x7F 表示区间，`[0,127]`
            0..=0x7F => payload_length = 1,

            /// b @ 用于绑定整个匹配表达式的结果到变量 b 上
            /// 区间为 `[128, 183]`
            b @ EMPTY_STRING_CODE..=0xB7 => {
                buf.advance(1);
                payload_length = (b - EMPTY_STRING_CODE) as usize;
                if payload_length == 1 && get_next_byte(buf)? < EMPTY_STRING_CODE {
                    return Err(Error::NonCanonicalSingleByte);
                }
            }

            /// `[184, 191]` | `[248, 255]`
            b @ (0xB8..=0xBF | 0xF8..=0xFF) => {
                buf.advance(1);

                list = b >= 0xF8; // second range, >= 248
                let code = if list { 0xF7 } else { 0xB7 };

                // SAFETY: `b - code` is always in the range `1..=8` in the current match arm.
                // The compiler/LLVM apparently cannot prove this because of the `|` pattern +
                // the above `if`, since it can do it in the other arms with only 1 range.
                let len_of_len = unsafe { b.checked_sub(code).unwrap_unchecked() } as usize;
                if len_of_len == 0 || len_of_len > 8 {
                    unsafe { unreachable_unchecked() }
                }

                if buf.len() < len_of_len {
                    return Err(Error::InputTooShort);
                }
                // SAFETY: length checked above
                let len = unsafe { buf.get_unchecked(..len_of_len) };
                buf.advance(len_of_len);

                let len = u64::from_be_bytes(static_left_pad(len)?);
                payload_length =
                    usize::try_from(len).map_err(|_| Error::Custom("Input too big"))?;
                if payload_length < 56 {
                    return Err(Error::NonCanonicalSize);
                }
            }

            /// `[192, 247]`
            b @ EMPTY_LIST_CODE..=0xF7 => {
                buf.advance(1);
                list = true;
                payload_length = (b - EMPTY_LIST_CODE) as usize
            }
        }

        if buf.remaining() < payload_length {
            return Err(Error::InputTooShort);
        }

        Ok(Self { list, payload_length })
    }

    /// Decodes the next payload from the given buffer, advancing it.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short or the header is invalid.
    #[inline]
    pub fn decode_bytes<'a>(buf: &mut &'a [u8], is_list: bool) -> Result<&'a [u8]> {
        let Self { list, payload_length } = Self::decode(buf)?;
        if list != is_list {
            return Err(if is_list { Error::UnexpectedString } else { Error::UnexpectedList });
        }

        // SAFETY: this is already checked in `decode`
        if buf.remaining() < payload_length {
            unsafe { unreachable_unchecked() }
        }
        let bytes = unsafe { buf.get_unchecked(..payload_length) };
        buf.advance(payload_length);
        Ok(bytes)
    }
}

/// Same as `buf.first().ok_or(Error::InputTooShort)`.
#[inline(always)]
fn get_next_byte(buf: &[u8]) -> Result<u8> {
    if buf.is_empty() {
        return Err(Error::InputTooShort);
    }
    // SAFETY: length checked above
    Ok(*unsafe { buf.get_unchecked(0) })
}

#[cfg(test)]
mod tests {
    use crate::header::get_next_byte;

    #[test]
    fn get_nex_byte() {
        let buf = [1u8, 2];
        let n = get_next_byte(&buf).unwrap();
        println!("{}", n)
    }
}
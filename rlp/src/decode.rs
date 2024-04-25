use crate::{Error, Result};

/// Left-pads a slice to a statically known size array.
/// 填充数组为指定的N长度，不足则在左侧补零
///
/// # Errors
///
/// Returns an error if the slice is too long or if the first byte is 0.
#[inline]
pub(crate) fn static_left_pad<const N: usize>(data: &[u8]) -> Result<[u8; N]> {
    if data.len() > N {
        return Err(Error::Overflow);
    }

    let mut v = [0; N];

    if data.is_empty() {
        return Ok(v);
    }

    if data[0] == 0 {
        return Err(Error::LeadingZero);
    }

    // SAFETY: length checked above
    unsafe { v.get_unchecked_mut(N - data.len()..) }.copy_from_slice(data);
    Ok(v)
}
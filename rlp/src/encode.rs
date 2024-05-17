use std::marker::{PhantomData, PhantomPinned};

use bytes::BufMut;
use open_fastrlp::{Header, length_of_length};

use crate::EMPTY_STRING_CODE;

/// A type that can be encoded via RLP.
pub trait Encodable {
    /// Encodes the type into the `out` buffer.
    fn encode(&self, out: &mut dyn BufMut);

    /// Returns the length of the encoding of this type in bytes.
    ///
    /// The default implementation computes this by encoding the type.
    /// When possible, we recommender implementers override this with a
    /// specialized implementation.
    #[inline]
    fn length(&self) -> usize {
        let mut out = Vec::new();
        self.encode(&mut out);
        out.len()
    }
}

/// The existence of this function makes the compiler catch if the Encodable
/// trait is "object-safe" or not.
fn _assert_trait_object(_b: &dyn Encodable) {}

/// Defines the max length of an [`Encodable`] type as a const generic.
///
/// # Safety
///
/// An invalid value can cause the encoder to panic.
pub unsafe trait MaxEncodedLen<const LEN: usize>: Encodable {}

/// Defines the max length of an [`Encodable`] type as an associated constant.
///
/// # Safety
///
/// An invalid value can cause the encoder to panic.
pub unsafe trait MaxEncodedLenAssoc: Encodable {
    /// The maximum length.
    const LEN: usize;
}

/// Implement [`MaxEncodedLen`] and [`MaxEncodedLenAssoc`] for a type.
///
/// # Safety
///
/// An invalid value can cause the encoder to panic.
#[macro_export]
macro_rules! impl_max_encoded_len {
    ($t:ty, $len:expr) => {
        unsafe impl $crate::MaxEncodedLen<{ $len }> for $t {}
        unsafe impl $crate::MaxEncodedLenAssoc for $t {
            const LEN: usize = $len;
        }
    };
}

impl_max_encoded_len!(bool, <u8 as MaxEncodedLenAssoc>::LEN);

macro_rules! to_be_bytes_trimmed {
    ($be:ident, $x:expr) => {{
        $be = $x.to_be_bytes();
        &$be[($x.leading_zeros() / 8) as usize..]
    }};
}
pub(crate) use to_be_bytes_trimmed;

impl Encodable for [u8] {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        if self.len() != 1 || self[0] > EMPTY_STRING_CODE {
            Header { list: false, payload_length: self.len() }.encode(out);
        }
        let a = out;
        out.put_slice(self)
    }

    #[inline]
    fn length(&self) -> usize {
        let mut len = self.len();
        if len != 1 || self[0] >= EMPTY_STRING_CODE {
            len += length_of_length(len);
        }
        len
    }
}

impl<T: ?Sized> Encodable for PhantomData<T> {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {}

    #[inline]
    fn length(&self) -> usize {
        0
    }
}

impl Encodable for PhantomPinned {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {}

    #[inline]
    fn length(&self) -> usize {
        0
    }
}

impl<const N: usize> Encodable for [u8; N] {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        self[..].encode(out)
    }

    #[inline]
    fn length(&self) -> usize {
        self[..].length()
    }
}

unsafe impl<const N: usize> MaxEncodedLenAssoc for [u8; N] {
    const LEN: usize = N + length_of_length(N);
}

impl Encodable for str {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        self.as_bytes().encode(out)
    }

    #[inline]
    fn length(&self) -> usize {
        self.as_bytes().length()
    }
}

impl Encodable for bool {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        // inlined `(*self as u8).encode(out)`
        out.put_u8(if *self { 1 } else { EMPTY_STRING_CODE })
    }

    #[inline]
    fn length(&self) -> usize {
        // a `bool` is always `< EMPTY_STRING_CODE`
        1
    }
}

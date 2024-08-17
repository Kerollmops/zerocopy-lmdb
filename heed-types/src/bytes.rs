use std::convert::Infallible;

use heed_traits::{BoxedError, BytesDecode, ToBytes};

/// Describes a byte slice `[u8]` that is totally borrowed and doesn't depend on
/// any [memory alignment].
///
/// [memory alignment]: std::mem::align_of()
pub enum Bytes {}

impl<'a> ToBytes<'a> for Bytes {
    type SelfType = [u8];

    type ReturnBytes = &'a [u8];

    type Error = Infallible;

    fn to_bytes(item: &'a Self::SelfType) -> Result<Self::ReturnBytes, Self::Error> {
        Ok(item)
    }
}

impl<'a> BytesDecode<'a> for Bytes {
    type DItem = &'a [u8];

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, BoxedError> {
        Ok(bytes)
    }
}

/// Like [`Bytes`], but always contains exactly `N` (the generic parameter) bytes.
pub enum FixedSizeBytes<const N: usize> {}

impl<'a, const N: usize> ToBytes<'a> for FixedSizeBytes<N> {
    type SelfType = [u8; N];

    type ReturnBytes = [u8; N]; // TODO &'a [u8; N] or [u8; N]

    type Error = Infallible;

    fn to_bytes(item: &'a Self::SelfType) -> Result<Self::ReturnBytes, Self::Error> {
        Ok(*item)
    }
}

impl<'a, const N: usize> BytesDecode<'a> for FixedSizeBytes<N> {
    type DItem = [u8; N]; // TODO &'a [u8; N] or [u8; N]

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, BoxedError> {
        bytes.try_into().map_err(Into::into)
    }
}

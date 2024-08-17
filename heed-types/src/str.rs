use std::convert::Infallible;

use heed_traits::{BoxedError, BytesDecode, ToBytes};

/// Describes a [`str`].
pub enum Str {}

impl<'a> ToBytes<'a> for Str {
    type SelfType = str;

    type ReturnBytes = &'a [u8];

    type Error = Infallible;

    fn to_bytes(item: &'a Self::SelfType) -> Result<Self::ReturnBytes, Self::Error> {
        Ok(item.as_bytes())
    }
}

impl<'a> BytesDecode<'a> for Str {
    type DItem = &'a str;

    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, BoxedError> {
        std::str::from_utf8(bytes).map_err(Into::into)
    }
}

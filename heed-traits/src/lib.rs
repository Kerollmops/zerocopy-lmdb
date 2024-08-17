#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/meilisearch/heed/main/assets//heed-pigeon.ico?raw=true"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/meilisearch/heed/main/assets/heed-pigeon-logo.png?raw=true"
)]

//! Contains the traits used to encode and decode database content.

#![warn(missing_docs)]

use std::borrow::Cow;
use std::cmp::{Ord, Ordering};
use std::error::Error as StdError;
use std::fmt;

/// A boxed `Send + Sync + 'static` error.
pub type BoxedError = Box<dyn StdError + Send + Sync + 'static>;

/// A trait that represents an encoding structure.
#[deprecated = "replaced by `ToBytes` to allow for more optimization"]
#[allow(deprecated)] // deprecated BoxedErrorWrapper is used in a bound
pub trait BytesEncode<'a>:
    // TODO are these bound needed?
    ToBytes<'a, SelfType = Self::EItem, ReturnBytes = Cow<'a, [u8]>, Error = BoxedErrorWrapper>
{
    /// The type to encode.
    type EItem: ?Sized + 'a;

    /// Encode the given item as bytes.
    fn bytes_encode(item: &'a Self::EItem) -> Result<Cow<'a, [u8]>, BoxedError>;
}

/// A trait that represents an encoding structure.
pub trait ToBytes<'a> {
    /// The type to encode to bytes.
    type SelfType: ?Sized + 'a;

    /// The type containing the encoded bytes.
    type ReturnBytes: Into<Vec<u8>> + AsRef<[u8]> + 'a;

    /// The error type to return when decoding goes wrong.
    type Error: StdError + Send + Sync + 'static;

    /// Encode the given item as bytes.
    fn to_bytes(item: &'a Self::SelfType) -> Result<Self::ReturnBytes, Self::Error>;
}

#[allow(deprecated)]
impl<'a, T: BytesEncode<'a>> ToBytes<'a> for T {
    type SelfType = <Self as BytesEncode<'a>>::EItem;

    type ReturnBytes = Cow<'a, [u8]>;

    type Error = BoxedErrorWrapper;

    fn to_bytes(item: &'a Self::SelfType) -> Result<Self::ReturnBytes, Self::Error> {
        Self::bytes_encode(item).map_err(BoxedErrorWrapper)
    }
}

/// Wraps the [`BoxedError`] type alias because for complicated reasons it does not implement
/// [`Error`][StdError]. This wrapper forwards [`Debug`][fmt::Debug], [`Display`][fmt::Display]
/// and [`Error`][StdError] through the wrapper and the [`Box`].
#[deprecated = "this wrapper was added for backwards compatibility of BytesEncode only"]
pub struct BoxedErrorWrapper(BoxedError);

#[allow(deprecated)]
impl fmt::Debug for BoxedErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <BoxedError as fmt::Debug>::fmt(&self.0, f)
    }
}

#[allow(deprecated)]
impl fmt::Display for BoxedErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <BoxedError as fmt::Display>::fmt(&self.0, f)
    }
}

#[allow(deprecated)]
impl StdError for BoxedErrorWrapper {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

/// A trait that represents a decoding structure.
pub trait BytesDecode<'a> {
    /// The type to decode.
    type DItem: 'a;

    /// Decode the given bytes as `DItem`.
    fn bytes_decode(bytes: &'a [u8]) -> Result<Self::DItem, BoxedError>;
}

/// Define a custom key comparison function for a database.
///
/// The comparison function is called whenever it is necessary to compare a key specified
/// by the application with a key currently stored in the database. If no comparison function
/// is specified, and no special key flags were specified, the keys are compared lexically,
/// with shorter keys collating before longer keys.
pub trait Comparator {
    /// Compares the raw bytes representation of two keys.
    fn compare(a: &[u8], b: &[u8]) -> Ordering;
}

/// Define a lexicographic comparator, which is a special case of [`Comparator`].
///
/// Types that implements [`LexicographicComparator`] will automatically have [`Comparator`]
/// implemented as well, where the [`Comparator::compare`] is implemented per the definition
/// of lexicographic ordering with [`LexicographicComparator::compare_elem`].
///
/// This trait is introduced to support prefix iterators, which implicit assumes that the
/// underlying key comparator is lexicographic.
pub trait LexicographicComparator: Comparator {
    /// Compare a single byte; this function is used to implement [`Comparator::compare`]
    /// by definition of lexicographic ordering.
    fn compare_elem(a: u8, b: u8) -> Ordering;

    /// Advances the given `elem` to its immediate lexicographic successor, if possible.
    /// Returns `None` if `elem` is already at its maximum value with respect to the
    /// lexicographic order defined by this comparator.
    fn successor(elem: u8) -> Option<u8>;

    /// Moves the given `elem` to its immediate lexicographic predecessor, if possible.
    /// Returns `None` if `elem` is already at its minimum value with respect to the
    /// lexicographic order defined by this comparator.
    fn predecessor(elem: u8) -> Option<u8>;

    /// Returns the maximum byte value per the comparator's lexicographic order.
    fn max_elem() -> u8;

    /// Returns the minimum byte value per the comparator's lexicographic order.
    fn min_elem() -> u8;
}

impl<C: LexicographicComparator> Comparator for C {
    fn compare(a: &[u8], b: &[u8]) -> Ordering {
        for idx in 0..std::cmp::min(a.len(), b.len()) {
            if a[idx] != b[idx] {
                return C::compare_elem(a[idx], b[idx]);
            }
        }
        Ord::cmp(&a.len(), &b.len())
    }
}

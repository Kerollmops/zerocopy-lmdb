#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/meilisearch/heed/main/assets//heed-pigeon.ico?raw=true"
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/meilisearch/heed/main/assets/heed-pigeon-logo.png?raw=true"
)]

//! Contains the traits used to encode and decode database content.

#![warn(missing_docs)]

use std::cmp::{Ord, Ordering};
use std::error::Error as StdError;
use std::io;

/// A boxed `Send + Sync + 'static` error.
pub type BoxedError = Box<dyn StdError + Send + Sync + 'static>;

/// A trait that represents an encoding structure.
pub trait BytesEncode<'a> {
    /// The type to encode.
    type EItem: ?Sized + 'a;

    /// The type containing the encoded bytes.
    type ReturnBytes: Into<Vec<u8>> + AsRef<[u8]> + 'a;

    /// The error type to return when decoding goes wrong.
    type Error: StdError + Send + Sync + 'static;

    /// Encode the given item as bytes.
    fn bytes_encode(item: &'a Self::EItem) -> Result<Self::ReturnBytes, Self::Error>;

    /// Encode the given item as bytes and write it into the writer. Returns the amount of bytes
    /// that were written. This function by default forwards to
    /// [`bytes_encode`][BytesEncode::bytes_encode].
    fn bytes_encode_into_writer<W: io::Write>(
        item: &'a Self::EItem,
        writer: &mut W,
    ) -> Result<usize, BoxedError> {
        let bytes = Self::bytes_encode(item)?;
        let bytes = bytes.as_ref();

        writer.write_all(bytes)?;

        Ok(bytes.len())
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

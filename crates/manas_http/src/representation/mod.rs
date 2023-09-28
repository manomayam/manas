//! I define types for representing and handling resource representations.
//!

use std::fmt::Debug;

use typed_record::TypedRecord;

use self::metadata::{KContentRange, RepresentationMetadata};

pub mod metadata;

#[cfg(feature = "impl-representation")]
pub mod impl_;

/// A trait for http resource representations.
///
/// > A "representation" is information that is intended to reflect
/// > a past, current, or desired state of a given resource,
/// > in a format that can be readily communicated via the protocol.
/// > A representation consists of a set of representation metadata and
/// > a potentially unbounded stream of representation data (Section 8).
///
pub trait Representation: Debug {
    /// Type of representation data.
    type Data;

    /// Get the representation data.
    fn data(&self) -> &Self::Data;

    /// Get the representation metadata.
    fn metadata(&self) -> &RepresentationMetadata;

    /// Convert into parts.
    fn into_parts(self) -> (Self::Data, RepresentationMetadata);
}

/// An extension trait for [`Representation`].
pub trait RepresentationExt: Representation {
    /// Get if representation is complete.
    #[inline]
    fn is_complete(&self) -> bool {
        self.metadata().get_rv::<KContentRange>().is_none()
    }
}

impl<R: Representation> RepresentationExt for R {}

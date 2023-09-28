//! I define type for representing valid uri segment, and few invariants of it.

pub mod invariant;
pub mod predicate;
pub mod safe_token;

use std::ops::Deref;

use ecow::EcoString;
use uriparse::{PathError, Segment};

/// A valid http uri segment str.
/// It is owned, and has O(1) semantics for clone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SegmentStr(EcoString);

impl<'segment> From<Segment<'segment>> for SegmentStr {
    #[inline]
    fn from(segment: Segment<'segment>) -> Self {
        Self(EcoString::from(segment.as_str()))
    }
}

impl<'segment> TryFrom<&'segment str> for SegmentStr {
    type Error = PathError;

    #[inline]
    fn try_from(value: &'segment str) -> Result<Self, Self::Error> {
        Ok(Segment::try_from(value)?.into())
    }
}

impl AsRef<str> for SegmentStr {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SegmentStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SegmentStr> for EcoString {
    #[inline]
    fn from(val: SegmentStr) -> Self {
        val.0
    }
}

impl SegmentStr {
    #[inline]
    /// Get parsed `Segment`.
    pub fn as_parsed(&self) -> Segment<'_> {
        Segment::try_from(self.0.as_str()).expect("Must be valid, as checked at instantiation.")
    }

    #[inline]
    /// Get as str.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

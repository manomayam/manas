//! I define traits and few implementations for
//! representation range negotiators.
//!

pub mod impl_;

use std::fmt::Debug;

use dyn_clone::DynClone;
use manas_http::{header::range::Range, representation::metadata::RepresentationMetadata};

/// A [`RangeNegotiator`] will be used by a caller to
/// negotiate it's preferred rep range, for any service that
/// derefers a representation.
pub trait RangeNegotiator: Debug + Send + DynClone + 'static {
    /// Resolve preferred range for selected representation
    /// with given metadata.
    fn resolve_pref_range(
        self: Box<Self>,
        selected_rep_metadata: &RepresentationMetadata,
    ) -> Option<Range>;
}

/// Type alias for type erased [`RangeNegotiator`].
pub type DynRangeNegotiator = dyn RangeNegotiator + Send + Sync + 'static;

dyn_clone::clone_trait_object!(RangeNegotiator);

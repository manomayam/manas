//! I define types for representation preferences.
//!

use typed_record::TypedRecordKey;

use self::range_negotiator::{impl_::CompleteRangeNegotiator, DynRangeNegotiator};

pub mod range_negotiator;

/// A struct representing representation preferences.
#[derive(Debug, Clone)]
pub struct RepresentationPreferences {
    /// Representation preference, if resource is a container.
    pub container_rep_preference: ContainerRepresentationPreference,

    /// Rep range negotiator, if resource is a non-container.
    pub non_container_rep_range_negotiator: Box<DynRangeNegotiator>,
}

impl RepresentationPreferences {
    /// Get configuration for lightweight rep op.
    // TODO should concretize guarantee.
    #[inline]
    pub fn new_light() -> Self {
        Self {
            container_rep_preference: ContainerRepresentationPreference::Minimal,
            non_container_rep_range_negotiator: Box::new(CompleteRangeNegotiator),
        }
    }
}

/// An enum corresponding to container representation preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerRepresentationPreference {
    /// Representation with all triples.
    All,

    /// Representation with only containment triples.and contained metadata triples.
    Containment,

    /// Representation with only container-minimal triples.
    Minimal,
    // /// User supplied triples
    // UserSupplied,
}

/// A [`TypedRecordKey`] for applied container rep preference.
#[derive(Clone)]
pub struct KAppliedContainerRepPref {}

impl TypedRecordKey for KAppliedContainerRepPref {
    type Value = ContainerRepresentationPreference;
}

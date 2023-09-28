//! I define types to represent slot relation types.
//!

use manas_http::header::link::RelationType;

use self::aux_rel_type::known::KnownAuxRelType;
use crate::SpcKnownAuxRelType;

pub mod aux_rel_type;

/// A slot relation type can be either `Contains`, or any known
/// auxiliary.
///
/// `Contains` relation type links a container resource to a
/// contained resource. It is equivalent to `ldp::contains` rdf
/// predicate.
///
/// An auxiliary relation type links a subject resource to an
/// auxiliary resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlotRelationType<KnAux> {
    /// Contains relation type
    Contains,

    /// Auxiliary relation type.
    Auxiliary(KnAux),
}

/// Type alias for [`SlotRelationType`] corresponding to a
/// `SolidStorageSpace`.
pub type SpcSlotRelType<Space> = SlotRelationType<SpcKnownAuxRelType<Space>>;

impl<KnAux> From<SlotRelationType<KnAux>> for RelationType
where
    KnAux: KnownAuxRelType,
{
    fn from(value: SlotRelationType<KnAux>) -> Self {
        match value {
            SlotRelationType::Contains => "http://www.w3.org/ns/ldp#contains"
                .parse()
                .expect("Must be valid"),
            SlotRelationType::Auxiliary(rel_type) => rel_type.into(),
        }
    }
}

impl<KnAux> SlotRelationType<KnAux>
where
    KnAux: KnownAuxRelType,
{
    /// Check if relation type is `Contains`.
    #[inline]
    pub fn is_contains(&self) -> bool {
        self == &Self::Contains
    }

    /// Check if relation type is an auxiliary.
    #[inline]
    pub fn is_auxiliary(&self) -> bool {
        !self.is_contains()
    }

    /// Check if relation type is auxiliary with given type.
    #[inline]
    pub fn is_auxiliary_of_type(&self, rel_type: &RelationType) -> bool {
        match self {
            SlotRelationType::Contains => false,
            SlotRelationType::Auxiliary(kn_rel_type) => kn_rel_type.deref() == rel_type,
        }
    }
}

#[cfg(feature = "test-utils")]
/// I define utils for easily mocking [`SlotRelationType`].
pub mod mock {
    use std::ops::Deref;

    use manas_http::header::link::RelationType;
    use once_cell::sync::Lazy;

    use super::{aux_rel_type::known::KnownAuxRelType, SlotRelationType};

    /// Hint for [`SlotRelationType`].
    #[derive(Debug, Clone)]
    pub enum SlotRelationTypeHint {
        /// Contains
        Contains,

        /// Auxiliary relation hint.
        Auxiliary(&'static Lazy<RelationType>),
    }

    impl SlotRelationTypeHint {
        /// Asserts given hint is valid.
        pub fn assert_valid_hint<KnAux: KnownAuxRelType>(
            &self,
            msg: &str,
        ) -> SlotRelationType<KnAux> {
            match self {
                SlotRelationTypeHint::Contains => SlotRelationType::Contains,
                SlotRelationTypeHint::Auxiliary(rel_type) => {
                    SlotRelationType::Auxiliary((*rel_type).deref().clone().try_into().expect(msg))
                }
            }
        }
    }
}

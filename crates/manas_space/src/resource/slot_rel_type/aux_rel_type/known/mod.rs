//! I define types for defining known aux relation types with their profiles.

pub mod impl_;

use std::{borrow::Borrow, fmt::Debug, hash::Hash, ops::Deref};

use manas_http::header::link::RelationType;

use super::ACL_REL_TYPE;
use crate::resource::kind::SolidResourceKind;

/// An enum representing role of an aux resource in it's access resolution process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuxAccessResolutionRole {
    /// Implies, it's access is determined by it's own acls.
    Independent,

    /// Implies, it's resolved access in any context
    ///  is same as that of subject resource.
    SubjectResource,

    /// Implies, it's resolved access is determined by control access to subject resource.
    SubjectResourceControl,
}

/// Trait for known aux rel type.
///
/// Each storage space has set of known valid aux rel types,
/// with few constraints.
pub trait KnownAuxRelType:
    Into<RelationType>
    + TryFrom<RelationType, Error = UnknownAuxRelTypeError>
    + Deref<Target = RelationType>
    + Borrow<RelationType>
    + Debug
    + Clone
    + PartialEq
    + Eq
    + Hash
    + Send
    + Sync
    + 'static
{
    /// Get target res kind, for given aux rel type.
    #[inline]
    fn target_res_kind(&self) -> SolidResourceKind {
        SolidResourceKind::NonContainer
    }

    /// Get if target must be rdf source.
    /// It will be ignored for container targets, as they are guaranteed to be rdf sources.
    #[inline]
    fn target_is_rdf_source(&self) -> bool {
        true
    }

    /// Get allowed subject res kinds for given aux rel type.
    #[inline]
    fn allowed_subject_res_kinds(&self) -> &'static [SolidResourceKind] {
        &[
            SolidResourceKind::Container,
            SolidResourceKind::NonContainer,
        ]
    }

    /// Get target's aux acl resolution role.
    #[inline]
    fn target_access_resolution_role(&self) -> AuxAccessResolutionRole {
        if self.borrow() == ACL_REL_TYPE.deref() {
            AuxAccessResolutionRole::SubjectResourceControl
        } else {
            AuxAccessResolutionRole::SubjectResource
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Given aux rel type is not a known rel type.")]
/// Error of an aux rel type being not a known one.
pub struct UnknownAuxRelTypeError(pub RelationType);

#[cfg(feature = "test-utils")]
/// I define utilities for easily mocking [`KnownAuxRelType`].
pub mod mock {
    use std::{borrow::Borrow, collections::HashSet, ops::Deref};

    use manas_http::header::link::RelationType;
    use once_cell::sync::Lazy;

    use super::*;
    use crate::resource::{
        kind::SolidResourceKind,
        slot_rel_type::aux_rel_type::{mock::*, *},
    };

    /// A mock implementation of [`KnownAuxRelType`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MockKnownAuxRelType(RelationType);

    static ALL_KNOWN_RAW_AUX_REL_TYPES: &[&Lazy<RelationType>] = &[
        &ACL_REL_TYPE,
        &DESCRIBED_BY_REL_TYPE,
        &CONTAINER_INDEX_REL_TYPE,
        &TA1_REL_TYPE,
        &TA2_REL_TYPE,
        &TC1_REL_TYPE,
        &TC2_REL_TYPE,
    ];

    /// Set of all known aux rel types for mocking.
    pub static ALL_KNOWN_AUX_REL_TYPES: Lazy<HashSet<MockKnownAuxRelType>> = Lazy::new(|| {
        ALL_KNOWN_RAW_AUX_REL_TYPES
            .iter()
            .map(|aux_rel_type| MockKnownAuxRelType(Lazy::force(*aux_rel_type).clone()))
            .collect()
    });

    impl Deref for MockKnownAuxRelType {
        type Target = RelationType;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Borrow<RelationType> for MockKnownAuxRelType {
        #[inline]
        fn borrow(&self) -> &RelationType {
            &self.0
        }
    }

    impl TryFrom<RelationType> for MockKnownAuxRelType {
        type Error = UnknownAuxRelTypeError;

        fn try_from(rel_type: RelationType) -> Result<Self, Self::Error> {
            if ALL_KNOWN_RAW_AUX_REL_TYPES
                .iter()
                .any(|known| (*known).deref() == &rel_type)
            {
                Ok(Self(rel_type))
            } else {
                Err(UnknownAuxRelTypeError(rel_type))
            }
        }
    }

    impl From<MockKnownAuxRelType> for RelationType {
        #[inline]
        fn from(val: MockKnownAuxRelType) -> Self {
            val.0
        }
    }

    impl KnownAuxRelType for MockKnownAuxRelType {
        fn target_res_kind(&self) -> SolidResourceKind {
            if [TC1_REL_TYPE.deref(), TC2_REL_TYPE.deref()].contains(&&self.0) {
                SolidResourceKind::Container
            } else {
                SolidResourceKind::NonContainer
            }
        }

        fn allowed_subject_res_kinds(&self) -> &'static [SolidResourceKind] {
            if &self.0 == CONTAINER_INDEX_REL_TYPE.deref() {
                &[SolidResourceKind::Container]
            } else {
                &[
                    SolidResourceKind::NonContainer,
                    SolidResourceKind::Container,
                ]
            }
        }
    }
}

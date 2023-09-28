//! I define a default implementations of [`KnownAuxRelType`].

use std::{borrow::Borrow, collections::HashSet, ops::Deref};

use manas_http::header::link::RelationType;
use once_cell::sync::Lazy;

use crate::resource::{
    kind::SolidResourceKind,
    slot_rel_type::aux_rel_type::{
        known::{KnownAuxRelType, UnknownAuxRelTypeError},
        *,
    },
};

/// An implementation of [`KnownAuxRelType`], that knows
/// `acl`, `describedby`, rel types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultKnownAuxRelType(RelationType);

impl Deref for DefaultKnownAuxRelType {
    type Target = RelationType;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<RelationType> for DefaultKnownAuxRelType {
    #[inline]
    fn borrow(&self) -> &RelationType {
        &self.0
    }
}

/// List of known rel types.
static KNOWN_RAW_AUX_REL_TYPES: &[&Lazy<RelationType>] = &[&ACL_REL_TYPE, &DESCRIBED_BY_REL_TYPE];

/// Set of all default known aux rel types.
pub static ALL_KNOWN_AUX_REL_TYPES: Lazy<HashSet<DefaultKnownAuxRelType>> = Lazy::new(|| {
    KNOWN_RAW_AUX_REL_TYPES
        .iter()
        .map(|aux_rel_type| DefaultKnownAuxRelType(Lazy::force(*aux_rel_type).clone()))
        .collect()
});

impl TryFrom<RelationType> for DefaultKnownAuxRelType {
    type Error = UnknownAuxRelTypeError;

    fn try_from(rel_type: RelationType) -> Result<Self, Self::Error> {
        if KNOWN_RAW_AUX_REL_TYPES
            .iter()
            .any(|known| (*known).deref() == &rel_type)
        {
            Ok(Self(rel_type))
        } else {
            Err(UnknownAuxRelTypeError(rel_type))
        }
    }
}

impl From<DefaultKnownAuxRelType> for RelationType {
    #[inline]
    fn from(val: DefaultKnownAuxRelType) -> Self {
        val.0
    }
}

impl KnownAuxRelType for DefaultKnownAuxRelType {
    /// Allows only atoms as targets for all aux links.
    #[inline]
    fn target_res_kind(&self) -> SolidResourceKind {
        SolidResourceKind::NonContainer
    }

    fn allowed_subject_res_kinds(&self) -> &'static [SolidResourceKind] {
        // Restrict container as subject for container index aux link.
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

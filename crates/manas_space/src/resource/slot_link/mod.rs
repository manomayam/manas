//! I define [`SlotLink`] type.

use std::ops::Deref;

use iri_string::types::UriReferenceStr;
use manas_http::header::link::{LinkRel, LinkTarget, LinkValue};

use super::{
    slot_rel_type::{aux_rel_type::ACL_REL_TYPE, SlotRelationType, SpcSlotRelType},
    uri::SolidResourceUri,
};
use crate::{policy::aux::AuxPolicy, SolidStorageSpace, SpcKnownAuxRelType};

/// A struct representing a slot link from host
/// resource to a contained or an auxiliary resource.
#[derive(Debug, Clone)]
pub struct SlotLink<Space>
where
    Space: SolidStorageSpace,
{
    /// Target of link.
    pub target: SolidResourceUri,

    /// rel type.
    pub rel_type: SpcSlotRelType<Space>,
}

/// An invariant of [`SlotLink`], that is guaranteed to be an
/// aux link.
#[derive(Debug, Clone)]
pub struct AuxLink<Space>(SlotLink<Space>)
where
    Space: SolidStorageSpace;

impl<Space> AuxLink<Space>
where
    Space: SolidStorageSpace,
{
    /// Create a new [`AuxLink`].
    #[inline]
    pub fn new(
        target: SolidResourceUri,
        aux_rel_type: <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType,
    ) -> Self {
        Self(SlotLink {
            target,
            rel_type: SlotRelationType::Auxiliary(aux_rel_type),
        })
    }

    /// Get the aux rel type.
    pub fn aux_rel_type(&self) -> &SpcKnownAuxRelType<Space> {
        match &self.0.rel_type {
            SlotRelationType::Contains => unreachable!("Invariant ensures"),
            SlotRelationType::Auxiliary(rel_type) => rel_type,
        }
    }

    /// Check if given aux link is an acl link
    #[inline]
    pub fn is_acl_link(&self) -> bool {
        self.aux_rel_type().deref() == &*ACL_REL_TYPE
    }
}

impl<Space> From<AuxLink<Space>> for SlotLink<Space>
where
    Space: SolidStorageSpace,
{
    #[inline]
    fn from(val: AuxLink<Space>) -> Self {
        val.0
    }
}

impl<Space> TryFrom<SlotLink<Space>> for AuxLink<Space>
where
    Space: SolidStorageSpace,
{
    type Error = NotAnAuxSlotLink;

    #[inline]
    fn try_from(value: SlotLink<Space>) -> Result<Self, Self::Error> {
        match &value.rel_type {
            SlotRelationType::Contains => Err(NotAnAuxSlotLink),
            SlotRelationType::Auxiliary(_) => Ok(Self(value)),
        }
    }
}

impl<Space> Deref for AuxLink<Space>
where
    Space: SolidStorageSpace,
{
    type Target = SlotLink<Space>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Space> From<AuxLink<Space>> for LinkValue
where
    Space: SolidStorageSpace,
{
    #[inline]
    fn from(val: AuxLink<Space>) -> Self {
        let aux_rel_type = val.0.rel_type.into();

        LinkValue::new(
            LinkTarget(AsRef::<UriReferenceStr>::as_ref(val.0.target.deref()).to_owned()),
            LinkRel::new(aux_rel_type),
            None,
        )
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("Given slot link is not an aux link.")]
/// Error of a slot-link being non aux-link.
pub struct NotAnAuxSlotLink;

#[cfg(test)]
mod tests {
    // TODO
}

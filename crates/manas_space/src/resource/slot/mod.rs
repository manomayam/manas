//! I define types to represent resource slots in solid storage
//! space.
//!

use std::sync::Arc;

use super::{
    kind::SolidResourceKind,
    slot_id::SolidResourceSlotId,
    slot_rel_type::{
        aux_rel_type::{known::KnownAuxRelType, ACL_REL_TYPE},
        SlotRelationType,
    },
    slot_rev_link::SlotRevLink,
};
use crate::{SolidStorageSpace, SpcKnownAuxRelType};

pub mod invariant;
pub mod predicate;

/// A resource in a solid storage space has a unique slot
/// characterized by the product of resource slot id, resource
/// kind, and it's slot reverse link.
#[derive(Debug, Clone, PartialEq)]
pub struct SolidResourceSlot<Space>
where
    Space: SolidStorageSpace,
{
    /// Slot id.
    id: SolidResourceSlotId<Space>,

    /// Kind of the resource.
    res_kind: SolidResourceKind,

    /// Slot reverse link of the resource.
    slot_rev_link: Option<SlotRevLink<Space>>,
}

/// Invalid solid resource slot.
#[derive(Debug, thiserror::Error)]
pub enum InvalidSolidResourceSlot {
    /// Invalid root resource kind.
    #[error("Invalid root resource kind.")]
    InvalidRootResourceKind,

    /// Invalid root resource host.
    #[error("Invalid root resource host.")]
    InvalidRootResourceHost,
}

impl<Space: SolidStorageSpace> SolidResourceSlot<Space> {
    /// Get storage space of the slot.
    #[inline]
    pub fn space(&self) -> &Arc<Space> {
        &self.id.space
    }

    /// Get id of the slot.
    #[inline]
    pub fn id(&self) -> &SolidResourceSlotId<Space> {
        &self.id
    }

    /// Get resource kind.
    #[inline]
    pub fn res_kind(&self) -> SolidResourceKind {
        self.res_kind
    }

    /// Get resource's slot reverse link.
    #[inline]
    pub fn slot_rev_link(&self) -> Option<&SlotRevLink<Space>> {
        self.slot_rev_link.as_ref()
    }

    /// Get resource's slot reverse link rel type.
    #[inline]
    pub fn prov_rev_rel_type(&self) -> Option<&SlotRelationType<SpcKnownAuxRelType<Space>>> {
        if let Some(link) = self.slot_rev_link.as_ref() {
            Some(&link.rev_rel_type)
        } else {
            None
        }
    }

    /// Get slot id of the host resource.
    pub fn host_slot_id(&self) -> Option<SolidResourceSlotId<Space>> {
        self.slot_rev_link
            .as_ref()
            .map(|rev_link| SolidResourceSlotId {
                space: self.space().clone(),
                uri: rev_link.target.clone(),
            })
    }

    /// Try to create a new slot from params.
    /// Returns error if storage root invariants are violated.
    pub fn try_new(
        id: SolidResourceSlotId<Space>,
        res_kind: SolidResourceKind,
        slot_rev_link: Option<SlotRevLink<Space>>,
    ) -> Result<Self, InvalidSolidResourceSlot> {
        if id.is_root_slot_id() {
            if res_kind == SolidResourceKind::NonContainer {
                return Err(InvalidSolidResourceSlot::InvalidRootResourceKind);
            }
            if slot_rev_link.is_some() {
                return Err(InvalidSolidResourceSlot::InvalidRootResourceHost);
            }
        }

        Ok(Self {
            id,
            res_kind,
            slot_rev_link,
        })
    }

    /// Get root slot of given space.
    #[inline]
    pub fn root_slot(space: Arc<Space>) -> Self {
        Self {
            id: SolidResourceSlotId::root_slot_id(space),
            res_kind: SolidResourceKind::Container,
            slot_rev_link: None,
        }
    }

    /// Get if the slot is root resource slot.
    #[inline]
    pub fn is_root_slot(&self) -> bool {
        self.id.is_root_slot_id()
    }

    /// Get if the slot is root acl slot.
    #[inline]
    pub fn is_root_acl_slot(&self) -> bool {
        self.slot_rev_link
            .as_ref()
            .map(|rev_link| {
                // If slot rev link's target is storage root,
                (&rev_link.target == self.id.space.root_res_uri())
                // And slot relation is aux/acl.
                && (rev_link
                    .rev_rel_type
                    .is_auxiliary_of_type(&ACL_REL_TYPE))
            })
            .unwrap_or(false)
    }

    /// Get if slot is that of a contained resource.
    pub fn is_contained_slot(&self) -> bool {
        self.prov_rev_rel_type()
            .map(|rev_rel_type| rev_rel_type.is_contains())
            .unwrap_or(false)
    }

    /// Get if slot is that of an auxiliary resource.
    pub fn is_aux_slot(&self) -> bool {
        self.prov_rev_rel_type()
            .map(|rev_rel_type| rev_rel_type.is_auxiliary())
            .unwrap_or(false)
    }

    /// Get if slot is that of an rdf source aux resource.
    #[inline]
    pub fn is_rdf_source_aux_res_slot(&self) -> bool {
        if let Some(SlotRelationType::Auxiliary(aux_rel_tye)) = self.prov_rev_rel_type() {
            if aux_rel_tye.target_is_rdf_source() {
                return true;
            }
        }
        false
    }

    /// Get if slot is that of a container.
    #[inline]
    pub fn is_container_slot(&self) -> bool {
        self.res_kind == SolidResourceKind::Container
    }
}

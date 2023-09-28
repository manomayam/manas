//! I define types to represent slot paths.
//!

use std::{borrow::Cow, fmt::Debug, sync::Arc};

use if_chain::if_chain;

use super::{
    kind::SolidResourceKind, slot::SolidResourceSlot, slot_rel_type::SlotRelationType,
    uri::SolidResourceUri,
};
use crate::{
    policy::aux::AuxPolicy, resource::slot_rel_type::aux_rel_type::known::KnownAuxRelType,
    RelativeSolidStorageSpace, SolidStorageSpace,
};

pub mod invariant;
pub mod predicate;

/// A struct to represent valid slot path of a resource in
/// a storage space.
#[derive(Debug, Clone)]
pub struct SolidResourceSlotPath<'p, Space: SolidStorageSpace>(Cow<'p, [SolidResourceSlot<Space>]>);

impl<'p, Space: SolidStorageSpace> SolidResourceSlotPath<'p, Space> {
    /// Get a borrowed version.
    #[inline]
    pub fn to_borrowed(&self) -> SolidResourceSlotPath<'_, Space> {
        SolidResourceSlotPath(Cow::Borrowed(self.0.as_ref()))
    }

    /// Convert into a owned version.
    #[inline]
    pub fn into_owned(self) -> SolidResourceSlotPath<'static, Space> {
        SolidResourceSlotPath(Cow::Owned(self.0.into_owned()))
    }

    /// Get the storage space
    #[inline]
    pub fn space(&self) -> &Arc<Space> {
        self.0[0].space()
    }

    /// Get slot of the path target resource..
    #[inline]
    pub fn target_res_slot(&self) -> &SolidResourceSlot<Space> {
        &self.0[self.0.len() - 1]
    }

    /// Get uri of the path target resource..
    #[inline]
    pub fn target_res_uri(&self) -> &SolidResourceUri {
        &self.target_res_slot().id().uri
    }

    /// Get slots in slot path..
    #[inline]
    pub fn slots(&self) -> &[SolidResourceSlot<Space>] {
        &self.0
    }

    /// Create a new [`SolidResourceSlotPath`] from given
    /// slots without any checks.
    ///
    /// # Safety
    /// Callers must ensure that supplied slots can form a
    /// valid slot path.
    #[inline]
    pub unsafe fn new_unchecked(slots: impl Into<Cow<'p, [SolidResourceSlot<Space>]>>) -> Self {
        Self(slots.into())
    }

    /// Try to create a new [`SolidResourceSlotPath`] from given slots.
    pub fn try_new(
        slots: impl Into<Cow<'p, [SolidResourceSlot<Space>]>>,
    ) -> Result<Self, InvalidResourceSlotPath> {
        let slots: Cow<'p, [SolidResourceSlot<Space>]> = slots.into();

        // Ensure there is at least root slot.
        if slots.is_empty() {
            return Err(InvalidResourceSlotPath::InvalidRootSlot);
        }

        let root_slot = &slots[0];
        let space = root_slot.space();
        if !root_slot.is_root_slot() {
            return Err(InvalidResourceSlotPath::InvalidRootSlot);
        }

        let mut aux_link_count = 0;
        let mut slots_iter = slots.iter().peekable();

        while let Some(slot) = slots_iter.next() {
            if slot.space() != space {
                return Err(InvalidResourceSlotPath::InconsistentStorageSpace);
            }

            if let Some(next_slot) = slots_iter.peek() {
                // Ensure next item has a slot rev link.
                let next_slot_rev_link = next_slot
                    .slot_rev_link()
                    .ok_or(InvalidResourceSlotPath::InconsistentSlotLinks)?;

                // Ensure valid target.
                if next_slot_rev_link.target != slot.id().uri {
                    return Err(InvalidResourceSlotPath::InconsistentSlotLinks);
                }

                // Ensure aux rel constraints.
                if let SlotRelationType::Auxiliary(aux_rel_type) = &next_slot_rev_link.rev_rel_type
                {
                    // Ensure aux links count is in limits.
                    aux_link_count += 1;
                    if_chain! {
                        if let Some(max_aux_links) =  <Space::AuxPolicy as AuxPolicy>::PROV_PATH_MAX_AUX_LINKS;
                        if aux_link_count >= usize::from(max_aux_links);

                        then {
                            return Err(InvalidResourceSlotPath::AuxLinksCountOutOfLimit);
                        }
                    }

                    //Ensure subject constraints are honoured.
                    if !aux_rel_type
                        .allowed_subject_res_kinds()
                        .contains(&slot.res_kind())
                    {
                        return Err(InvalidResourceSlotPath::AuxLinkSubjectConstraintsViolation);
                    }

                    // Ensure aux rel target constraints are upheld.
                    if aux_rel_type.target_res_kind() != next_slot.res_kind() {
                        return Err(InvalidResourceSlotPath::AuxLinkTargetConstraintsViolation);
                    }
                } else {
                    //Ensure containment subject constraints are honoured.
                    if slot.res_kind() != SolidResourceKind::Container {
                        return Err(
                            InvalidResourceSlotPath::ContainmentLinkSubjectConstraintsViolation,
                        );
                    }
                }
            }
        }

        Ok(Self(slots))
    }

    /// Get slot path of the storage root.
    #[inline]
    pub fn root_slot_path(space: Arc<Space>) -> SolidResourceSlotPath<'static, Space> {
        SolidResourceSlotPath(vec![SolidResourceSlot::root_slot(space)].into())
    }

    /// Split the slot path at end.
    pub fn rsplit(self) -> (Option<Self>, SolidResourceSlot<Space>) {
        match self.0 {
            Cow::Borrowed(slots) => {
                let host_slot_path = &slots[..slots.len() - 1];
                (
                    if host_slot_path.is_empty() {
                        None
                    } else {
                        Some(Self(Cow::Borrowed(host_slot_path)))
                    },
                    self.target_res_slot().clone(),
                )
            }
            Cow::Owned(mut slots) => {
                let last = slots.pop().expect("Must be some, as invariant guarantees.");
                (
                    if slots.is_empty() {
                        None
                    } else {
                        Some(Self(Cow::Owned(slots)))
                    },
                    last,
                )
            }
        }
    }
}

/// Invalid resource slot path.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum InvalidResourceSlotPath {
    /// Invalid root slot.
    #[error("Invalid root slot.")]
    InvalidRootSlot,

    /// Storage space is inconsistent across slots.
    #[error("Storage space is inconsistent across slots.")]
    InconsistentStorageSpace,

    /// Slot links are inconsistent.
    #[error("Slot links are inconsistent.")]
    InconsistentSlotLinks,

    /// Containment link subject constraints violation.
    #[error("Containment rel subject constraints violation.")]
    ContainmentLinkSubjectConstraintsViolation,

    /// Aux link subject constraints violation.
    #[error("Aux link subject constraints violation.")]
    AuxLinkSubjectConstraintsViolation,

    /// Aux link target constraints violation.
    #[error("Aux link target constraints violation.")]
    AuxLinkTargetConstraintsViolation,

    /// Aux links count out of limit.
    #[error("Aux links count out of limit.")]
    AuxLinksCountOutOfLimit,
}

/// A type alias for relative resource slot path.
pub type RelativeSolidResourceSlotPath<'p, Space> =
    SolidResourceSlotPath<'p, RelativeSolidStorageSpace<Space>>;

#[cfg(feature = "test-utils")]
/// I define utilities for easily mocking with [`SolidResourceSlotPath`].
pub mod mock {
    use std::{borrow::Cow, sync::Arc};

    use super::SolidResourceSlotPath;
    use crate::{
        resource::{
            kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
            slot_rel_type::mock::SlotRelationTypeHint, slot_rev_link::SlotRevLink,
            uri::SolidResourceUri,
        },
        SolidStorageSpace,
    };

    impl<Space: SolidStorageSpace> SolidResourceSlotPath<'static, Space> {
        /// Create a new mock simple resource slot path with
        /// given params.
        pub fn new_mock(
            space: Arc<Space>,
            slot_path_hint: Option<Vec<(SlotRelationTypeHint, &str, SolidResourceKind)>>,
        ) -> Self {
            let mut slots = vec![SolidResourceSlot::root_slot(space.clone())];

            if let Some(hint) = slot_path_hint {
                let hint_iter = hint.into_iter().peekable();

                for (slot_rev_rel_type_hint, uri, res_kind) in hint_iter {
                    slots.push(
                        SolidResourceSlot::try_new(
                            SolidResourceSlotId {
                                space: space.clone(),
                                uri: SolidResourceUri::try_new_from(uri)
                                    .expect("Claimed valid slot path hint."),
                            },
                            res_kind,
                            Some(SlotRevLink {
                                target: slots.last().unwrap().id().uri.clone(),
                                rev_rel_type: slot_rev_rel_type_hint
                                    .assert_valid_hint("Claimed valid slot path."),
                            }),
                        )
                        .expect("Claimed valid slot path"),
                    )
                }
            }

            SolidResourceSlotPath::try_new(Cow::Owned(slots)).expect("Claimed valid slot path.")
        }
    }
}

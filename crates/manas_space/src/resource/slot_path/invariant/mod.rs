//! I define few invariants of [`SolidResourceSlotPath`](super::SolidResourceSlotPath).
//!

use std::{ops::Deref, sync::Arc};

use gdp_rs::{
    proven::{ProvenError, TryProven},
    Proven,
};
use iri_string::{
    format::ToDedicatedString,
    types::{UriRelativeStr, UriStr},
};
use manas_http::uri::invariant::NormalAbsoluteHttpUri;
use vec1::Vec1;

use super::{
    predicate::{IsNotSansAuxLink, IsSansAuxLink},
    SolidResourceSlotPath,
};
use crate::{
    resource::{
        kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
        slot_rel_type::SlotRelationType, slot_rev_link::SlotRevLink, uri::SolidResourceUri,
    },
    RelativeSolidStorageSpace, SolidStorageSpace,
};

/// An invariant of [`SolidResourceSlotPath`] that represents slot
/// paths with out any aux links.
#[derive(Debug, Clone)]
pub struct SansAuxLinkResourceSlotPath<'p, Space: SolidStorageSpace>(
    pub Proven<SolidResourceSlotPath<'p, Space>, IsSansAuxLink>,
);

impl<'p, Space: SolidStorageSpace> TryFrom<SolidResourceSlotPath<'p, Space>>
    for SansAuxLinkResourceSlotPath<'p, Space>
{
    type Error = ProvenError<SolidResourceSlotPath<'p, Space>, IsNotSansAuxLink>;

    #[inline]
    fn try_from(slot_path: SolidResourceSlotPath<'p, Space>) -> Result<Self, Self::Error> {
        Ok(Self(slot_path.try_proven()?))
    }
}

impl<'p, Space: SolidStorageSpace> From<SansAuxLinkResourceSlotPath<'p, Space>>
    for SolidResourceSlotPath<'p, Space>
{
    #[inline]
    fn from(value: SansAuxLinkResourceSlotPath<'p, Space>) -> Self {
        value.0.into_subject()
    }
}

impl<'p, Space: SolidStorageSpace> Deref for SansAuxLinkResourceSlotPath<'p, Space> {
    type Target = SolidResourceSlotPath<'p, Space>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Space: SolidStorageSpace> SansAuxLinkResourceSlotPath<'static, Space> {
    /// Get a new [`SansAuxLinkResourceSlotPath`] using given hint.
    #[inline]
    pub fn new(
        space: Arc<Space>,
        opt_containment_path_hint: Option<(Vec1<SolidResourceUri>, SolidResourceKind)>,
    ) -> Self {
        let mut slots = vec![SolidResourceSlot::root_slot(space.clone())];

        if let Some(hint) = opt_containment_path_hint {
            let mut iter_host_res_uri = space.root_res_uri().clone();
            let mut hint_iter = hint.0.into_iter().peekable();

            while let Some(iter_res_uri) = hint_iter.next() {
                slots.push(
                    SolidResourceSlot::try_new(
                        SolidResourceSlotId::new(space.clone(), iter_res_uri.clone()),
                        if hint_iter.peek().is_some() {
                            SolidResourceKind::Container
                        } else {
                            hint.1
                        },
                        Some(SlotRevLink {
                            rev_rel_type: SlotRelationType::Contains,
                            target: iter_host_res_uri,
                        }),
                    )
                    .expect("Must be valid"),
                );

                iter_host_res_uri = iter_res_uri;
            }
        }

        Self(
            SolidResourceSlotPath::try_new(slots)
                .expect("Must be valid")
                .try_proven()
                .expect("Must be sans aux link"),
        )
    }

    /// Decode contained slot path from resource slot id
    /// using hierarchical uri semantics.
    pub fn decode_with_hierarchical_uri_semantics(
        res_slot_id: SolidResourceSlotId<Space>,
    ) -> Result<Self, InvalidHierarchicalResourceSlotId> {
        let root_res_uri = res_slot_id.space.root_res_uri();

        if res_slot_id.uri.query().is_some()
            || !root_res_uri.as_str().ends_with('/')
            || !res_slot_id.uri.as_str().starts_with(root_res_uri.as_str())
        {
            return Err(InvalidHierarchicalResourceSlotId);
        }

        // Decode resource kind from uri's trailing slash
        // info.
        let res_kind = if res_slot_id.uri.as_str().ends_with('/') {
            SolidResourceKind::Container
        } else {
            SolidResourceKind::NonContainer
        };

        let double_dot_ref = UriRelativeStr::new("..").expect("Must be valid");
        let dot_ref = UriRelativeStr::new(".").expect("Must be valid");

        let mut slot_path_uris = vec![];

        let mut iter_res_uri = res_slot_id.uri;

        loop {
            // If iter res uri is storage root uri, exit the loop.
            if &iter_res_uri == root_res_uri {
                break;
            }

            // Decode host res uri, using hierarchical uri
            // semantics
            let host_res_uri = NormalAbsoluteHttpUri::try_new_from(
                if iter_res_uri.as_str().ends_with('/') {
                    double_dot_ref
                } else {
                    dot_ref
                }
                .resolve_against(iter_res_uri.to_absolute())
                .to_dedicated_string()
                .as_ref() as &UriStr,
            )
            .expect("Must be normalized, as child res uri is normalized.");

            slot_path_uris.push(iter_res_uri);
            iter_res_uri = host_res_uri;
        }

        slot_path_uris.reverse();

        Ok(Self::new(
            res_slot_id.space,
            slot_path_uris.try_into().ok().map(|uris| (uris, res_kind)),
        ))
    }

    /// Split the slot path at end.
    pub fn rsplit(self) -> (Option<Self>, SolidResourceSlot<Space>) {
        let inner_result = self.0.into_subject().rsplit();
        (
            inner_result
                .0
                .map(|p| Self(p.try_proven().expect("Must be valid"))),
            inner_result.1,
        )
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid hierarchical resource slot id.")]
/// Invalid hierarchical resource slot id.
pub struct InvalidHierarchicalResourceSlotId;

///Alias for type of sans-aux-link resource slot path in a
/// relative space.
pub type SansAuxLinkRelativeResourceSlotPath<'p, Space> =
    SansAuxLinkResourceSlotPath<'p, RelativeSolidStorageSpace<Space>>;

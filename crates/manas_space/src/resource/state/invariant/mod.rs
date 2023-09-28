//! I define few invariants of [`SolidResourceState`].
//!

use std::ops::Deref;

use gdp_rs::{
    proven::{ProvenError, TryProven},
    Proven,
};
use manas_http::representation::{metadata::RepresentationMetadata, Representation};

use super::{
    predicate::{IsNotRepresentedResourceState, IsRepresented},
    SolidResourceState,
};
use crate::{resource::slot::SolidResourceSlot, SolidStorageSpace};

/// A type to represent represented invariant of [`SolidResourceState`].
#[derive(Debug, Clone)]
pub struct RepresentedSolidResourceState<Space: SolidStorageSpace, Rep>(
    pub Proven<SolidResourceState<Space, Rep>, IsRepresented>,
);

impl<Space: SolidStorageSpace, Rep> TryFrom<SolidResourceState<Space, Rep>>
    for RepresentedSolidResourceState<Space, Rep>
{
    type Error = ProvenError<SolidResourceState<Space, Rep>, IsNotRepresentedResourceState>;

    #[inline]
    fn try_from(state: SolidResourceState<Space, Rep>) -> Result<Self, Self::Error> {
        Ok(Self(state.try_proven()?))
    }
}

impl<Space: SolidStorageSpace, Rep> Deref for RepresentedSolidResourceState<Space, Rep> {
    type Target = SolidResourceState<Space, Rep>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<Space, Rep> RepresentedSolidResourceState<Space, Rep>
where
    Space: SolidStorageSpace,
    Rep: Representation,
{
    /// Get the representation of the represented resource.
    #[inline]
    pub fn representation(&self) -> &Rep {
        self.representation
            .as_ref()
            .expect("Must be some, as invariant guarantees.")
    }

    /// Get the representation metadata of the represented resource.
    #[inline]
    pub fn representation_metadata(&self) -> &RepresentationMetadata {
        self.representation().metadata()
    }

    /// Convert into inner state.
    #[inline]
    pub fn into_inner(self) -> SolidResourceState<Space, Rep> {
        self.0.into_subject()
    }

    /// Convert the state into it's parts.
    #[inline]
    pub fn into_parts(self) -> (SolidResourceSlot<Space>, Rep) {
        let inner = self.into_inner();
        (
            inner.slot,
            inner
                .representation
                .expect("Must be some, as invariant guarantees."),
        )
    }

    /// Map the representation.
    #[inline]
    pub fn map_representation<Rep2, F>(self, f: F) -> RepresentedSolidResourceState<Space, Rep2>
    where
        F: FnOnce(Rep) -> Rep2,
        Rep2: Representation,
    {
        let (slot, representation) = self.into_parts();
        RepresentedSolidResourceState::new(slot, f(representation))
    }

    /// Create a new [`RepresentedSolidResourceState`] with
    /// given params.
    #[inline]
    pub fn new(slot: SolidResourceSlot<Space>, representation: Rep) -> Self {
        Self(
            SolidResourceState {
                slot,
                representation: Some(representation),
            }
            .try_proven()
            .expect("Must be ok, as representation is some."),
        )
    }
}

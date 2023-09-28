//! I define types to represent state of a solid resource.
//!

use manas_http::representation::{metadata::RepresentationMetadata, Representation};

use super::{slot::SolidResourceSlot, uri::SolidResourceUri};
use crate::SolidStorageSpace;

pub mod invariant;
pub mod predicate;

/// A struct for representing the state of a solid resource.
#[derive(Debug, Clone)]
pub struct SolidResourceState<Space, Rep>
where
    Space: SolidStorageSpace,
{
    /// Slot of the resource.
    pub slot: SolidResourceSlot<Space>,

    /// Optional representation of the resource.
    pub representation: Option<Rep>,
}

impl<Space, Rep> SolidResourceState<Space, Rep>
where
    Space: SolidStorageSpace,
    Rep: Representation,
{
    /// Get uri of the resource.
    #[inline]
    pub fn uri(&self) -> &SolidResourceUri {
        &self.slot.id().uri
    }

    // /// Map representation data.
    // #[inline]
    // pub fn map_rep_data<TRepData, F>(self, f: F) -> SolidResourceState<Space, TRepData>
    // where
    //     F: FnOnce(Rep) -> TRepData,
    // {
    //     SolidResourceState {
    //         slot: self.slot,
    //         representation: self.representation.map(|rep| rep.map_data(f)),
    //     }
    // }

    /// Get a reference to optional representation metadata
    #[inline]
    pub fn representation_metadata(&self) -> Option<&RepresentationMetadata> {
        self.representation.as_ref().map(|rep| rep.metadata())
    }
}

//! I define type to represent in-memory quads data.
//!

use std::ops::{Deref, DerefMut};

use rdf_utils::model::{dataset::EcoDataset, quad::ArcQuad};
use sophia_api::prelude::Dataset;

/// Quads inmem data.
#[derive(Debug, Clone, Default)]
pub struct QuadsInmem<D: Dataset>(pub D);

impl<D: Dataset> QuadsInmem<D> {
    /// Create a new [`QuadsInmem`] with given dataset.
    #[inline]
    pub fn new(dataset: D) -> Self {
        Self(dataset)
    }

    /// Get inner dataset.
    #[inline]
    pub fn into_inner(self) -> D {
        self.0
    }
}

impl<D: Dataset> Deref for QuadsInmem<D> {
    type Target = D;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<D: Dataset> DerefMut for QuadsInmem<D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Type of [`QuadsInmem`] backed by an ecovec.
pub type EcoQuadsInmem = QuadsInmem<EcoDataset<ArcQuad>>;

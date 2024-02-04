//! I provide an implementation of [`ODRObjectStoreBackend`]
//! with gcs backend.
//!

use flagset::FlagSet;
use opendal::{services::Gcs, Operator};

use crate::object_store::backend::{
    path_es::impl_::identical::IdenticalBackendObjectPathEncodingScheme, BackendExtraCapability,
    BuildableODRObjectStoreBackend, ODRObjectStoreBackend,
};

// use super::common::stat_fix_layer::StatFixLayer;

/// An implementation of [`ODRObjectStoreBackend`]
/// with gcs backend.
#[derive(Debug, Clone)]
pub struct GcsBackend {
    operator: Operator,
}

impl ODRObjectStoreBackend for GcsBackend {
    type ObjectPathEncodingScheme = IdenticalBackendObjectPathEncodingScheme;

    #[inline]
    fn operator(&self) -> &Operator {
        &self.operator
    }

    #[inline]
    fn extra_caps(&self) -> FlagSet<BackendExtraCapability> {
        BackendExtraCapability::ProvidesObjectValidators
            | BackendExtraCapability::HasIndependentDirObjects
            | BackendExtraCapability::SupportsNativeContentTypeMetadata
    }
}

impl TryFrom<Gcs> for GcsBackend {
    type Error = opendal::Error;

    #[inline]
    fn try_from(builder: Gcs) -> Result<Self, Self::Error> {
        Ok(Self {
            operator: Operator::new(builder)?
                // .layer(StatFixLayer)
                .finish(),
        })
    }
}

impl BuildableODRObjectStoreBackend<Gcs> for GcsBackend {}

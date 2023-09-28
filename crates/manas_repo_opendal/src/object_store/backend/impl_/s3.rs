//! I provide an implementation of [`ODRObjectStoreBackend`]
//! with s3 backend.
//!

use flagset::FlagSet;
use opendal::{services::S3, Operator};

use crate::object_store::backend::{
    path_es::impl_::identical::IdenticalBackendObjectPathEncodingScheme, BackendExtraCapability,
    BuildableODRObjectStoreBackend, ODRObjectStoreBackend,
};

use super::common::stat_fix_layer::StatFixLayer;

/// An implementation of [`ODRObjectStoreBackend`]
/// with s3 backend.
#[derive(Debug, Clone)]
pub struct S3Backend {
    operator: Operator,
}

impl ODRObjectStoreBackend for S3Backend {
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

impl TryFrom<S3> for S3Backend {
    type Error = opendal::Error;

    #[inline]
    fn try_from(builder: S3) -> Result<Self, Self::Error> {
        Ok(Self {
            operator: Operator::new(builder)?.layer(StatFixLayer).finish(),
        })
    }
}

impl BuildableODRObjectStoreBackend<S3> for S3Backend {}

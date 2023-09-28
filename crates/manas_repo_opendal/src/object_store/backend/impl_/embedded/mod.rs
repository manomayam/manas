//! I provide an implementation of [`ODRObjectStoreBackend`]
//! with embedded backend.
//!

use flagset::FlagSet;
use opendal::Operator;
use rust_embed::RustEmbed;

use self::service::Embedded;
use crate::object_store::backend::{
    path_es::impl_::pct_decoded::PctDecodedBackendObjectPathEncodingScheme, BackendExtraCapability,
    BuildableODRObjectStoreBackend, ODRObjectStoreBackend,
};

pub mod service;

/// An implementation of [`ODRObjectStoreBackend`]
/// with embedded backend.
#[derive(Debug, Clone)]
pub struct EmbeddedBackend {
    operator: Operator,
}

impl ODRObjectStoreBackend for EmbeddedBackend {
    type ObjectPathEncodingScheme = PctDecodedBackendObjectPathEncodingScheme;

    #[inline]
    fn operator(&self) -> &Operator {
        &self.operator
    }

    #[inline]
    fn extra_caps(&self) -> FlagSet<BackendExtraCapability> {
        BackendExtraCapability::HasIndependentDirObjects
            | BackendExtraCapability::ProvidesObjectValidators
            | BackendExtraCapability::SupportsNativeContentTypeMetadata
    }
}

impl<Assets: RustEmbed + 'static> TryFrom<Embedded<Assets>> for EmbeddedBackend {
    type Error = opendal::Error;

    #[inline]
    fn try_from(builder: Embedded<Assets>) -> Result<Self, Self::Error> {
        Ok(Self {
            operator: Operator::new(builder)?.finish(),
        })
    }
}

impl<Assets: RustEmbed + 'static> BuildableODRObjectStoreBackend<Embedded<Assets>>
    for EmbeddedBackend
{
}

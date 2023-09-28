//! I provide an implementation of [`ODRObjectStoreBackend`]
//! with file system backend.
//!

use flagset::FlagSet;
use opendal::{services::Fs, Operator};

use crate::object_store::backend::{
    path_es::impl_::pct_decoded::PctDecodedBackendObjectPathEncodingScheme, BackendExtraCapability,
    BuildableODRObjectStoreBackend, ODRObjectStoreBackend,
};

/// An implementation of [`ODRObjectStoreBackend`]
/// with file system backend.
#[derive(Debug, Clone)]
pub struct FsBackend {
    operator: Operator,
}

impl ODRObjectStoreBackend for FsBackend {
    type ObjectPathEncodingScheme = PctDecodedBackendObjectPathEncodingScheme;

    #[inline]
    fn operator(&self) -> &Operator {
        &self.operator
    }

    #[inline]
    fn extra_caps(&self) -> FlagSet<BackendExtraCapability> {
        BackendExtraCapability::ProvidesObjectValidators.into()
    }
}

impl TryFrom<Fs> for FsBackend {
    type Error = opendal::Error;

    #[inline]
    fn try_from(builder: Fs) -> Result<Self, Self::Error> {
        Ok(Self {
            operator: Operator::new(builder)?.finish(),
        })
    }
}

impl BuildableODRObjectStoreBackend<Fs> for FsBackend {}

// impl TryFrom<FsBackendConfig> for FsBackend {
//     type Error = opendal::Error;

//     #[inline]
//     fn try_from(config: FsBackendConfig) -> Result<Self, Self::Error> {
//         Fs::from(&config).try_into()
//     }
// }

// /// Configuration for [`FsBackend`].
// #[derive(Debug, Clone)]
// pub struct FsBackendConfig {
//     /// Root directory for the backend.
//     pub root_dir: String,

//     /// Atomic write dir.
//     pub atomic_write_dir: Option<String>,
// }

// impl From<&FsBackendConfig> for Fs {
//     fn from(config: &FsBackendConfig) -> Self {
//         let mut builder = Fs::default();
//         builder.root(&config.root_dir);

//         if let Some(awd) = &config.atomic_write_dir {
//             builder.atomic_write_dir(awd);
//         }
//         builder
//     }
// }

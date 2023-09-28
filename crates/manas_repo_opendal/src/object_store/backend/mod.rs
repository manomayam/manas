//! I define types and traits to define backend paths, capabilities, etc.
//!

use std::fmt::Debug;

use flagset::{flags, FlagSet};
use opendal::Operator;

use self::path_es::ODRBackendObjectPathEncodingScheme;

pub mod impl_;
pub mod path_es;

/// A trait for odr object store backends.
///
/// # Safety
///
/// Implementations must ensure that backend indeed has
/// specified extra capabilities.
///
/// And it must be ensured that backend can support all paths
/// encoded by the specified path encoding scheme.
///
pub trait ODRObjectStoreBackend: Send + Sync + 'static + Clone + Debug {
    /// Type of backend object path encoding scheme.
    type ObjectPathEncodingScheme: ODRBackendObjectPathEncodingScheme;

    /// Get the backend operator.
    fn operator(&self) -> &Operator;

    /// Get the extra capabilities of the backend.
    fn extra_caps(&self) -> FlagSet<BackendExtraCapability>;
}

/// A trait for [`ODRObjectStoreBackend`] that can be built
/// from an opendal service builder.
pub trait BuildableODRObjectStoreBackend<Builder: opendal::Builder>:
    ODRObjectStoreBackend + TryFrom<Builder, Error = opendal::Error>
{
}

flags! {
    /// A flag representing backend's extra capabilities.
    pub enum BackendExtraCapability: u8 {
        /// Whether dir objects managed by backend are independent.
        /// Independent dir objects can be deleted
        /// with out deleting objects in it's namespace.
        ///
        /// This is required for solid-level atomic delete.
        HasIndependentDirObjects,

        /// Whether backend provides last_modified timestamp
        /// or version etag validators for the objects.
        ///
        /// Needed for solid-level atomic update in the case of
        /// content-type migration in few backends.
        ProvidesObjectValidators,

        /// Whether backend supports native content-type metadata.
        SupportsNativeContentTypeMetadata,
    }
}

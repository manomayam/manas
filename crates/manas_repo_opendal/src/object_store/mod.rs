//! I define types for defining
//! object spaces and object stores for ODR.
//!

use std::{collections::HashMap, fmt::Debug};

use manas_space::resource::uri::SolidResourceUri;
use opendal::Entry;
pub use setup::ODRObjectStoreSetup;

use self::{
    assoc_object_map::ODRAssocObjectMap,
    backend::{
        path_es::{ODRBackendObjectPathDecodeError, ODRBackendObjectPathEncodeError},
        BackendExtraCapability, ODRObjectStoreBackend,
    },
    object::{
        invariant::{ODRClassifiedObject, ODRFileObject, ODRNamespaceObject},
        ODRObject,
    },
    object_id::{normal_rootless_uri_path::NormalRootlessUriPath, ODRObjectId},
    object_space::{assoc::rel_type::AssocRelType, ODRAssocMappingError, ODRObjectSpace},
};

pub mod assoc_object_map;
pub mod backend;
pub mod object;
pub mod object_id;
pub mod object_space;
pub mod setup;
pub mod util;

/// Type of object store backend path encoding scheme.
pub type OstBackendObjectPathES<OstSetup> =
    <<OstSetup as ODRObjectStoreSetup>::Backend as ODRObjectStoreBackend>::ObjectPathEncodingScheme;

/// Type of object store backend path encode errors.
pub type OstBackendPathEncodeError<OstSetup> =
    ODRBackendObjectPathEncodeError<OstBackendObjectPathES<OstSetup>>;

/// Type of object store backend path decode errors.
pub type OstBackendPathDecodeError<OstSetup> =
    ODRBackendObjectPathDecodeError<OstBackendObjectPathES<OstSetup>>;

/// An object store manages objects in an object space
/// manifested in it's backend.
#[derive(Debug, Clone)]
pub struct ODRObjectStore<OstSetup: ODRObjectStoreSetup> {
    /// Object space.
    pub space: ODRObjectSpace<OstSetup::ObjectSpaceSetup>,

    /// Backend.
    pub backend: OstSetup::Backend,
}

impl<OstSetup: ODRObjectStoreSetup> ODRObjectStore<OstSetup> {
    /// Get associated odr object for given resource and given
    /// association.
    pub fn assoc_odr_object(
        &self,
        res_uri: &SolidResourceUri,
        assoc_rel_type: AssocRelType,
    ) -> Result<ODRObject<'static, OstSetup>, ODRObjectAssociationError<OstSetup>> {
        // Get id of associated object.
        let assoc_object_id = self.space.encode_assoc_obj_id(res_uri, assoc_rel_type)?;

        Ok(self.odr_object(assoc_object_id.root_relative_path)?)
    }

    /// Get map from association link type to associated odr
    /// object for given resource.
    pub fn assoc_odr_object_map(
        &self,
        res_uri: &SolidResourceUri,
    ) -> Result<ODRAssocObjectMap<OstSetup>, ODRObjectAssociationError<OstSetup>> {
        let mut assoc_links = self.space.assoc_links_for_res(res_uri)?;

        // Compute base object.
        let base_object = ODRClassifiedObject::new(
            self.odr_object(
                assoc_links
                    .remove(&AssocRelType::Base)
                    .expect("Must be some.")
                    .target
                    .root_relative_path,
            )?,
        );

        // Compute aux namespace object.
        let aux_ns_object = ODRNamespaceObject::try_new(
            self.odr_object(
                assoc_links
                    .remove(&AssocRelType::AuxNS)
                    .expect("Must be some.")
                    .target
                    .root_relative_path,
            )?,
        )
        .expect("Must be a namespace object.");

        // Map of sidecar objects.
        let mut sidecars = HashMap::new();

        for assoc_link in assoc_links.into_values() {
            if let AssocRelType::Sidecar(sidecar_rel_type) = assoc_link.rel_type {
                sidecars.insert(
                    sidecar_rel_type,
                    ODRFileObject::try_new(self.odr_object(assoc_link.target.root_relative_path)?)
                        .expect("Must be file object."),
                );
            }
        }

        Ok(ODRAssocObjectMap {
            base_object,
            aux_ns_object,
            sidecars,
        })
    }

    /// Get the handle to odr object at given root relative path.
    pub fn odr_object(
        &self,
        odr_object_path: NormalRootlessUriPath<'static>,
    ) -> Result<ODRObject<'static, OstSetup>, OstBackendPathEncodeError<OstSetup>> {
        ODRObject::try_new(
            ODRObjectId {
                space: self.space.clone(),
                root_relative_path: odr_object_path,
            },
            self.backend.clone(),
        )
    }

    /// Get odr object from backend entry.
    pub fn odr_object_from_backend_entry(
        &self,
        backend_entry: Entry,
    ) -> Result<ODRObject<'static, OstSetup>, OstBackendPathDecodeError<OstSetup>> {
        ODRObject::try_new_from_cached_entry(
            backend_entry,
            self.backend.clone(),
            self.space.clone(),
        )
    }

    /// Get if backend is object content-type metadata capable.
    pub fn is_cty_capable_backend(&self) -> bool {
        self.backend
            .extra_caps()
            .contains(BackendExtraCapability::SupportsNativeContentTypeMetadata)
    }
}

/// An error in computing assoc odr object for a resource.
#[derive(Debug, thiserror::Error)]
pub enum ODRObjectAssociationError<OstSetup: ODRObjectStoreSetup> {
    /// Error in computing associated odr object id for resource.
    #[error("Error in computing associated odr object id for resource.")]
    AssocMappingError(#[from] ODRAssocMappingError),

    /// Error in encoding backend path for mapped odr object.
    #[error("Error in encoding backend path for mapped odr object.")]
    BackendPathEncodeError(#[from] OstBackendPathEncodeError<OstSetup>),
}

//! I define a handle type [`ODRObject`] to odr object.
//!

use flagset::FlagSet;
use once_cell::sync::Lazy;
use opendal::{Entry, Metadata, Metakey};

use super::{
    backend::{path_es::ODRBackendObjectPathEncodingSchemeExt, ODRObjectStoreBackend},
    object_id::ODRObjectId,
    object_space::{assoc::rev_link::AssocRevLink, ODRObjectSpace, ODRRevAssocMappingError},
    ODRObjectStoreSetup, OstBackendObjectPathES, OstBackendPathDecodeError,
    OstBackendPathEncodeError,
};

pub mod invariant;
pub mod predicate;

/// An enum to represent opendal backend entry.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum OpendalBackendEntry {
    /// Simple entry identified by path.
    Simple {
        /// Path of the entry.
        path: String,
    },

    /// A cached entry.
    Cached(Entry),
}

impl OpendalBackendEntry {
    /// Get the entry backend path.
    pub fn path(&self) -> &str {
        match self {
            Self::Simple { path } => path,
            Self::Cached(cached_entry) => cached_entry.path(),
        }
    }
}

/// A type representing handle to an odr object.
#[derive(Debug, Clone)]
pub struct ODRObject<'id, OstSetup>
where
    OstSetup: ODRObjectStoreSetup,
{
    /// odr object id.
    id: ODRObjectId<'id, OstSetup::ObjectSpaceSetup>,

    /// Backend object entry.
    backend_entry: OpendalBackendEntry,

    /// Backend.
    backend: OstSetup::Backend,
}

impl<'id, OstSetup> ODRObject<'id, OstSetup>
where
    OstSetup: ODRObjectStoreSetup,
{
    /// Create new [`ODRObject`] without any checks.
    #[inline]
    pub(crate) fn new_unchecked(
        id: ODRObjectId<'id, OstSetup::ObjectSpaceSetup>,
        backend_entry: OpendalBackendEntry,
        backend: OstSetup::Backend,
    ) -> Self {
        Self {
            id,
            backend_entry,
            backend,
        }
    }

    /// Get id of the odr object.
    #[inline]
    pub fn id(&self) -> &ODRObjectId<'id, OstSetup::ObjectSpaceSetup> {
        &self.id
    }

    /// Get backend object entry.
    #[inline]
    pub fn backend_entry(&self) -> &OpendalBackendEntry {
        &self.backend_entry
    }

    /// Get a identical borrowed odr object.
    #[inline]
    pub fn to_borrowed(&self) -> ODRObject<'_, OstSetup> {
        ODRObject {
            id: self.id.to_borrowed(),
            backend_entry: self.backend_entry.clone(),
            backend: self.backend.clone(),
        }
    }

    /// Try to create a new [`ODRObject`] from given params.
    pub fn try_new(
        object_id: ODRObjectId<'id, OstSetup::ObjectSpaceSetup>,
        backend: OstSetup::Backend,
    ) -> Result<Self, OstBackendPathEncodeError<OstSetup>> {
        let backend_obj_path =
            <OstBackendObjectPathES<OstSetup> as ODRBackendObjectPathEncodingSchemeExt>::encode(
                &object_id.root_relative_path,
            )?;

        Ok(ODRObject::new_unchecked(
            object_id,
            OpendalBackendEntry::Simple {
                path: backend_obj_path,
            },
            backend,
        ))
    }

    /// Try to create a new [`ODRObject`] from given params.
    pub fn try_new_from_cached_entry(
        cached_entry: Entry,
        backend: OstSetup::Backend,
        object_space: ODRObjectSpace<OstSetup::ObjectSpaceSetup>,
    ) -> Result<Self, OstBackendPathDecodeError<OstSetup>> {
        // Get backend object path.
        // Currently opendal marshals rootless empty path "" to rootful "/"
        // as a special case, while all other paths being rootless.
        // This fn fixes back that, until opendal gets fixed.
        let backend_obj_path = if cached_entry.path() == "/" {
            ""
        } else {
            cached_entry.path()
        };

        // Decode odr object path.
        let odr_object_path =
            <OstBackendObjectPathES<OstSetup> as ODRBackendObjectPathEncodingSchemeExt>::decode(
                backend_obj_path,
            )?;

        Ok(ODRObject::new_unchecked(
            ODRObjectId {
                space: object_space,
                root_relative_path: odr_object_path,
            },
            OpendalBackendEntry::Cached(cached_entry),
            backend,
        ))
    }

    /// Get assoc rev link for the odr object.
    pub fn assoc_rev_link(
        &self,
    ) -> Result<AssocRevLink<OstSetup::ObjectSpaceSetup>, ODRRevAssocMappingError> {
        let assoc_rev_link = self
            .id
            .space
            .assoc_rev_link_for_odr_obj(&self.id.root_relative_path)?;

        Ok(assoc_rev_link)
    }
}

/// Meta key for metadata fields required by odr from backend.
pub static ODR_OBJECT_METAKEY: Lazy<FlagSet<Metakey>> = Lazy::new(|| {
    Metakey::ContentLength
        | Metakey::LastModified
        | Metakey::Etag
        | Metakey::ContentType
        | Metakey::Mode
});

impl<'id, OstSetup: ODRObjectStoreSetup> ODRObject<'id, OstSetup> {
    /// Get if given exists or not.
    #[inline]
    pub async fn is_exist(&self) -> Result<bool, opendal::Error> {
        self.backend
            .operator()
            .is_exist(self.backend_entry.path())
            .await
    }

    /// Get backend metadata of the given object.
    #[inline]
    pub async fn metadata(&self) -> Result<Metadata, opendal::Error> {
        match &self.backend_entry {
            OpendalBackendEntry::Simple { path } => self.backend.operator().stat(path).await,
            OpendalBackendEntry::Cached(cached_entry) => {
                // Ok(cached_entry.metadata().clone())
                self.backend
                    .operator()
                    .metadata(cached_entry, *ODR_OBJECT_METAKEY)
                    .await
            }
        }
    }

    /// Delete object.
    ///
    /// Guarantees:
    ///
    /// - If object doesn't exists, return success.
    ///
    #[inline]
    pub async fn delete(&self) -> Result<(), opendal::Error> {
        self.backend
            .operator()
            .delete(self.backend_entry.path())
            .await
    }

    /// Delete object. If it is a namespace object, delete recursively.
    /// This operation is not atomic.
    ///
    /// Guarantees:
    ///
    /// - If object doesn't exists, return success. See <https://github.com/datafuselabs/opendal/issues/1585>
    /// - Backends must support `scan`. If they support `list`,
    /// scan` will be simulated by `CompleteLayer`  with guarantee of child entries coming before dir objects.
    /// See <https://github.com/datafuselabs/opendal/discussions/1584#discussioncomment-5293890>
    #[inline]
    pub async fn delete_recursive(&self) -> Result<(), opendal::Error> {
        self.backend
            .operator()
            .remove_all(self.backend_entry.path())
            .await
    }
}

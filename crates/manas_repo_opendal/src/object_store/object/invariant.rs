//! I define few invariants of [`ODRObject`].
//!

use std::ops::RangeBounds;

use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream::BoxStream, StreamExt, TryFutureExt, TryStreamExt};
use gdp_rs::{binclassified::BinaryClassified, Proven};
use manas_http::{
    header::common::media_type::MediaType,
    representation::impl_::common::data::bytes_stream::BoxBytesStream,
};
use opendal::{EntryMode, Lister, Metakey, Writer};
use tracing::{error, warn};

use super::{
    predicate::{
        is_file_object::IsFileObject, is_namespace_object::IsNamespaceObject,
        ObjectKindBasedClassification,
    },
    ODRObject,
};
use crate::object_store::{
    backend::ODRObjectStoreBackend, ODRObjectStoreSetup, OstBackendPathDecodeError,
};

/// A type alias for an invariant of [`ODRObject`], that ensures inner value to be a namespace object.
pub type ODRNamespaceObject<'id, OstSetup> =
    Proven<ODRObject<'id, OstSetup>, IsNamespaceObject<OstSetup>>;

/// A type alias for an invariant of [`ODRObject`], that ensures inner value to be a file object.
pub type ODRFileObject<'id, OstSetup> = Proven<ODRObject<'id, OstSetup>, IsFileObject<OstSetup>>;

/// A type alias for binary classified invariant of [`ODRObject`].
pub type ODRClassifiedObject<'id, OstSetup> =
    BinaryClassified<ODRObject<'id, OstSetup>, ObjectKindBasedClassification<OstSetup>>;

mod seal {
    use super::{ODRFileObject, ODRNamespaceObject};
    use crate::object_store::ODRObjectStoreSetup;

    /// Trait for module seal.
    pub trait Sealed {}

    impl<'id, OstSetup: ODRObjectStoreSetup> Sealed for ODRFileObject<'id, OstSetup> {}

    impl<'id, OstSetup: ODRObjectStoreSetup> Sealed for ODRNamespaceObject<'id, OstSetup> {}
}

/// An extension trait for [`ODRFileObject`] types.
#[async_trait]
pub trait ODRFileObjectExt<OstSetup: ODRObjectStoreSetup>: seal::Sealed {
    /// Read all the content of the file object.
    // TODO Should have an option to set maximum allowed size.
    async fn read_complete(&self) -> Result<Vec<u8>, opendal::Error>;

    /// Stream the content of the file object in given range.
    async fn stream_range(
        &self,
        range: impl RangeBounds<u64> + Send + 'static,
    ) -> Result<BoxBytesStream, opendal::Error>;

    /// Stream complete content of the file object.
    async fn stream_complete(&self) -> Result<BoxBytesStream, opendal::Error>;

    /// Write complete content of the file object.
    async fn write(
        &self,
        data: impl Into<Bytes> + Send + 'static,
        content_type: &MediaType,
    ) -> Result<(), opendal::Error>;

    /// Write given data in streaming way.
    async fn write_streaming(
        &self,
        data: BoxBytesStream,
        content_type: &MediaType,
    ) -> Result<(), opendal::Error>;
}

#[async_trait]
impl<'id, OstSetup: ODRObjectStoreSetup> ODRFileObjectExt<OstSetup>
    for ODRFileObject<'id, OstSetup>
{
    #[inline]
    async fn read_complete(&self) -> Result<Vec<u8>, opendal::Error> {
        self.backend
            .operator()
            .read(self.backend_entry.path())
            .await
    }

    async fn stream_range(
        &self,
        range: impl RangeBounds<u64> + Send + 'static,
    ) -> Result<BoxBytesStream, opendal::Error> {
        Ok(Box::pin(
            self.backend
                .operator()
                .reader_with(self.backend_entry.path())
                .range(range)
                .await?
                .err_into::<anyhow::Error>(),
        ) as BoxStream<_>)
    }

    async fn stream_complete(&self) -> Result<BoxBytesStream, opendal::Error> {
        Ok(Box::pin(
            self.backend
                .operator()
                .reader(self.backend_entry.path())
                .await?
                .err_into::<anyhow::Error>(),
        ) as BoxStream<_>)
    }

    #[inline]
    async fn write(
        &self,
        data: impl Into<Bytes> + Send + 'static,
        content_type: &MediaType,
    ) -> Result<(), opendal::Error> {
        self.backend
            .operator()
            .write_with(self.backend_entry.path(), data.into())
            .content_type(content_type.essence_str())
            .await
    }

    /// Write given data in streaming way.
    async fn write_streaming(
        &self,
        mut data: BoxBytesStream,
        content_type: &MediaType,
    ) -> Result<(), opendal::Error> {
        // Get writer.
        let mut writer = self
            .backend
            .operator()
            .writer_with(self.backend_entry.path())
            .content_type(content_type.essence_str())
            .await?;

        let mut written_once = false;

        while let Some(r) = data.next().await {
            abort_on_error(
                match r {
                    Ok(bs) => writer.write(bs).await,
                    // Source error.
                    Err(_) => Err(opendal::Error::new(
                        opendal::ErrorKind::Unexpected,
                        "Source error",
                    )),
                },
                &mut writer,
            )
            .await?;
            written_once = true;
        }

        if !written_once {
            // Empty content.
            abort_on_error(writer.write(Bytes::default()).await, &mut writer).await?
        }

        writer
            .close()
            .inspect_err(|e| error!("Error in closing the writer. Error:\n {}", e))
            .await?;

        Ok(())
    }
}

async fn abort_on_error(
    write_result: Result<(), opendal::Error>,
    writer: &mut Writer,
) -> Result<(), opendal::Error> {
    if let Err(e) = write_result {
        error!("Error in writing bytes to the writer. Error:\n {}", e);

        // Try to abort.
        let _ = writer
            .abort()
            .inspect_err(|e| warn!("Error in aborting write operation. Error:\n {}", e))
            .await;

        return Err(e);
    }
    Ok(())
}

/// A type alias for try stream that decodes odr objects from backend objects.
pub type DecodedODRObjectStream<OstSetup> =
    BoxStream<'static, Result<ODRObject<'static, OstSetup>, ODRObjectYieldError<OstSetup>>>;

/// An extension trait for [`ODRNamespaceObject`] types.
#[async_trait]
pub trait ODRNamespaceObjectExt<OstSetup: ODRObjectStoreSetup>: seal::Sealed {
    /// Create namespace object.
    async fn create(&self) -> Result<(), opendal::Error>;

    /// List odr objects in this namespace.
    async fn list(&self) -> Result<DecodedODRObjectStream<OstSetup>, opendal::Error>;
}

#[async_trait]
impl<'id, OstSetup: ODRObjectStoreSetup> ODRNamespaceObjectExt<OstSetup>
    for ODRNamespaceObject<'id, OstSetup>
{
    #[inline]
    async fn create(&self) -> Result<(), opendal::Error> {
        self.backend
            .operator()
            .create_dir(self.backend_entry.path())
            .await
    }

    async fn list(&self) -> Result<DecodedODRObjectStream<OstSetup>, opendal::Error> {
        let backend_listing: Lister = self
            .backend
            .operator()
            .list(self.backend_entry.path())
            // .metakey(*ODR_OBJECT_METAKEY)
            .inspect_err(|_| error!("Error in getting backend entries."))
            .await?;

        let backend = self.backend.clone();
        let object_space = self.id.space.clone();

        Ok(Box::pin(async_stream::try_stream! {
            for await entry in backend_listing {
                let entry = entry?;
                // Get mode from metadata.
                // let mode = entry.metadata().mode();
                let mode = backend.operator().metadata(&entry, Metakey::Mode).await?.mode();

                // If unknown mode, yield error.
                if mode ==EntryMode::Unknown {
                    Err(ODRObjectYieldError::InvalidBackendObjectMode)?;
                }

                // Decode odr object.
                let odr_object: ODRObject<'static, OstSetup> = ODRObject::try_new_from_cached_entry(
                    entry,
                    backend.clone(),
                    object_space.clone()
                ).map_err(|e| ODRObjectYieldError::BackendPathDecodeError(e))?;

                yield odr_object;
            }
        }))
    }
}

/// An error type for representing error in yielding an odr object in namespace listing.
#[derive(Debug, thiserror::Error)]
pub enum ODRObjectYieldError<OstSetup: ODRObjectStoreSetup> {
    /// Invalid backend object kind.
    #[error("Invalid backend object kind.")]
    InvalidBackendObjectMode,

    /// Error in decoding backend object path.
    #[error("Error in decoding backend object path.")]
    BackendPathDecodeError(OstBackendPathDecodeError<OstSetup>),

    /// Unknown io error.
    #[error("Unknown io error.")]
    UnknownIoError(#[from] opendal::Error),
}

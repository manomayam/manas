use std::io;

use async_compat::CompatExt;
use futures::{AsyncWrite, Stream};
use sophia_api::{
    dataset::Dataset,
    quad::Quad,
    serializer::QuadSerializer,
    source::{QuadSource, StreamError, StreamResult},
};
use tokio::{io::BufWriter, task::spawn_blocking};
use tokio_util::io::SyncIoBridge;
use tracing::error;

use super::{factory::DynSynQuadSerializerFactory, sync::DynSynQuadSerializer, BridgedWrite};
use crate::{
    syntax::invariant::quads_serializable::QuadsSerializableSyntax,
    util::stream::BlockingStreamIterator,
};

/// Async quad serializer.
pub struct DynSynAsyncQuadSerializer<W>(pub(crate) DynSynQuadSerializer<BridgedWrite<W>>)
where
    W: AsyncWrite + Unpin;

impl<W> DynSynAsyncQuadSerializer<W>
where
    W: AsyncWrite + Send + 'static + Unpin,
{
    pub(crate) async fn _serialize<S>(
        mut self,
        serializable: S,
    ) -> StreamResult<Self, S::SourceErr, io::Error>
    where
        S: QuadSerializable + Send + 'static,
        S::SourceErr: Send + 'static,
    {
        // Spawn serialization task.
        spawn_blocking(move || {
            // Serialize, and return
            match serializable.serialize_with(&mut self.0) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| {
            error!(
                "Error in running serialization task to completion. Error:\n {}",
                e
            );
            StreamError::SinkError(io::Error::new(io::ErrorKind::Other, e))
        })?
    }

    /// Serialize given quad source.
    #[inline]
    pub async fn serialize_quads<QS>(
        self,
        quad_source: QS,
    ) -> StreamResult<Self, QS::Error, io::Error>
    where
        QS: QuadSource + Send + 'static,
        QS::Error: Send + 'static,
    {
        self._serialize(QuadSerializableSource(quad_source)).await
    }

    /// Serialize given dataset.
    #[inline]
    pub async fn serialize_dataset<D>(self, dataset: D) -> StreamResult<Self, D::Error, io::Error>
    where
        D: Dataset + Send + 'static,
        D::Error: Send + 'static,
    {
        self._serialize(QuadSerializableDataset(dataset)).await
    }

    /// Serialize given quads stream.
    pub async fn serialize<Q, E, QS>(self, quads: QS) -> StreamResult<Self, E, io::Error>
    where
        Q: Quad + Send + 'static,
        E: Send + 'static + std::error::Error,
        QS: Stream<Item = Result<Q, E>> + Send + 'static + Unpin,
    {
        let quads_blocking_iterator =
            BlockingStreamIterator::new(quads).map_err(|e| StreamError::SinkError(e))?;

        self.serialize_quads(quads_blocking_iterator).await
    }
}

pub(crate) trait QuadSerializable {
    /// Type of error, this quad serializable raises.
    type SourceErr: 'static + std::error::Error;

    /// Serialize the serializable with given serializer.
    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynQuadSerializer<W>,
    ) -> StreamResult<&mut DynSynQuadSerializer<W>, Self::SourceErr, io::Error>;
}

pub(crate) struct QuadSerializableSource<QS: QuadSource>(QS);

impl<QS: QuadSource> QuadSerializable for QuadSerializableSource<QS> {
    type SourceErr = QS::Error;

    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynQuadSerializer<W>,
    ) -> StreamResult<&mut DynSynQuadSerializer<W>, Self::SourceErr, io::Error> {
        s.serialize_quads(self.0)
    }
}

pub(crate) struct QuadSerializableDataset<D: Dataset>(D);

impl<D: Dataset> QuadSerializable for QuadSerializableDataset<D> {
    type SourceErr = D::Error;

    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynQuadSerializer<W>,
    ) -> StreamResult<&mut DynSynQuadSerializer<W>, Self::SourceErr, io::Error> {
        s.serialize_quads(self.0.quads())
    }
}

impl DynSynQuadSerializerFactory {
    /// Create new [`DynSynAsyncQuadSerializer`] instance, for given `syntax_`, `write`,
    pub fn new_async_serializer<W>(
        &self,
        syntax_: QuadsSerializableSyntax,
        write: W,
    ) -> DynSynAsyncQuadSerializer<W>
    where
        W: AsyncWrite + Unpin,
    {
        DynSynAsyncQuadSerializer(
            self.new_serializer(syntax_, SyncIoBridge::new(BufWriter::new(write.compat()))),
        )
    }
}

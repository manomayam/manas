use std::io;

use async_compat::CompatExt;
use futures::{AsyncWrite, Stream};
use sophia_api::{
    graph::Graph,
    prelude::Dataset,
    serializer::TripleSerializer,
    source::{StreamError, StreamResult, TripleSource},
    term::SimpleTerm,
    triple::Triple,
};
use tokio::{io::BufWriter, task::spawn_blocking};
use tokio_util::io::SyncIoBridge;
use tracing::error;

use super::{factory::DynSynTripleSerializerFactory, sync::DynSynTripleSerializer};
use crate::{
    serializer::quads::BridgedWrite,
    syntax::invariant::triples_serializable::TriplesSerializableSyntax,
    util::stream::BlockingStreamIterator,
};

/// Async triple serializer.
pub struct DynSynAsyncTripleSerializer<W>(pub(crate) DynSynTripleSerializer<BridgedWrite<W>>)
where
    W: AsyncWrite + Unpin;

impl<W> DynSynAsyncTripleSerializer<W>
where
    W: AsyncWrite + Send + 'static + Unpin,
{
    pub(crate) async fn _serialize<S>(
        mut self,
        serializable: S,
    ) -> StreamResult<Self, S::SourceErr, io::Error>
    where
        S: TripleSerializable + Send + 'static,
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

    /// Serialize given triple source.
    #[inline]
    pub async fn serialize_triples<TS>(
        self,
        triple_source: TS,
    ) -> StreamResult<Self, TS::Error, io::Error>
    where
        TS: TripleSource + Send + 'static,
        TS::Error: Send + 'static,
    {
        self._serialize(TripleSerializableSource(triple_source))
            .await
    }

    /// Serialize given graph.
    #[inline]
    pub async fn serialize_graph<G>(self, graph: G) -> StreamResult<Self, G::Error, io::Error>
    where
        G: Graph + Send + 'static,
        G::Error: Send + 'static,
    {
        self._serialize(TripleSerializableGraph(graph)).await
    }

    #[inline]
    pub(crate) async fn wrapping_serialize_dataset<D>(
        self,
        dataset: D,
    ) -> StreamResult<Self, D::Error, io::Error>
    where
        D: Dataset + Send + 'static,
        D::Error: Send + 'static,
    {
        self._serialize(TripleSerializableWrappedDataset(dataset))
            .await
    }

    /// Serialize given triples stream.
    pub async fn serialize<Q, E, TS>(self, triples: TS) -> StreamResult<Self, E, io::Error>
    where
        Q: Triple + Send + 'static,
        E: Send + 'static + std::error::Error,
        TS: Stream<Item = Result<Q, E>> + Send + 'static + Unpin,
    {
        let triples_blocking_iterator =
            BlockingStreamIterator::new(triples).map_err(|e| StreamError::SinkError(e))?;

        self.serialize_triples(triples_blocking_iterator).await
    }
}

pub(crate) trait TripleSerializable {
    /// Type of error, this triple serializable raises.
    type SourceErr: 'static + std::error::Error;

    /// Serialize the serializable with given serializer.
    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynTripleSerializer<W>,
    ) -> StreamResult<&mut DynSynTripleSerializer<W>, Self::SourceErr, io::Error>;
}

pub(crate) struct TripleSerializableSource<TS: TripleSource>(TS);

impl<TS: TripleSource> TripleSerializable for TripleSerializableSource<TS> {
    type SourceErr = TS::Error;

    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynTripleSerializer<W>,
    ) -> StreamResult<&mut DynSynTripleSerializer<W>, Self::SourceErr, io::Error> {
        s.serialize_triples(self.0)
    }
}

pub(crate) struct TripleSerializableGraph<G: Graph>(G);

impl<G: Graph> TripleSerializable for TripleSerializableGraph<G> {
    type SourceErr = G::Error;

    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynTripleSerializer<W>,
    ) -> StreamResult<&mut DynSynTripleSerializer<W>, Self::SourceErr, io::Error> {
        s.serialize_triples(self.0.triples())
    }
}

pub(crate) struct TripleSerializableWrappedDataset<D: Dataset>(D);

impl<D: Dataset> TripleSerializable for TripleSerializableWrappedDataset<D> {
    type SourceErr = D::Error;

    #[allow(unused_qualifications)]
    fn serialize_with<W: io::Write>(
        self,
        s: &mut DynSynTripleSerializer<W>,
    ) -> StreamResult<&mut DynSynTripleSerializer<W>, Self::SourceErr, io::Error> {
        s.serialize_graph(
            &self
                .0
                .partial_union_graph([Option::<&'static SimpleTerm>::None]),
        )
    }
}
impl DynSynTripleSerializerFactory {
    /// Create new [`DynSynAsyncTripleSerializer`] instance, for given `syntax_`, `write`,
    pub fn new_async_serializer<W>(
        &self,
        syntax_: TriplesSerializableSyntax,
        write: W,
    ) -> DynSynAsyncTripleSerializer<W>
    where
        W: AsyncWrite + Unpin,
    {
        DynSynAsyncTripleSerializer(
            self.new_serializer(syntax_, SyncIoBridge::new(BufWriter::new(write.compat()))),
        )
    }
}

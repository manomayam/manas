//! I provide a basic implementation of [`Representation`].
//!

use std::{
    io::{self, BufReader},
    sync::Arc,
};

use async_convert::async_trait;
use ecow::eco_vec;
use either::Either;
use futures::TryFutureExt;
use headers::ContentLength;
use http_uri::HttpUri;
use rdf_dynsyn::{
    correspondence::SYNTAX_TO_MEDIA_TYPE_CORRESPONDENCE,
    parser::DynSynParserFactorySet,
    serializer::DynSynSerializerFactorySet,
    syntax::invariant::{parsable::DynSynParsableSyntax, serializable::DynSynSerializableSyntax},
};
use sophia_api::prelude::{Dataset, MutableDataset};
use tokio::task::spawn_blocking;
use tower::BoxError;
use tracing::{error, info};

use super::{
    binary::BinaryRepresentation,
    common::data::{
        bytes::BytesData,
        bytes_inmem::BytesInmem,
        bytes_stream::BytesStream,
        quads::QuadsData,
        quads_inmem::{EcoQuadsInmem, QuadsInmem},
        quads_stream::QuadsStream,
    },
};
use crate::representation::{
    metadata::{KCompleteContentLength, KContentType, RepresentationMetadata},
    Representation,
};

/// A basic implementation of [`Representation`].
#[derive(Clone, Default)]
pub struct BasicRepresentation<D> {
    /// Data.
    pub data: D,

    /// Metadata.
    pub metadata: RepresentationMetadata,

    /// Base uri.
    pub base_uri: Option<HttpUri>,
}

impl<D> std::fmt::Debug for BasicRepresentation<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicRepresentation")
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl<D> Representation for BasicRepresentation<D> {
    type Data = D;

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn metadata(&self) -> &RepresentationMetadata {
        &self.metadata
    }

    fn into_parts(self) -> (Self::Data, RepresentationMetadata) {
        (self.data, self.metadata)
    }
}

impl From<BasicRepresentation<BytesStream>> for BasicRepresentation<BytesData> {
    fn from(rep: BasicRepresentation<BytesStream>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

impl From<BasicRepresentation<BytesInmem>> for BasicRepresentation<BytesStream> {
    fn from(rep: BasicRepresentation<BytesInmem>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

impl From<BasicRepresentation<BytesData>> for BasicRepresentation<BytesStream> {
    fn from(rep: BasicRepresentation<BytesData>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

impl From<BasicRepresentation<BytesInmem>> for BasicRepresentation<BytesData> {
    fn from(rep: BasicRepresentation<BytesInmem>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

impl From<BasicRepresentation<QuadsStream>> for BasicRepresentation<QuadsData> {
    fn from(rep: BasicRepresentation<QuadsStream>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

impl From<BasicRepresentation<EcoQuadsInmem>> for BasicRepresentation<QuadsData> {
    fn from(rep: BasicRepresentation<EcoQuadsInmem>) -> Self {
        Self {
            data: rep.data.into(),
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<BasicRepresentation<BytesData>> for BasicRepresentation<BytesInmem> {
    type Error = anyhow::Error;

    async fn try_from(rep: BasicRepresentation<BytesData>) -> Result<Self, Self::Error> {
        match rep.into_either() {
            Either::Left(v) => async_convert::TryFrom::try_from(v).await,
            Either::Right(v) => Ok(v),
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<BasicRepresentation<BytesStream>> for BasicRepresentation<BytesInmem> {
    type Error = anyhow::Error;

    async fn try_from(rep: BasicRepresentation<BytesStream>) -> Result<Self, Self::Error> {
        Ok(BasicRepresentation {
            data: async_convert::TryFrom::try_from(rep.data)
                .inspect_err(|e| error!("Error in collecting stream data. Error: {e}"))
                .await?,
            metadata: rep.metadata,
            base_uri: rep.base_uri,
        })
    }
}

impl BasicRepresentation<BytesData> {
    /// Convert into binary representation.
    #[inline]
    pub fn into_binary(self) -> BinaryRepresentation {
        self.into()
    }

    /// Convert into streaming variant.
    #[inline]
    pub fn into_streaming(self) -> BasicRepresentation<BytesStream> {
        self.into()
    }

    /// Convert into stream size capped rep.
    /// It caps only if inner variant is stream.
    #[inline]
    pub fn into_stream_size_capped(mut self, size_limit: u64) -> Self {
        self.data = match self.data {
            BytesData::Stream(d) => BytesData::Stream(d.into_size_capped(size_limit)),
            BytesData::Inmem(d) => BytesData::Inmem(d),
        };
        self
    }

    /// Convert into either type.
    pub fn into_either(
        self,
    ) -> Either<BasicRepresentation<BytesStream>, BasicRepresentation<BytesInmem>> {
        match self.data {
            BytesData::Stream(sd) => Either::Left(BasicRepresentation {
                data: sd,
                metadata: self.metadata,
                base_uri: self.base_uri,
            }),
            BytesData::Inmem(imd) => Either::Right(BasicRepresentation {
                data: imd,
                metadata: self.metadata,
                base_uri: self.base_uri,
            }),
        }
    }
}

impl BasicRepresentation<BytesStream> {
    /// Convert into binary streaming representation.
    #[inline]
    pub fn into_binary_streaming(self) -> BinaryRepresentation<BytesStream> {
        self.into()
    }

    /// Convert into binary representation.
    #[inline]
    pub fn into_binary(self) -> BinaryRepresentation {
        self.into_binary_streaming().into()
    }

    /// Convert into stream size capped rep.
    #[inline]
    pub fn into_size_capped(mut self, size_limit: u64) -> Self {
        self.data = self.data.into_size_capped(size_limit);
        self
    }
}

impl BasicRepresentation<BytesInmem> {
    /// Convert into binary inmem representation.
    #[inline]
    pub fn into_binary_inmem(self) -> BinaryRepresentation<BytesInmem> {
        self.into()
    }

    /// Convert into binary representation.
    #[inline]
    pub fn into_binary(self) -> BinaryRepresentation {
        self.into_binary_inmem().into()
    }

    /// Try to parse quads into a dataset.
    /// If content-type of rep is not quadable, returns [`None`].
    pub async fn try_parse_quads<D>(
        &self,
        parser_factories: Arc<DynSynParserFactorySet>,
    ) -> Option<Result<QuadsInmem<D>, BoxError>>
    where
        D: MutableDataset + Default + Send + 'static,
        D::MutationError: Send + Sync + 'static,
    {
        // Resolve parsable syntax.
        let parsable_syntax = self
            .metadata()
            .rdf_syntax::<DynSynParsableSyntax>()
            .or_else(|| {
                info!("Representation content-type is not quadable.");
                None
            })?
            .value;

        let data = self.data().clone();
        let base_uri = self.base_uri.clone();

        spawn_blocking(move || {
            let mut ds = D::default();
            parser_factories
                .parse_collect_quads(
                    BufReader::new(data.as_read()),
                    base_uri.as_ref().map(From::from),
                    parsable_syntax,
                    &mut ds,
                )
                .map(|_| QuadsInmem::new(ds))
                .map_err(Into::into)
        })
        .await
        .map_or_else(
            |e| {
                error!("Error in executing parse task. Error: {e}");
                Some(Err(e.into()))
            },
            Some,
        )
    }

    /// Try to create representation from Wrapping serialize the quads.
    /// if syntax is graph serializing, then it only serializes
    /// default graph.
    pub async fn try_from_wrap_serializing_quads<D: Dataset + Send + 'static>(
        quads: QuadsInmem<D>,
        serializer_factories: Arc<DynSynSerializerFactorySet>,
        syntax: DynSynSerializableSyntax,
    ) -> Result<Self, io::Error> {
        let dataset = quads.into_inner();
        let serialized_bytes = spawn_blocking(move || {
            let mut buf = Vec::new();
            serializer_factories
                .wrapping_serialize_quads(dataset.quads(), &mut buf, syntax)
                .map(|_| buf)
                .map_err(|e| e.unwrap_sink_error())
        })
        .await
        .map_err(|e| {
            error!("Error in spawning serialization task: {e}");
            io::Error::new(io::ErrorKind::Other, e)
        })??;

        let content_type = SYNTAX_TO_MEDIA_TYPE_CORRESPONDENCE[&syntax.into_subject()]
            .value
            .clone()
            .try_into()
            .expect("Must be valid media type.");

        Ok(Self {
            metadata: RepresentationMetadata::new()
                .with::<KContentType>(content_type)
                .with::<KCompleteContentLength>(ContentLength(serialized_bytes.len() as u64)),
            data: BytesInmem::from(eco_vec![serialized_bytes.into()]),
            base_uri: None,
        })
    }
}

//! I provide an implementation of [`Representation`] with
//! binary data.
//!

use std::{
    io::{self},
    sync::Arc,
};

use async_convert::async_trait;
use async_once_cell::OnceCell;
use either::Either;
use http_uri::HttpUri;
use rdf_dynsyn::{
    parser::DynSynParserFactorySet, serializer::DynSynSerializerFactorySet,
    syntax::invariant::serializable::DynSynSerializableSyntax,
};
use rdf_utils::model::{dataset::EcoDataset, quad::ArcQuad};
use sophia_api::prelude::{Dataset, MutableDataset};
use tower::BoxError;

use super::{
    basic::BasicRepresentation,
    common::data::{
        bytes::BytesData,
        bytes_inmem::BytesInmem,
        bytes_stream::BytesStream,
        quads_inmem::{EcoQuadsInmem, QuadsInmem},
    },
};
use crate::representation::{metadata::RepresentationMetadata, Representation};

/// An implementation of [`Representation`] with
/// binary data.
pub struct BinaryRepresentation<BD = BytesData> {
    /// Inner
    inner: BasicRepresentation<BD>,

    /// Rdf parse result, if representation is that of an rdf doc.
    mb_rdf_parse_result: OnceCell<Option<Result<EcoQuadsInmem, BoxError>>>,
}

impl<BD: Clone> Clone for BinaryRepresentation<BD> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mb_rdf_parse_result: Default::default(),
        }
    }
}

impl<BD> std::fmt::Debug for BinaryRepresentation<BD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinaryRepresentation")
            .field("inner", &self.inner)
            .field("mb_rdf_parse_result", &self.mb_rdf_parse_result)
            .finish()
    }
}

impl<BD> Representation for BinaryRepresentation<BD> {
    type Data = BD;

    #[inline]
    fn data(&self) -> &Self::Data {
        self.inner.data()
    }

    #[inline]
    fn metadata(&self) -> &RepresentationMetadata {
        self.inner.metadata()
    }

    #[inline]
    fn into_parts(self) -> (Self::Data, RepresentationMetadata) {
        self.inner.into_parts()
    }
}

impl<BD> BinaryRepresentation<BD> {
    /// Get the base uri of representation.
    #[inline]
    pub fn base_uri(&self) -> &Option<HttpUri> {
        &self.inner.base_uri
    }

    /// Convert into [`BasicRepresentation`].
    #[inline]
    pub fn into_basic(self) -> BasicRepresentation<BD> {
        self.inner
    }
}

#[async_trait]
impl async_convert::TryFrom<BinaryRepresentation<BytesStream>>
    for BinaryRepresentation<BytesInmem>
{
    type Error = BoxError;

    async fn try_from(rep: BinaryRepresentation<BytesStream>) -> Result<Self, Self::Error> {
        Ok(BinaryRepresentation {
            inner: async_convert::TryFrom::try_from(rep.inner).await?,
            mb_rdf_parse_result: rep.mb_rdf_parse_result,
        })
    }
}

#[async_trait]
impl async_convert::TryFrom<BinaryRepresentation> for BinaryRepresentation<BytesInmem> {
    type Error = BoxError;

    async fn try_from(rep: BinaryRepresentation) -> Result<Self, Self::Error> {
        match rep.into_either() {
            Either::Left(rep) => async_convert::TryFrom::try_from(rep).await,
            Either::Right(rep) => Ok(rep),
        }
    }
}

impl BinaryRepresentation {
    /// Convert into streaming variant.
    pub fn into_streaming(self) -> BinaryRepresentation<BytesStream> {
        self.into()
    }

    /// Convert into stream size capped rep.
    /// It caps only if inner variant is stream.
    #[inline]
    pub fn into_stream_size_capped(mut self, size_limit: u64) -> Self {
        self.inner = self.inner.into_stream_size_capped(size_limit);
        self
    }

    /// Convert into either type.
    pub fn into_either(
        self,
    ) -> Either<BinaryRepresentation<BytesStream>, BinaryRepresentation<BytesInmem>> {
        match self.inner.into_either() {
            Either::Left(inner) => Either::Left(BinaryRepresentation {
                inner,
                mb_rdf_parse_result: self.mb_rdf_parse_result,
            }),
            Either::Right(inner) => Either::Right(BinaryRepresentation {
                inner,
                mb_rdf_parse_result: self.mb_rdf_parse_result,
            }),
        }
    }
}

impl BinaryRepresentation<BytesInmem> {
    /// Try to parse quads into a dataset.
    /// If content-type of rep is not quadable, returns [`None`].
    #[inline]
    pub async fn try_parse_quads<D>(
        &self,
        parser_factories: Arc<DynSynParserFactorySet>,
    ) -> Option<Result<QuadsInmem<D>, BoxError>>
    where
        D: MutableDataset + Default + Send + 'static,
        D::MutationError: Send + Sync + 'static,
    {
        self.inner.try_parse_quads(parser_factories).await
    }

    /// Try to parse quads from the binary representation.
    /// Returns [`None`] if content-type is not quads parsable.
    pub async fn try_parse_quads_caching(
        &self,
        parser_factories: Arc<DynSynParserFactorySet>,
    ) -> Option<&Result<EcoQuadsInmem, BoxError>> {
        self.mb_rdf_parse_result
            .get_or_init(async move {
                self.try_parse_quads::<EcoDataset<ArcQuad>>(parser_factories)
                    .await
            })
            .await
            .as_ref()
    }

    /// Try to create representation from Wrapping serialize the quads.
    /// if syntax is graph serializing, then it only serializes
    /// default graph.
    pub async fn try_from_wrap_serializing_quads<D: Dataset + Send + 'static>(
        quads: QuadsInmem<D>,
        serializer_factories: Arc<DynSynSerializerFactorySet>,
        syntax: DynSynSerializableSyntax,
    ) -> Result<Self, io::Error> {
        Ok(BasicRepresentation::try_from_wrap_serializing_quads(
            quads,
            serializer_factories,
            syntax,
        )
        .await?
        .into())
    }
}

impl From<BinaryRepresentation<BytesStream>> for BinaryRepresentation {
    fn from(rep: BinaryRepresentation<BytesStream>) -> Self {
        Self {
            inner: rep.inner.into(),
            mb_rdf_parse_result: rep.mb_rdf_parse_result,
        }
    }
}

impl From<BinaryRepresentation> for BinaryRepresentation<BytesStream> {
    fn from(rep: BinaryRepresentation) -> Self {
        Self {
            inner: rep.inner.into(),
            mb_rdf_parse_result: rep.mb_rdf_parse_result,
        }
    }
}

impl From<BinaryRepresentation<BytesInmem>> for BinaryRepresentation {
    fn from(rep: BinaryRepresentation<BytesInmem>) -> Self {
        Self {
            inner: rep.inner.into(),
            mb_rdf_parse_result: rep.mb_rdf_parse_result,
        }
    }
}

impl From<BasicRepresentation<BytesData>> for BinaryRepresentation {
    fn from(rep: BasicRepresentation<BytesData>) -> Self {
        Self {
            inner: rep,
            mb_rdf_parse_result: Default::default(),
        }
    }
}

impl From<BasicRepresentation<BytesStream>> for BinaryRepresentation {
    fn from(rep: BasicRepresentation<BytesStream>) -> Self {
        BasicRepresentation::<BytesData>::from(rep).into()
    }
}

impl From<BasicRepresentation<BytesInmem>> for BinaryRepresentation {
    fn from(rep: BasicRepresentation<BytesInmem>) -> Self {
        BasicRepresentation::<BytesData>::from(rep).into()
    }
}

impl From<BasicRepresentation<BytesStream>> for BinaryRepresentation<BytesStream> {
    fn from(rep: BasicRepresentation<BytesStream>) -> Self {
        Self {
            inner: rep,
            mb_rdf_parse_result: Default::default(),
        }
    }
}

impl From<BasicRepresentation<BytesInmem>> for BinaryRepresentation<BytesInmem> {
    fn from(rep: BasicRepresentation<BytesInmem>) -> Self {
        Self {
            inner: rep,
            mb_rdf_parse_result: Default::default(),
        }
    }
}

impl<D> From<BinaryRepresentation<D>> for BasicRepresentation<D> {
    fn from(rep: BinaryRepresentation<D>) -> Self {
        rep.inner
    }
}

//! I define type to represent streaming bytes data.
//!

use async_convert::async_trait;
use bytes::Bytes;
use capped_stream::{BytesWeigher, CappedStream};
use ecow::EcoVec;
use futures::{stream::BoxStream, TryStreamExt};
use hyper::body::{HttpBody, SizeHint};

use super::bytes_inmem::BytesInmem;

/// Type alias for a boxed fallible bytes stream.
pub type BoxBytesStream = BoxStream<'static, Result<Bytes, anyhow::Error>>;

/// Bytes stream data.
pub struct BytesStream {
    /// Data stream.
    pub stream: BoxBytesStream,

    /// Size hint.
    pub size_hint: SizeHint,
}

impl std::fmt::Debug for BytesStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BytesStream")
            .field("size_hint", &self.size_hint)
            .finish()
    }
}

impl From<hyper::Body> for BytesStream {
    fn from(value: hyper::Body) -> Self {
        Self::from_hyper_body(value, None)
    }
}

impl From<BytesInmem> for BytesStream {
    fn from(value: BytesInmem) -> Self {
        Self {
            size_hint: SizeHint::with_exact(value.size),
            stream: Box::pin(futures::stream::iter(value.bytes.into_iter().map(Ok))),
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<BytesStream> for BytesInmem {
    type Error = anyhow::Error;

    async fn try_from(data: BytesStream) -> Result<Self, Self::Error> {
        Ok(BytesInmem::from(
            data.stream.try_collect::<EcoVec<_>>().await?,
        ))
    }
}

impl BytesStream {
    /// Convert into size capped stream.
    pub fn into_size_capped(mut self, size_limit: u64) -> Self {
        self.stream = Box::pin(CappedStream::new(
            self.stream,
            BytesWeigher::<Bytes>::default(),
            size_limit,
        ));
        self
    }

    /// Try to create [`BytesStream`] from hyper body.
    pub fn from_hyper_body(body: hyper::Body, size_hint: Option<SizeHint>) -> Self {
        Self {
            size_hint: size_hint.unwrap_or(body.size_hint()),
            stream: Box::pin(TryStreamExt::map_err(body, anyhow::Error::new)),
        }
    }
}

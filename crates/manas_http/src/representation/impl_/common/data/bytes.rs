//! I define type to represent bytes data that can be streaming/in-memory..
//!

use async_convert::async_trait;
use http_body::SizeHint;

use crate::{body::Body, BoxError};

use super::{bytes_inmem::BytesInmem, bytes_stream::BytesStream};

/// A struct to represent bytes data.
#[derive(Debug)]
pub enum BytesData {
    /// Stream data.
    Stream(BytesStream),

    /// Inmem data.
    Inmem(BytesInmem),
}

impl Default for BytesData {
    #[inline]
    fn default() -> Self {
        Self::Inmem(Default::default())
    }
}

impl From<Body> for BytesData {
    #[inline]
    fn from(value: Body) -> Self {
        Self::Stream(value.into())
    }
}

impl From<BytesStream> for BytesData {
    #[inline]
    fn from(value: BytesStream) -> Self {
        Self::Stream(value)
    }
}

impl From<BytesInmem> for BytesData {
    #[inline]
    fn from(value: BytesInmem) -> Self {
        Self::Inmem(value)
    }
}

impl From<BytesData> for BytesStream {
    fn from(val: BytesData) -> Self {
        match val {
            BytesData::Stream(d) => d,
            BytesData::Inmem(d) => d.into(),
        }
    }
}

impl BytesData {
    /// Get the size hint.
    pub fn size_hint(&self) -> SizeHint {
        match self {
            Self::Stream(d) => d.size_hint.clone(),
            Self::Inmem(d) => SizeHint::with_exact(d.size()),
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<BytesData> for BytesInmem {
    type Error = BoxError;

    async fn try_from(data: BytesData) -> Result<Self, Self::Error> {
        match data {
            BytesData::Stream(d) => async_convert::TryFrom::try_from(d).await,
            BytesData::Inmem(d) => Ok(d),
        }
    }
}

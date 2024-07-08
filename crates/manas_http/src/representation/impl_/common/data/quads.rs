//! I define type to represent quads data that can be streaming/in-memory..
//!

use crate::BoxError;
use async_convert::async_trait;
use http_body::SizeHint;

use super::{quads_inmem::EcoQuadsInmem, quads_stream::QuadsStream};

/// A struct to represent quads data.
#[derive(Debug)]
pub enum QuadsData {
    /// Stream data.
    Stream(QuadsStream),

    /// Inmem data.
    Inmem(EcoQuadsInmem),
}

impl From<QuadsStream> for QuadsData {
    #[inline]
    fn from(value: QuadsStream) -> Self {
        Self::Stream(value)
    }
}

impl From<EcoQuadsInmem> for QuadsData {
    #[inline]
    fn from(value: EcoQuadsInmem) -> Self {
        Self::Inmem(value)
    }
}

impl From<QuadsData> for QuadsStream {
    fn from(val: QuadsData) -> Self {
        match val {
            QuadsData::Stream(d) => d,
            QuadsData::Inmem(d) => d.into(),
        }
    }
}

impl QuadsData {
    /// Get the size hint.
    pub fn size_hint(&self) -> SizeHint {
        match self {
            Self::Stream(d) => d.size_hint.clone(),
            Self::Inmem(d) => SizeHint::with_exact(d.len() as u64),
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<QuadsData> for EcoQuadsInmem {
    type Error = BoxError;

    async fn try_from(data: QuadsData) -> Result<Self, Self::Error> {
        match data {
            QuadsData::Stream(d) => async_convert::TryFrom::try_from(d).await,
            QuadsData::Inmem(d) => Ok(d),
        }
    }
}

//! I define type to represent streaming quads data.
//!

use async_convert::async_trait;
use ecow::EcoVec;
use futures::{stream::BoxStream, TryStreamExt};
use http_body::SizeHint;
use rdf_utils::model::{dataset::CompatDataset, quad::ArcQuad};

use crate::BoxError;

use super::quads_inmem::{EcoQuadsInmem, QuadsInmem};

/// Type alias for a boxed fallible quads stream.
pub type BoxQuadsStream = BoxStream<'static, Result<ArcQuad, BoxError>>;

/// Quads stream data.
pub struct QuadsStream {
    /// Data stream.
    pub stream: BoxQuadsStream,

    /// Size hint.
    pub size_hint: SizeHint,
}

impl std::fmt::Debug for QuadsStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadsStream")
            .field("size_hint", &self.size_hint)
            .finish()
    }
}

impl From<EcoQuadsInmem> for QuadsStream {
    fn from(value: EcoQuadsInmem) -> Self {
        Self {
            size_hint: SizeHint::with_exact(value.len() as u64),
            stream: Box::pin(futures::stream::iter(
                value.into_inner().0.into_iter().map(Ok),
            )),
        }
    }
}

#[async_trait]
impl async_convert::TryFrom<QuadsStream> for EcoQuadsInmem {
    type Error = BoxError;

    async fn try_from(data: QuadsStream) -> Result<Self, Self::Error> {
        Ok(QuadsInmem::new(CompatDataset(
            data.stream.try_collect::<EcoVec<_>>().await?,
        )))
    }
}

//! I define traits and implementations for derived conneg layers.
//!

use std::{fmt::Debug, sync::Arc};

use manas_http::representation::Representation;
use manas_repo::{service::resource_operator::reader::FlexibleResourceReader, Repo};
use tower::Layer;

pub mod impl_;

/// A trait for derived content negotiation layers.
pub trait DerivedContentNegotiationLayer<R, LRep, S>:
    Debug + Layer<S, Service = Self::WService> + Send + 'static
where
    R: Repo,
    LRep: Representation + Send + 'static,
    S: FlexibleResourceReader<R, R::Representation>,
{
    /// Type of the layer config.
    type Config: Debug + Send + Sync + 'static;

    /// Type of wrapped services.
    type WService: FlexibleResourceReader<R, LRep>;

    /// Create a new layer with given config.
    fn new(config: Arc<Self::Config>) -> Self;
}

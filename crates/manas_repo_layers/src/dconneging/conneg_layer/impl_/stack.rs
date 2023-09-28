//! I define an implementations of [`DerivedContentNegotiationLayer`](DerivedContentNegotiationLayer)
//! that stacks two other layers.
//!

use std::sync::Arc;

use manas_http::representation::Representation;
use manas_repo::{service::resource_operator::reader::FlexibleResourceReader, Repo};
use tower::layer::util::Stack;

use crate::dconneging::conneg_layer::DerivedContentNegotiationLayer;

/// Configuration for stack of layers.
#[derive(Debug)]
pub struct StackConfig<InnerConfig, OuterConfig> {
    /// Inner layer config.
    pub inner_config: Arc<InnerConfig>,

    /// Outer layer config.
    pub outer_config: Arc<OuterConfig>,
}

impl<InnerConfig, OuterConfig> Clone for StackConfig<InnerConfig, OuterConfig> {
    fn clone(&self) -> Self {
        Self {
            inner_config: self.inner_config.clone(),
            outer_config: self.outer_config.clone(),
        }
    }
}

impl<R, ORep, S, Inner, Outer> DerivedContentNegotiationLayer<R, ORep, S> for Stack<Inner, Outer>
where
    R: Repo,
    ORep: Representation + Send + 'static,
    S: FlexibleResourceReader<R, R::Representation>,
    Inner: DerivedContentNegotiationLayer<R, R::Representation, S>,
    Outer: DerivedContentNegotiationLayer<R, ORep, Inner::WService>,
{
    type Config = StackConfig<Inner::Config, Outer::Config>;

    type WService = Outer::WService;

    fn new(config: Arc<Self::Config>) -> Self {
        Self::new(
            Inner::new(config.as_ref().inner_config.clone()),
            Outer::new(config.as_ref().outer_config.clone()),
        )
    }
}

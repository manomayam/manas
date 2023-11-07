use crate::serializer::config::DynSynSerializerConfig;

/// A factory to instantiate [`DynSynQuadSerializer`](super::sync::DynSynQuadSerializer).
#[derive(Debug, Default)]
pub struct DynSynQuadSerializerFactory {
    pub(crate) config: DynSynSerializerConfig,
}

impl DynSynQuadSerializerFactory {
    /// Create a new quads serializer factory with given config.
    #[inline]
    pub fn new(config: DynSynSerializerConfig) -> Self {
        Self { config }
    }
}

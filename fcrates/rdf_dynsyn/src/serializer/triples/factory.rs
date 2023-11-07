use crate::serializer::config::DynSynSerializerConfig;

/// A factory to instantiate [`DynSynTripleSerializer`](super::sync::DynSynTripleSerializer).
#[derive(Debug, Default)]
pub struct DynSynTripleSerializerFactory {
    pub(crate) config: DynSynSerializerConfig,
}

impl DynSynTripleSerializerFactory {
    /// Create a new triples serializer factory with given config.
    #[inline]
    pub fn new(config: DynSynSerializerConfig) -> Self {
        Self { config }
    }
}

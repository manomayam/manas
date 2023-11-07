use crate::parser::config::DynSynParserConfig;

/// A factory to instantiate [`DynSynQuadParser`](super::sync::DynSynQuadParser).
#[derive(Debug, Default)]
pub struct DynSynQuadParserFactory {
    pub(crate) config: DynSynParserConfig,
}

impl DynSynQuadParserFactory {
    /// Create a new quads parser factory with given config.
    #[inline]
    pub fn new(config: DynSynParserConfig) -> Self {
        Self { config }
    }
}

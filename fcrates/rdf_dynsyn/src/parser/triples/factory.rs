use crate::parser::config::DynSynParserConfig;

/// A factory to instantiate [`DynSynTripleParser`](super::sync::DynSynTripleParser).
#[derive(Debug, Default)]
pub struct DynSynTripleParserFactory {
    _config: DynSynParserConfig,
}

impl DynSynTripleParserFactory {
    /// Create a new quads parser factory with given config.
    #[inline]
    pub fn new(config: DynSynParserConfig) -> Self {
        Self { _config: config }
    }
}

use crate::ConfigMap;

/// A factory to instantiate [`DynSynTripleParser`](super::sync::DynSynTripleParser).
#[derive(Debug, Default, Clone)]
pub struct DynSynTripleParserFactory {
    _parser_config_map: ConfigMap,
}

impl DynSynTripleParserFactory {
    /// Instantiate a factory. It takes a `parser_config_map`,
    /// an optional [`ConfigMap`], which can be populated with
    /// configuration structures corresponding to supported syntaxes.
    #[inline]
    pub fn new(parser_config_map: Option<ConfigMap>) -> Self {
        Self {
            _parser_config_map: parser_config_map.unwrap_or_default(),
        }
    }
}

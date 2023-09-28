use crate::ConfigMap;

/// A factory to instantiate [`DynSynQuadParser`](super::sync::DynSynQuadParser).
#[derive(Debug, Default, Clone)]
pub struct DynSynQuadParserFactory {
    _parser_config_map: ConfigMap,
}

impl DynSynQuadParserFactory {
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

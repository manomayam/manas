//! I define types to represent dynsyn parser config.
//!

#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdOptions;

#[cfg(feature = "jsonld")]
use self::jsonld::{DynDocumentLoaderFactory, JsonLdConfig};

#[cfg(feature = "jsonld")]
pub mod jsonld;

/// Config for dynsyn parsers.
#[derive(Debug, Default, Clone)]
pub struct DynSynParserConfig {
    #[cfg(feature = "jsonld")]
    pub(crate) jsonld: Option<JsonLdConfig>,
}

impl DynSynParserConfig {
    #[cfg(feature = "jsonld")]
    /// Get parser config augmented with given jsonld parser config.
    pub fn with_jsonld_config(mut self, config: JsonLdConfig) -> Self {
        self.jsonld = Some(config);
        self
    }

    #[cfg(feature = "jsonld")]
    pub(crate) fn resolved_jsonld_options(&self) -> JsonLdOptions<DynDocumentLoaderFactory> {
        // use tracing::debug;

        // debug!("jsonld config: {:?}", self.jsonld);

        use sophia_jsonld::{loader::NoLoader, loader_factory::DefaultLoaderFactory};

        self.jsonld.as_ref().map_or_else(
            || {
                JsonLdOptions::new().with_document_loader_factory(DynDocumentLoaderFactory::wrap(
                    DefaultLoaderFactory::<NoLoader>::new(),
                ))
            },
            |config| config.clone().options,
        )
    }
}

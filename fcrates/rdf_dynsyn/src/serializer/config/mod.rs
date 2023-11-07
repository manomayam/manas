//! I define types to represent dynsyn serializer config.
//!

use sophia_turtle::serializer::{
    nq::NqConfig, nt::NtConfig, trig::TrigConfig, turtle::TurtleConfig,
};

#[cfg(feature = "rdf-xml")]
use sophia_xml::serializer::RdfXmlConfig;

#[cfg(feature = "jsonld")]
use crate::parser::config::jsonld::{DynDocumentLoader, JsonLdConfig};
#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdOptions;

/// Config for dynsyn parsers.
#[derive(Debug, Default)]
pub struct DynSynSerializerConfig {
    pub(crate) nquads: Option<NqConfig>,
    pub(crate) nt: Option<NtConfig>,
    pub(crate) turtle: Option<TurtleConfig>,
    pub(crate) trig: Option<TrigConfig>,

    #[cfg(feature = "rdf-xml")]
    pub(crate) rdf_xml: Option<RdfXmlConfig>,

    #[cfg(feature = "jsonld")]
    pub(crate) jsonld: Option<JsonLdConfig>,
}

impl DynSynSerializerConfig {
    #[inline]
    /// Get serializer config augmented with given nquads serializer config.
    pub fn with_nquads_config(mut self, config: NqConfig) -> Self {
        self.nquads = Some(config);
        self
    }

    #[inline]
    /// Get serializer config augmented with given nt serializer config.
    pub fn with_nt_config(mut self, config: NtConfig) -> Self {
        self.nt = Some(config);
        self
    }

    #[inline]
    /// Get serializer config augmented with given turtle serializer config.
    pub fn with_turtle_config(mut self, config: TurtleConfig) -> Self {
        self.turtle = Some(config);
        self
    }

    #[inline]
    /// Get serializer config augmented with given trig serializer config.
    pub fn with_trig_config(mut self, config: TrigConfig) -> Self {
        self.trig = Some(config);
        self
    }

    #[cfg_attr(doc_cfg, doc(cfg(feature = "rdf-xml")))]
    #[cfg(feature = "rdf-xml")]
    #[inline]
    /// Get serializer config augmented with given rdf_xml serializer config.
    pub fn with_rdf_xml_config(mut self, config: RdfXmlConfig) -> Self {
        self.rdf_xml = Some(config);
        self
    }

    #[cfg_attr(doc_cfg, doc(cfg(feature = "jsonld")))]
    #[cfg(feature = "jsonld")]
    #[inline]
    /// Get serializer config augmented with given jsonld serializer config.
    pub fn with_jsonld_config(mut self, config: JsonLdConfig) -> Self {
        self.jsonld = Some(config);
        self
    }

    #[cfg(feature = "jsonld")]
    pub(crate) fn resolved_jsonld_options(&self) -> JsonLdOptions<DynDocumentLoader> {
        self.jsonld.as_ref().map_or_else(
            || JsonLdOptions::new().with_document_loader(DynDocumentLoader::new_no_loading()),
            |config| config.effective_options(),
        )
    }
}

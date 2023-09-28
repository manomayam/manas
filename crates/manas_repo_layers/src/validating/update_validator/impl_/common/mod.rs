//! I provide few common utils for implementing rep update validators.
//!

use std::sync::Arc;

use rdf_dynsyn::parser::DynSynParserFactorySet;

/// Config for rdf source rep update validators.
#[derive(Debug, Clone)]
pub struct RdfSourceRepUpdateValidatorConfig {
    /// Dynsyn parser factories.
    pub dynsyn_parser_factories: Arc<DynSynParserFactorySet>,

    /// Maximum size of user supplied rep.
    pub max_user_supplied_rep_size: Option<u64>,
}

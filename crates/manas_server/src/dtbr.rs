//! I define items to help in integrating solidos into recipe
//! as databrowser.
//!

use std::sync::Arc;

use manas_http::{
    header::common::qvalue::QValue, representation::impl_::common::data::bytes_inmem::BytesInmem,
    uri::invariant::AbsoluteHttpUri,
};
use manas_repo_layers::dconneging::conneg_layer::impl_::{
    constant_overriding::{
        ConstantOverrideNegotiationConfig, ConstantOverrideNegotiationLayer,
        ConstantOverridingPreferences,
    },
    stack::StackConfig,
};
use manas_repo_opendal::service::resource_operator::reader::ODRResourceReader;
use once_cell::sync::Lazy;
use tower::{layer::util::Stack, Layer};
use upon::Engine;

use crate::repo::{RcpBaseRepo, RcpBaseRepoSetup, RcpRdfSourceCNL};

static DATABROWSER_TEMPLATE_ENGINE: Lazy<Engine> = Lazy::new(|| {
    let mut engine = Engine::new();
    engine
        .add_template(
            "databrowser",
            include_str!("../templates/databrowser.html.template"),
        )
        .expect("Template must be valid.");
    engine
});

/// A struct for representing databrowser context.
#[derive(serde::Serialize)]
pub struct DatabrowserContext {
    /// Mashlib js uri.
    pub mashlib_js_uri: AbsoluteHttpUri,

    /// Mashlib css uri.
    pub mashlib_css_uri: AbsoluteHttpUri,
}

impl DatabrowserContext {
    /// Get a new [`DatabrowserContext`] with mashlib served from unpkg cdn.
    pub fn new_from_unpkg() -> Self {
        Self {
            mashlib_js_uri: "https://www.unpkg.com/mashlib/dist/mashlib.min.js"
                .parse()
                .unwrap(),
            mashlib_css_uri: "https://www.unpkg.com/mashlib/dist/mash.css"
                .parse()
                .unwrap(),
        }
    }
}

/// Resolve databrowser content for given context.
pub fn resolve_databrowser_content(context: DatabrowserContext) -> BytesInmem {
    DATABROWSER_TEMPLATE_ENGINE
        .get_template("databrowser")
        .expect("Template must exist")
        .render(&context)
        .to_string()
        .expect("Must be valid")
        .into()
}

/// Type of databrowser adapted conneg layer.
pub type DatabrowserAdaptedCNL<Backend, Inner> = Stack<
    Inner,
    ConstantOverrideNegotiationLayer<
        RcpBaseRepo<Backend>,
        <Inner as Layer<ODRResourceReader<RcpBaseRepoSetup<Backend>>>>::Service,
    >,
>;

/// Type of databrowser adapted rdf source conneg layer.
pub type RcpDatabrowserAdaptedRdfSourceCNL<Backend> =
    DatabrowserAdaptedCNL<Backend, RcpRdfSourceCNL<Backend>>;

/// Adapt conneg layer config.
pub fn adapt_dconneg_layer_config<InnerConfig>(
    inner_config: Arc<InnerConfig>,
    opt_databrowser_context: Option<DatabrowserContext>,
) -> Arc<StackConfig<InnerConfig, Option<ConstantOverrideNegotiationConfig>>> {
    let overrider_config = opt_databrowser_context.map(|databrowser_context| {
        ConstantOverrideNegotiationConfig {
            constant_rep_content_type: mime::TEXT_HTML.try_into().expect("Must be valid"),
            constant_rep_complete_data: resolve_databrowser_content(databrowser_context),
            overriding_preferences: Arc::new(ConstantOverridingPreferences {
                min_quality: QValue::ZERO,
                enabled_src_media_ranges: vec![mime::STAR_STAR],
                // TODO disable for images, videos etc.
                disabled_src_media_ranges: vec![],
            }),
        }
    });
    Arc::new(StackConfig {
        inner_config,
        outer_config: Arc::new(overrider_config),
    })
}

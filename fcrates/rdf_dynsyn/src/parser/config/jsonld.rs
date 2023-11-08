//! I define types to represent jsonld parser options.
//!

use std::{fmt::Display, sync::Arc};

use futures::{future::BoxFuture, TryFutureExt};
use json_ld::{syntax::Value, Loader, RemoteDocument};
use locspan::Location;
use rdf_types::IriVocabularyMut;
use sophia_iri::Iri;
use sophia_jsonld::{loader::NoLoader, vocabulary::ArcVoc, JsonLdOptions};

type ArcIri = Iri<Arc<str>>;
type DynDisplay = dyn Display + Send + 'static;
type LoaderFut<'a> = BoxFuture<
    'a,
    Result<RemoteDocument<ArcIri, Location<ArcIri>, Value<Location<ArcIri>>>, Box<DynDisplay>>,
>;

trait _Loader: Send + Sync + 'static {
    fn load(&mut self, url: ArcIri) -> LoaderFut<'_>;
}

static mut VOC: ArcVoc = ArcVoc {};

impl<L> _Loader for L
where
    L: Loader<ArcIri, Location<ArcIri>, Output = Value<Location<ArcIri>>> + Send + Sync + 'static,
    L::Error: Display + Send + 'static,
{
    fn load(&mut self, url: ArcIri) -> LoaderFut<'_> {
        Box::pin(
            Loader::<ArcIri, Location<ArcIri>>::load_with(self, unsafe { &mut VOC }, url)
                .map_err(|e| Box::new(e) as Box<DynDisplay>),
        )
    }
}

/// A loader that wraps over another type erased loader trait objects.
pub struct DynDocumentLoader {
    inner: Box<dyn _Loader>,
}

impl Loader<ArcIri, Location<ArcIri>> for DynDocumentLoader {
    type Output = Value<Location<ArcIri>>;

    type Error = Box<DynDisplay>;

    #[inline]
    fn load_with<'a>(
        &'a mut self,
        _vocabulary: &'a mut (impl Sync + Send + IriVocabularyMut<Iri = ArcIri>),
        url: ArcIri,
    ) -> LoaderFut<'a>
    where
        ArcIri: 'a,
    {
        _Loader::load(self.inner.as_mut(), url)
    }
}

impl DynDocumentLoader {
    /// Create a new [`DynDocumentLoader`] by wrapping a given
    /// inner loader.
    #[inline]
    pub fn new<L>(inner: L) -> Self
    where
        L: Loader<ArcIri, Location<ArcIri>, Output = Value<Location<ArcIri>>>
            + Send
            + Sync
            + 'static,
        L::Error: Display + Send + 'static,
    {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Get a new [`DynDocumentLoader`] that doesn't load any document.
    #[inline]
    pub fn new_no_loading() -> Self {
        DynDocumentLoader::new(NoLoader::new())
    }
}

/// Type for jsonld parser/serializer configuration.
pub struct JsonLdConfig {
    /// jsonld options.
    pub options: JsonLdOptions<NoLoader>,

    /// Context loader factory.
    pub context_loader_factory: Arc<dyn Fn() -> DynDocumentLoader + Send + Sync>,
}

impl std::fmt::Debug for JsonLdConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonLdConfig").finish()
    }
}

impl JsonLdConfig {
    /// Resolve effective options.
    pub fn effective_options(&self) -> JsonLdOptions<DynDocumentLoader> {
        let mut options = JsonLdOptions::new()
            .with_document_loader((self.context_loader_factory)())
            .with_compact_arrays(self.options.compact_arrays)
            .with_compact_to_relative(self.options.compact_to_relative)
            .with_ordered(self.options.ordered)
            .with_processing_mode(self.options.processing_mode)
            .with_produce_generalized_rdf(self.options.produce_generalized_rdf)
            .with_use_native_types(self.options.use_native_types())
            .with_use_rdf_type(self.options.use_rdf_type())
            .with_expansion_policy(self.options.expansion_policy)
            .with_spaces(self.options.spaces());

        if let Some(v) = &self.options.base {
            options = options.with_base(v.clone());
        }
        // if let Some(v) = &self.options.expand_context {
        //     options = options.with_expand_context(v.clone());
        // }

        options
    }
}

#[cfg(feature = "jsonld-http-loader")]
mod http_loader {
    // use reqwest::Client;

    // /// A [`Loader``] that loads http served documents.
    // pub struct HttpDocumentLoader {
    //     client: Client,
    // }
}

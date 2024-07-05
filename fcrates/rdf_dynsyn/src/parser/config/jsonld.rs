//! I define types to represent jsonld parser options.
//!

use std::{fmt::Display, ptr::addr_of_mut, sync::Arc};

use futures::{future::BoxFuture, TryFutureExt};
use json_ld::{syntax::Value, Loader, RemoteDocument};
use locspan::Location;
use rdf_types::IriVocabularyMut;
use sophia_iri::Iri;
pub use sophia_jsonld::JsonLdOptions;
use sophia_jsonld::{
    loader::NoLoader,
    loader_factory::{DefaultLoaderFactory, LoaderFactory},
    vocabulary::ArcVoc,
};

type ArcIri = Iri<Arc<str>>;
type DynDisplay = dyn Display + Send + 'static;
type BasicRemoteDocument = RemoteDocument<ArcIri, Location<ArcIri>, Value<Location<ArcIri>>>;
type LoaderFut<'a> = BoxFuture<'a, Result<BasicRemoteDocument, Box<DynDisplay>>>;

trait _Loader: Send + Sync + 'static {
    fn load(&mut self, url: ArcIri) -> LoaderFut<'_>;
}

// The ArcVoc is unit struct without any state. But implements jsonld's
// vocabulary traits. Hence safe for static mut.
static mut VOC: ArcVoc = ArcVoc {};

impl<L> _Loader for L
where
    L: Loader<ArcIri, Location<ArcIri>, Output = Value<Location<ArcIri>>> + Send + Sync + 'static,
    L::Error: Display + Send + 'static,
{
    fn load(&mut self, url: ArcIri) -> LoaderFut<'_> {
        Box::pin(
            // See https://github.com/rust-lang/rust/issues/114447
            Loader::<ArcIri, Location<ArcIri>>::load_with(self, unsafe { &mut *addr_of_mut!(VOC) }, url).map_err(
                |e| {
                    error!("Error in loading the document. {}", e);
                    Box::new(e) as Box<DynDisplay>
                },
            ),
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
        // info!("In dyn document loader's load_with");
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

/// A factory that yields [`DynDocumentLoader`].
#[derive(Clone)]
pub struct DynDocumentLoaderFactory {
    inner: Arc<dyn Fn() -> DynDocumentLoader + Send + Sync>,
}

impl Default for DynDocumentLoaderFactory {
    #[inline]
    fn default() -> Self {
        Self::wrap(DefaultLoaderFactory::<NoLoader>::new())
    }
}

impl DynDocumentLoaderFactory {
    /// Create a new [`DynDocumentLoaderFactory`]
    pub fn new(inner: Arc<dyn Fn() -> DynDocumentLoader + Send + Sync>) -> Self {
        Self { inner }
    }

    /// Wrap the given loader factory.
    pub fn wrap<LF>(factory: LF) -> Self
    where
        LF: LoaderFactory + Send + Sync + 'static,
        LF::LoaderError: Display + Send + 'static,
        for<'a> LF::Loader<'a>: Loader<ArcIri, Location<ArcIri>, Output = Value<Location<ArcIri>>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            inner: Arc::new(move || DynDocumentLoader::new(factory.yield_loader())),
        }
    }
}

impl LoaderFactory for DynDocumentLoaderFactory {
    type Loader<'l> = DynDocumentLoader
    where
        Self: 'l;

    type LoaderError = Box<DynDisplay>;

    #[inline]
    fn yield_loader(&self) -> Self::Loader<'_> {
        (self.inner)()
    }
}

/// Type for jsonld parser/serializer configuration.
#[derive(Default)]
pub struct JsonLdConfig {
    /// jsonld options.
    pub options: JsonLdOptions<DynDocumentLoaderFactory>,
}

impl Clone for JsonLdConfig {
    fn clone(&self) -> Self {
        let mut options = JsonLdOptions::new()
            .with_compact_arrays(self.options.compact_arrays)
            .with_compact_to_relative(self.options.compact_to_relative)
            .with_ordered(self.options.ordered)
            .with_processing_mode(self.options.processing_mode)
            .with_produce_generalized_rdf(self.options.produce_generalized_rdf)
            .with_use_native_types(self.options.use_native_types())
            .with_use_rdf_type(self.options.use_rdf_type())
            .with_expansion_policy(self.options.expansion_policy)
            .with_spaces(self.options.spaces())
            .with_document_loader_factory(self.options.document_loader_factory().clone());

        if let Some(v) = &self.options.base {
            options = options.with_base(v.clone());
        }
        // if let Some(v) = &self.options.expand_context {
        //     options = options.with_expand_context(v.clone());
        // }

        Self { options }
    }
}

impl std::fmt::Debug for JsonLdConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonLdConfig").finish()
    }
}

#[cfg(feature = "jsonld-http-loader")]
pub use http_loader::*;
use tracing::error;

#[cfg(feature = "jsonld-http-loader")]
mod http_loader {
    use std::{string::FromUtf8Error, sync::Arc};

    use futures::future::BoxFuture;
    use headers::{ContentType, HeaderMapExt};
    use http_typed_headers::{
        accept::Accept, define_static_rel_types, link::Link, location::Location, HeaderMap,
    };
    use json_ld::{Loader, Profile, RemoteDocument};
    use json_syntax::{Parse, Value};
    use mime::APPLICATION_JSON;
    use rdf_types::IriVocabularyMut;
    use reqwest::{redirect::Policy, StatusCode};
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware};
    use tracing::{debug, error, info};

    use crate::media_type::APPLICATION_JSON_LD;

    use super::{ArcIri, BasicRemoteDocument, DynDocumentLoader};

    /// Options for [`HttpDocumentLoader`].
    #[derive(Debug, Clone)]
    pub struct HttpDocumentLoaderOptions {
        /// IRIs to use in the request as a profile parameter.
        pub request_profile: Vec<ArcIri>,

        /// Maximum number of redirections.
        pub max_redirections: u8,
    }

    /// HTTP body parse error.
    #[derive(Debug, thiserror::Error)]
    pub enum JsonDocumentParseError<M = locspan::Location<ArcIri>> {
        /// Invalid encoding.
        #[error("Invalid encoding.")]
        InvalidEncoding(FromUtf8Error),

        /// JSON parse error.
        #[error("Json parse error: {0}")]
        Json(json_ld::syntax::parse::MetaError<M>),
    }

    /// Error in loading http document.
    #[derive(Debug, thiserror::Error)]
    pub enum HttpDocumentLoadingError {
        /// Redirection with 303.
        #[error("Redirection with 303.")]
        Redirection303,

        /// Redirection without location header.
        #[error("Redirection without location header.")]
        MissingRedirectionLocation,

        /// Invalid redirection uri.
        #[error("Invalid redirection uri.")]
        InvalidRedirectionUri,

        /// Invalid content type.
        #[error("Invalid content type.")]
        InvalidContentType,

        /// Multiple context link headers.
        #[error("Multiple context link headers.")]
        MultipleContextLinkHeaders,

        /// Too many alternate redirections.
        #[error("Too many redirections.")]
        TooManyRedirections,

        /// Parsing error.
        #[error("Parsing error.")]
        ParseError(JsonDocumentParseError<locspan::Location<ArcIri>>),

        /// Query error.
        #[error("Query error.")]
        QueryError(StatusCode),

        /// Unknown io error.
        #[error("Unknown io error.")]
        UnknownIoError(reqwest_middleware::Error),
    }

    /// A [`Loader``] that loads http served json-ld documents.
    /// It will honour non-303 redirections and "alternate"
    /// linked redirections as specified in jsonld-api spec.
    #[derive(Debug, Clone)]
    pub struct HttpDocumentLoader {
        /// Client.
        client: ClientWithMiddleware,

        /// Loader options.
        options: HttpDocumentLoaderOptions,

        /// Effective `Accept` header for conneg.
        accept: Accept,
    }

    define_static_rel_types!(
        /// "alternate" rel type.
        ALTERNATE: "alternate";

        /// Json-ld context rel type.
        CONTEXT: "http://www.w3.org/ns/json-ld#context";
    );

    impl HttpDocumentLoader {
        /// Create a new [`HttpDocumentLoader`] from given params.
        /// The middleware must not influence client's
        /// redirection policy.
        #[inline]
        pub fn new(
            options: HttpDocumentLoaderOptions,
            client_middleware: Option<Arc<dyn Middleware>>,
        ) -> Self {
            let mut jsonld_mt_range = String::from(APPLICATION_JSON_LD.essence_str());
            if !options.request_profile.is_empty() {
                jsonld_mt_range.push_str(&format!(
                    ";profile=\"{}\"",
                    options.request_profile.join("")
                ));
            }

            let accept = Accept {
                accept_values: vec![
                    jsonld_mt_range.parse().expect("Must be valid"),
                    APPLICATION_JSON.try_into().expect("Must be valid"),
                ],
            };

            let mut client_builder = ClientBuilder::new(
                reqwest::ClientBuilder::new()
                    // Set redirect policy to none, as we have to
                    //handle for 303, and alternate links.
                    .redirect(Policy::none())
                    .build()
                    .expect("Must be valid client."),
            );

            if let Some(middleware) = client_middleware {
                client_builder = client_builder.with_arc(middleware);
            }

            Self {
                options,
                client: client_builder.build(),
                accept,
            }
        }

        fn load_remote_doc<'a>(
            &'a self,
            vocabulary: &'a mut (impl Send + Sync + IriVocabularyMut<Iri = ArcIri>),
            uri: ArcIri,
            redirections_count: u8,
        ) -> BoxFuture<'a, Result<BasicRemoteDocument, HttpDocumentLoadingError>> {
            debug!("Loading {}", uri.as_str());
            Box::pin(async move {
                // Ensure alt redirections limit.
                if (redirections_count > self.options.max_redirections)
                    || (redirections_count == u8::MAX)
                {
                    error!("Too many redirections ({}).", redirections_count);
                    return Err(HttpDocumentLoadingError::TooManyRedirections);
                }

                debug!("Requesting {}", uri.as_str());
                let mut headers = HeaderMap::new();
                headers.typed_insert(self.accept.clone());
                let resp = self
                    .client
                    .get(uri.as_str())
                    .headers(headers)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Unknown io error in retrieving remote document. {}", e);
                        HttpDocumentLoadingError::UnknownIoError(e)
                    })?;

                let status = resp.status();

                // Return error, if status is not in [200, 400).
                if (status.as_u16() < 200) || (status.as_u16() >= 400) {
                    error!("Status \"{}\" is not in range of [200, 400)", status);
                    return Err(HttpDocumentLoadingError::QueryError(status));
                }

                // Return error for 303.
                if status == StatusCode::SEE_OTHER {
                    error!("Status code is 303");
                    return Err(HttpDocumentLoadingError::Redirection303);
                }

                if status.is_redirection() {
                    debug!("Status is non-303 redirection.");
                    let location = resp
                        .headers()
                        .typed_try_get::<Location>()
                        .map_err(|e| {
                            error!("Invalid Location header. {}", e);
                            HttpDocumentLoadingError::InvalidRedirectionUri
                        })?
                        .ok_or_else(|| {
                            error!("No Location header for redirect response");
                            HttpDocumentLoadingError::MissingRedirectionLocation
                        })?;

                    let location_abs = resp.url().join(location.0.as_str()).map_err(|e| {
                        error!("Invalid Location header. {}", e);
                        HttpDocumentLoadingError::InvalidRedirectionUri
                    })?;

                    let location_abs = vocabulary.insert_owned(
                        location_abs
                            .to_string()
                            .try_into()
                            .expect("Must be valid iri."),
                    );

                    info!("Redirecting to {}", location_abs);
                    return self
                        .load_remote_doc(
                            vocabulary,
                            location_abs,
                            redirections_count.saturating_add(1),
                        )
                        .await;
                }

                let cty: mime::Mime = resp
                    .headers()
                    .typed_get::<ContentType>()
                    .unwrap_or_else(ContentType::octet_stream)
                    .into();

                debug!("Response content type: {}", cty);

                let is_json_cty = (cty == APPLICATION_JSON) || (cty.suffix() == Some(mime::JSON));
                let h_link = resp.headers().typed_get::<Link>();

                if !is_json_cty {
                    let alt_uri = h_link
                        .and_then(|link| {
                            link.values.into_iter().find_map(|v| {
                                let targets_jsonld_doc = v
                                    .params()
                                    .get_value(&"type".parse().unwrap())
                                    .map(|v| v.as_ref() == APPLICATION_JSON_LD.essence_str())
                                    .unwrap_or_default();
                                (v.rel().rel_types.contains(&*ALTERNATE) && targets_jsonld_doc)
                                    .then_some(
                                        resp.url()
                                            .join(v.target().as_str())
                                            .expect("Must be valid"),
                                    )
                            })
                        })
                        .ok_or_else(|| {
                            error!("Non json resource doesn't have alternate link");
                            HttpDocumentLoadingError::InvalidContentType
                        })?;

                    let alt_uri = vocabulary
                        .insert_owned(alt_uri.to_string().try_into().expect("Must be valid iri."));

                    info!("Following alternative link to {}", alt_uri);
                    return self
                        .load_remote_doc(vocabulary, alt_uri, redirections_count.saturating_add(1))
                        .await;
                }

                let mut context_uri = None;

                if cty.essence_str() != APPLICATION_JSON_LD.essence_str() {
                    let context_links = h_link
                        .map(|mut link| {
                            link.values
                                .retain(|v| v.rel().rel_types.contains(&*CONTEXT));
                            link.values
                        })
                        .unwrap_or_default();

                    if context_links.len() > 1 {
                        error!("multiple context links.");
                        return Err(HttpDocumentLoadingError::MultipleContextLinkHeaders);
                    }

                    context_uri = context_links.first().map(|v| {
                        vocabulary.insert_owned(
                            v.target()
                                .0
                                .to_string()
                                .try_into()
                                .expect("Must be valid iri."),
                        )
                    });
                }

                let profiles = cty
                    .params()
                    .filter_map(|(k, v)| {
                        iref::Iri::new(v.as_str()).ok().and_then(|v_iri| {
                            (k.as_str() == "profile").then_some(Profile::new(v_iri, vocabulary))
                        })
                    })
                    .collect();

                let bytes = resp
                    .bytes()
                    .await
                    .map_err(|e| HttpDocumentLoadingError::UnknownIoError(e.into()))?;

                let document = String::from_utf8(bytes.to_vec())
                    .map_err(JsonDocumentParseError::InvalidEncoding)
                    .and_then(|content| {
                        Value::parse_str(&content, |span| {
                            locspan::Location::new(uri.clone(), span)
                        })
                        .map_err(JsonDocumentParseError::Json)
                    })
                    .map_err(|e| {
                        error!("Invalid document content. {}", e);
                        HttpDocumentLoadingError::ParseError(e)
                    })?;

                // let document = ;

                Ok(RemoteDocument::new_full(
                    Some(uri),
                    Some(cty),
                    context_uri,
                    profiles,
                    document,
                ))
            })
        }
    }

    impl Loader<ArcIri, locspan::Location<ArcIri>> for HttpDocumentLoader {
        type Output = Value<locspan::Location<ArcIri>>;

        type Error = HttpDocumentLoadingError;

        fn load_with<'a>(
            &'a mut self,
            vocabulary: &'a mut (impl Sync + Send + IriVocabularyMut<Iri = ArcIri>),
            url: ArcIri,
        ) -> BoxFuture<
            'a,
            json_ld::LoadingResult<ArcIri, locspan::Location<ArcIri>, Self::Output, Self::Error>,
        >
        where
            ArcIri: 'a,
        {
            self.load_remote_doc(vocabulary, url, 0)
        }
    }

    impl From<HttpDocumentLoader> for DynDocumentLoader {
        #[inline]
        fn from(value: HttpDocumentLoader) -> Self {
            Self::new(value)
        }
    }
}

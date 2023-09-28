//! I define simple webid profile requesting agent.
//!

use futures::StreamExt;
use headers::{ContentType, HeaderMapExt};
use http_uri::invariant::AbsoluteHttpUri;
use iri_string::types::UriStr;
use mime::Mime;
use rdf_dynsyn::{
    parser::triples::DynSynTripleParserFactory, syntax::invariant::triples_parsable::TP_TURTLE,
};
use reqwest::{
    header::{ACCEPT, LOCATION},
    redirect::Policy,
    Client, StatusCode,
};
use sophia_api::{graph::MutableGraph, prelude::Iri, term::SimpleTerm, triple::Triple};
use tracing::error;

use crate::WebId;

/// Web id profile requesting agent.
///
/// A [`WebIdProfileReqAgent`] requests as a public client
/// and is capable of parsing of only turtle profile documents.
#[derive(Debug, Clone)]
pub struct WebIdProfileReqAgent {
    /// Http client.
    client: Client,

    /// Http client with no redirects.
    client_no_redirects: Client,

    /// Triple parser factory.
    triple_parser_factory: DynSynTripleParserFactory,
}

impl Default for WebIdProfileReqAgent {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl WebIdProfileReqAgent {
    /// Get a new [`WebIdProfileReqAgent]
    pub fn new() -> Self {
        let client = Client::new();
        let client_no_redirects = Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("Tls backend cannot be initialized for reqwest client.");

        Self {
            client,
            client_no_redirects,
            triple_parser_factory: DynSynTripleParserFactory::new(None),
        }
    }

    /// Try to resolve profile doc uri.
    /// If webid has fragment part, then absolute part of it will be it's profile doc uri.
    /// Otherwise, the value of  `Location`  header filed in
    /// redirect response will be profile doc uri.
    pub async fn try_resolve_profile_doc_uri(
        &self,
        webid: impl AsRef<WebId>,
    ) -> Result<AbsoluteHttpUri, ProfileDocResolutionError> {
        let webid = webid.as_ref();

        // If web id has a fragment, then absolute part is it's profile doc's uri.
        if webid.fragment().is_some() {
            return Ok(
                AbsoluteHttpUri::try_new_from::<&UriStr>(webid.to_absolute().as_ref())
                    .expect("Must be valid."),
            );
        }

        // Otherwise, the value of  `Location`  header filed in
        // redirect response will be profile doc uri.

        let resp = self
            .client_no_redirects
            .head(webid.as_str())
            .send()
            .await
            .map_err(|e| {
                error!(
                    "Unknown io error in dereferencing web id profile doc. Error:\n {}",
                    e
                );
                e
            })?;

        if resp.status() == StatusCode::SEE_OTHER {
            if let Some(loc) = resp
                .headers()
                .get(LOCATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|loc_str| AbsoluteHttpUri::try_new_from(loc_str).ok())
            {
                return Ok(loc);
            }
        }

        Err(ProfileDocResolutionError::InvalidDerefResponse)
    }

    /// Try to get webid profile document.
    pub async fn try_get_profile_document<G: Default + MutableGraph>(
        &self,
        webid: &WebId,
    ) -> Result<G, ProfileDocResolutionError> {
        let resp = self
            .client
            .get(webid.as_str())
            .header(ACCEPT, "text/turtle")
            .send()
            .await
            .map_err(|e| {
                error!(
                    "Unknown io error in dereferencing web id profile doc. Error:\n {}",
                    e
                );
                e
            })?;

        if !resp.status().is_success() {
            error!("Deref request not successful. status: {}", resp.status());
            return Err(ProfileDocResolutionError::InvalidDerefResponse);
        }

        // If turtle is not available, reject.
        if !resp
            .headers()
            .typed_get::<ContentType>()
            .map(|cty| Mime::from(cty).essence_str() == "text/turtle")
            .unwrap_or_default()
        {
            error!("Profile document is not available in turtle.");
            return Err(ProfileDocResolutionError::InvalidDerefResponse);
        }

        // Parse the graph.
        let parser = self
            .triple_parser_factory
            .new_async_parser(TP_TURTLE, Some(Iri::new_unchecked(resp.url().to_string())));

        let mut triples_stream = parser
            .parse_stream::<SimpleTerm, _>(resp.bytes_stream())
            .await;

        let mut graph = G::default();

        while let Some(triple_r) = triples_stream.next().await {
            let triple = triple_r.map_err(|e| {
                error!("Error in parsing profile doc. Error:\n {}", e);
                ProfileDocResolutionError::InvalidProfileContent
            })?;
            graph
                .insert(triple.s(), triple.p(), triple.o())
                .map_err(|_| ProfileDocResolutionError::InvalidProfileContent)?;
        }

        Ok(graph)
    }
}

/// An error type for representing errors in profile doc resolution.
#[derive(Debug, thiserror::Error)]
pub enum ProfileDocResolutionError {
    /// Invalid deref response.
    #[error("Invalid deref response.")]
    InvalidDerefResponse,

    /// Invalid profile content.
    #[error("Invalid profile content.")]
    InvalidProfileContent,

    /// Unknown io error.
    #[error("Unknown io error.")]
    UnknownIoError(#[from] reqwest::Error),
}

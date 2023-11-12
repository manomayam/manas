//! conneg over derived rdf binary doc representations of
//! inner resolved rdf representation.
//!

use std::{collections::HashSet, marker::PhantomData, ops::Deref, sync::Arc};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, ProbResult, Problem};
use futures::{FutureExt, TryFutureExt};
use if_chain::if_chain;
use manas_http::{
    header::{
        accept::Accept,
        common::media_type::{MediaType, TEXT_TURTLE},
    },
    representation::{
        impl_::{
            basic::BasicRepresentation, binary::BinaryRepresentation,
            common::data::bytes_inmem::BytesInmem,
        },
        metadata::{KDerivedETag, KLastModified},
    },
};
use manas_repo::{
    service::resource_operator::reader::{
        rep_preferences::range_negotiator::impl_::{
            DConnegLayeredRangeNegotiator, DContentTypeNegotiator,
        },
        FlexibleResourceReader, ResourceReadRequest, ResourceReadResponse,
    },
    Repo,
};
use manas_space::resource::state::invariant::RepresentedSolidResourceState;
use once_cell::sync::Lazy;
use rdf_dynsyn::{
    correspondence::{Correspondent, SYNTAX_TO_MEDIA_TYPE_CORRESPONDENCE},
    syntax::invariant::{
        parsable::DynSynParsableSyntax,
        serializable::{DynSynSerializableSyntax, S_ALL},
    },
    DynSynFactorySet,
};
use rdf_utils::model::{dataset::EcoDataset, quad::ArcQuad};
use tower::{Layer, Service};
use tracing::{debug, error, info, warn};
use typed_record::TypedRecord;

use crate::dconneging::conneg_layer::DerivedContentNegotiationLayer;

/// An implementation of
/// [`DerivedContentNegotiationLayer`] that wraps a reader to
/// conneg over derived rdf concrete doc representations of
/// inner resolved rdf representation.
#[derive(Clone)]
pub struct BinaryRdfDocContentNegotiationLayer<R, S> {
    config: Arc<BinaryRdfDocContentNegotiationConfig>,
    _phantom: PhantomData<fn(R, S)>,
}

impl<R, S> std::fmt::Debug for BinaryRdfDocContentNegotiationLayer<R, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinaryRdfDocContentNegotiationLayer")
            .field("config", &self.config)
            .finish()
    }
}

impl<R, S> Layer<S> for BinaryRdfDocContentNegotiationLayer<R, S>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    S: FlexibleResourceReader<R, R::Representation>,
{
    type Service = BinaryRdfDocContentNegotiatingResourceReader<R, S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        BinaryRdfDocContentNegotiatingResourceReader {
            inner,
            config: self.config.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R, S> DerivedContentNegotiationLayer<R, BinaryRepresentation, S>
    for BinaryRdfDocContentNegotiationLayer<R, S>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    S: FlexibleResourceReader<R, R::Representation>,
{
    type Config = BinaryRdfDocContentNegotiationConfig;

    type WService = BinaryRdfDocContentNegotiatingResourceReader<R, S>;

    fn new(config: Arc<Self::Config>) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

/// A layered resource reader that connegs over derived rdf
/// concrete doc representations of inner resolved rdf
/// representation.
///
#[derive(Debug)]
pub struct BinaryRdfDocContentNegotiatingResourceReader<R, Inner> {
    inner: Inner,
    config: Arc<BinaryRdfDocContentNegotiationConfig>,
    _phantom: PhantomData<fn(R)>,
}

impl<R, Inner: Clone> Clone for BinaryRdfDocContentNegotiatingResourceReader<R, Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            config: self.config.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R, Inner> Service<ResourceReadRequest<R>>
    for BinaryRdfDocContentNegotiatingResourceReader<R, Inner>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    Inner: FlexibleResourceReader<R, R::Representation>,
{
    type Response = ResourceReadResponse<R, BinaryRepresentation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(skip_all, name = "BinaryRdfDocContentNegotiatingResourceReader::call")]
    fn call(&mut self, req: ResourceReadRequest<R>) -> Self::Future {
        let config = self.config.clone();

        // Create derived-content negotiator.
        let accept = req.rep_conneg_params.accept.clone();

        // Create request to inner service
        let mut inner_req = req;

        // Pass on new range negotiator, that takes dconneg into account.
        inner_req.rep_preferences.non_container_rep_range_negotiator =
            Box::new(DConnegLayeredRangeNegotiator {
                outer: inner_req.rep_preferences.non_container_rep_range_negotiator,
                dconneger: BinaryRdfDocContentTypeNegotiator {
                    accept: accept.clone(),
                },
            });

        // Create inner future by calling inner service.
        let inner_fut = self.inner.call(inner_req);

        Box::pin(async move {
            // Get inner response.
            let inner_resp: ResourceReadResponse<R, _> = inner_fut
                .inspect_ok(|_| info!("Success in calling inner service."))
                .inspect_err(|e| {
                    error!("Error in calling inner service. Error:\n {}", e);
                })
                .await?
                .map_representation(Into::into);

            resolve_negotiated_response(inner_resp, accept, config).await
        })
    }
}

impl<R, Inner> FlexibleResourceReader<R, BinaryRepresentation>
    for BinaryRdfDocContentNegotiatingResourceReader<R, Inner>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    Inner: FlexibleResourceReader<R, R::Representation>,
{
}

/// Config for [`BinaryRdfDocContentNegotiationLayer`].
#[derive(Debug, Clone, Default)]
pub struct BinaryRdfDocContentNegotiationConfig {
    /// Dynsyn factories.
    pub dynsyn_factories: DynSynFactorySet,
}

/// Resolve negotiated response.
pub(super) async fn resolve_negotiated_response<R: Repo>(
    inner_resp: ResourceReadResponse<R, BinaryRepresentation>,
    accept: Option<Accept>,
    conneg_config: Arc<BinaryRdfDocContentNegotiationConfig>,
) -> ProbResult<ResourceReadResponse<R, BinaryRepresentation>> {
    let inner_rep_metadata = inner_resp.state.representation_metadata();

    // Get content-type of inner rep.
    let inner_rep_content_type = inner_rep_metadata.content_type();

    // Create derived-content negotiator.
    let dconneger = BinaryRdfDocContentTypeNegotiator { accept };
    // Resolve derived rep content type.
    let dcontent_type = dconneger.resolve_pref_derived_content_type(inner_rep_content_type);

    debug!(
        "Inner rep content type: {}, Resolved dcontent type: {}",
        inner_rep_content_type, dcontent_type
    );

    // If inner rep content-type is same as resolved derived content-type,
    // then pass on inner state.
    if inner_rep_content_type == &dcontent_type {
        return Ok(inner_resp);
    }

    // Now, we know that dcontent-type is different from that of inner.
    // And `dcontent-type` is dynsyn parsable as dconneger guarantees.

    info!("Inner representation doesn't satisfies conneg resolved preferences.");

    // Compute derived representation.
    let (slot, inner_rep) = inner_resp.state.into_parts();

    // Resolve rdf syntax of derived rep.
    let dsyntax = Correspondent::<DynSynSerializableSyntax>::try_from(&*dcontent_type)
        .expect("Must be ok, as dconneger guarantees.")
        .value;

    // Compute converted rep.
    // TODO Must optimize.

    // Convert into inmemory rep.
    let rep_inmem: BasicRepresentation<BytesInmem> =
        async_convert::TryFrom::try_from(inner_rep.into_basic())
            .await
            .map_err(|e| {
                error!("Error in converting source rep into inmem rep. {e}");
                UNKNOWN_IO_ERROR
                    .new_problem_builder()
                    .source_in_a_box(e)
                    .finish()
            })?;

    let mut effective_rep = None;
    if let Some(Ok(quads_inmem)) = rep_inmem
        .try_parse_quads::<EcoDataset<ArcQuad>>(conneg_config.dynsyn_factories.parser.clone())
        .inspect(|v| {
            match v.as_ref() {
                Some(Err(e)) => {
                    warn!("Error in parsing quads. {}", e);
                }
                None => {
                    warn!("Negotiator intervened even as the source syntax is not parsable.");
                }
                _ => (),
            };
        })
        .await
    {
        // TODO must cache rdf data.
        // Serialize resultant quads.
        if let Ok(mut converted_rep_inmem) = BasicRepresentation::try_from_wrap_serializing_quads(
            quads_inmem,
            conneg_config.dynsyn_factories.serializer.clone(),
            dsyntax,
        )
        .inspect_err(|e| {
            warn!("Error in wrap serializing quads. Error:{}", e);
        })
        .await
            as Result<BasicRepresentation<BytesInmem>, _>
        {
            // Pass on metadata.
            converted_rep_inmem.metadata = converted_rep_inmem
                .metadata
                .with_opt::<KDerivedETag>(
                    rep_inmem
                        .metadata
                        .get_rv::<KDerivedETag>()
                        .map(|base_etag| base_etag.derived_rep_etag(("rdf_serializing", &dsyntax))),
                )
                .with_opt::<KLastModified>(rep_inmem.metadata.get_rv::<KLastModified>().copied());

            effective_rep = Some(converted_rep_inmem)
        }
    }

    let effective_rep = effective_rep.unwrap_or(rep_inmem).into_binary();

    Ok(ResourceReadResponse {
        state: RepresentedSolidResourceState::new(slot, effective_rep),
        aux_links_index: inner_resp.aux_links_index,
        tokens: inner_resp.tokens,
        extensions: inner_resp.extensions,
    })
}

/// An implementation of [`DContentTypeNegotiator`], that
/// negotiates as per `Accept` when base rep is a binary rdf doc rep.
#[derive(Debug, Clone)]
pub struct BinaryRdfDocContentTypeNegotiator {
    /// `Accept` header against which to negotiate.
    pub accept: Option<Accept>,
}

/// Set of content-types into which rdf statements can be serialized.
static RDF_SERIALIZABLE_CONTENT_TYPES: Lazy<HashSet<MediaType>> = Lazy::new(|| {
    let mut result = HashSet::new();
    // For each serializable syntax, include corresponding content-type.
    for syntax in S_ALL {
        if_chain! {
            // If there is a correspondent media type.
            if let Some(correspondent) = SYNTAX_TO_MEDIA_TYPE_CORRESPONDENCE.get(syntax);
            // And correspondence is total,
            if correspondent.is_total;
            // Media-Type is valid
            if let Ok(content_type) = MediaType::try_from(correspondent.value.clone());

            then {
                result.insert(content_type);
            }
        }
    }
    result
});

impl DContentTypeNegotiator for BinaryRdfDocContentTypeNegotiator {
    fn resolve_pref_derived_content_type(self, base_content_type: &MediaType) -> MediaType {
        // Check if source rep is an rdf source rep.
        let is_binary_rdf_base =
            Correspondent::<DynSynParsableSyntax>::try_from(base_content_type.deref())
                .map(|c| c.is_total)
                .unwrap_or_default();

        // If it is not an binary rdf base rep, then prefer base content type.
        if !is_binary_rdf_base {
            return base_content_type.clone();
        }

        // Try to satisfy accept.
        if let Some(mut accept) = self.accept {
            accept.sort_accept_values_by_precedence();

            // Resolve all available content-types,
            // with turtle and base content-type being first two.
            // LDP requires turtle to be preferred, in cases of tie.
            let mut available_content_types = vec![&*TEXT_TURTLE, base_content_type];
            available_content_types.extend(RDF_SERIALIZABLE_CONTENT_TYPES.iter());

            for accept_value in accept.accept_values {
                for content_type in available_content_types.iter() {
                    if accept_value.matches(content_type, false) {
                        return (*content_type).clone();
                    }
                }
            }
        }

        // Else, prefer base rep content type.
        base_content_type.clone()
    }
}

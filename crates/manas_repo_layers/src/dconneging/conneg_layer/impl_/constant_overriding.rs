//! I define an implementation of
//! [`DerivedContentNegotiationLayer`] that warps a reader
//! to  conneg between configured constant representation and
//! inner representation.
//!

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use dyn_problem::{ProbFuture, Problem};
use futures::TryFutureExt;
use if_chain::if_chain;
use manas_http::{
    header::{
        accept::{Accept, MediaRangeSpecificity},
        common::{media_type::MediaType, qvalue::QValue},
    },
    representation::{
        impl_::{
            basic::BasicRepresentation, binary::BinaryRepresentation,
            common::data::bytes_inmem::BytesInmem,
        },
        metadata::{KContentType, KDerivedETag, KLastModified, RepresentationMetadata},
        Representation,
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
use tower::{Layer, Service};
use tracing::{debug, error, info};
use typed_record::TypedRecord;

use crate::dconneging::conneg_layer::DerivedContentNegotiationLayer;

/// Config for [`ConstantOverrideNegotiatingResourceReader`].
#[derive(Debug, Clone)]
pub struct ConstantOverrideNegotiationConfig {
    /// Constant representation's complete data..
    pub constant_rep_complete_data: BytesInmem,

    /// Constant rep's content-type.
    pub constant_rep_content_type: MediaType,

    /// Overriding preferences.
    pub overriding_preferences: Arc<ConstantOverridingPreferences>,
}

/// Preferences for overriding.
#[derive(Debug, Clone)]
pub struct ConstantOverridingPreferences {
    /// The minimum requested quality for overriding.
    pub min_quality: QValue,

    /// Source rep media ranges, for which overriding is enabled.
    pub enabled_src_media_ranges: Vec<mime::Mime>,

    /// Source rep media ranges, for which overriding is enabled.
    pub disabled_src_media_ranges: Vec<mime::Mime>,
}

impl Default for ConstantOverridingPreferences {
    fn default() -> Self {
        Self {
            min_quality: QValue::ZERO,
            enabled_src_media_ranges: vec![mime::STAR_STAR],
            disabled_src_media_ranges: Default::default(),
        }
    }
}

/// An implementation of
/// [`DerivedContentNegotiationLayer`] that wraps a reader to
/// conneg between configured constant representation and
/// inner representation.
#[derive(Clone)]
pub struct ConstantOverrideNegotiationLayer<R, S> {
    config: Arc<Option<ConstantOverrideNegotiationConfig>>,
    _phantom: PhantomData<fn(R, S)>,
}

impl<R, S> std::fmt::Debug for ConstantOverrideNegotiationLayer<R, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstantOverrideNegotiationLayer")
            .field("config", &self.config)
            .finish()
    }
}

impl<R, S> Layer<S> for ConstantOverrideNegotiationLayer<R, S>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    S: FlexibleResourceReader<R, R::Representation>,
{
    type Service = ConstantOverrideNegotiatingResourceReader<R, S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        ConstantOverrideNegotiatingResourceReader {
            inner,
            config: self.config.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R, S> DerivedContentNegotiationLayer<R, BinaryRepresentation, S>
    for ConstantOverrideNegotiationLayer<R, S>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    S: FlexibleResourceReader<R, R::Representation>,
{
    type Config = Option<ConstantOverrideNegotiationConfig>;

    fn new(config: Arc<Self::Config>) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

/// A layered resource reader that connegs between configured
/// constant representation and inner representation.
///
#[derive(Debug)]
pub struct ConstantOverrideNegotiatingResourceReader<R, Inner> {
    inner: Inner,
    config: Arc<Option<ConstantOverrideNegotiationConfig>>,
    _phantom: PhantomData<fn(R)>,
}

impl<R, Inner: Clone> Clone for ConstantOverrideNegotiatingResourceReader<R, Inner> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            config: self.config.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R, Inner> Service<ResourceReadRequest<R>>
    for ConstantOverrideNegotiatingResourceReader<R, Inner>
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

    #[tracing::instrument(skip_all, name = "ConstantOverrideNegotiatingResourceReader::call")]
    fn call(&mut self, req: ResourceReadRequest<R>) -> Self::Future {
        let config = if let Some(c) = self.config.as_ref() {
            c.clone()
        } else {
            // If config is `None` pass on inner response.
            let inner_fut = self.inner.call(req);
            return Box::pin(async move { Ok(inner_fut.await?.map_representation(Into::into)) });
        };

        // Create derived-content negotiator.
        let dconneger = ConstantOverridingContentTypeNegotiator {
            accept: req.rep_conneg_params.accept.clone(),
            config: config.clone(),
        };

        // Create request to inner service
        let mut inner_req = req;

        // Pass on new range negotiator, that takes dconneg into account.
        inner_req.rep_preferences.non_container_rep_range_negotiator =
            Box::new(DConnegLayeredRangeNegotiator {
                outer: inner_req.rep_preferences.non_container_rep_range_negotiator,
                dconneger: dconneger.clone(),
            });

        // Create inner future by calling inner service.
        let inner_fut = self.inner.call(inner_req);

        Box::pin(async move {
            // Get inner response.
            let inner_response: ResourceReadResponse<R, _> = inner_fut
                .inspect_ok(|_| info!("Success in calling inner service."))
                .await
                .map_err(|e| {
                    error!("Error in calling inner service. Error:\n {}", e);
                    e
                })?
                .map_representation(Into::into);

            let inner_rep = inner_response.state.representation();

            // Get content-type of inner rep.
            let inner_rep_content_type = inner_rep.metadata().content_type();

            // Resolve derived rep content type.
            let dcontent_type = dconneger.resolve_pref_derived_content_type(inner_rep_content_type);

            debug!(
                "Inner rep content type: {}, Resolved dcontent type: {}",
                inner_rep_content_type, dcontent_type
            );

            // If inner rep content-type is same as resolved derived content-type,
            // then pass on inner state.
            if inner_rep_content_type == &dcontent_type {
                info!("Inner representation satisfies conneg resolved preferences.");
                return Ok(inner_response);
            }

            // Now, we know that dcontent-type is different from that of inner.
            // Return resource state with overriden rep.
            let overriding_rep_metadata = RepresentationMetadata::default()
                // Last-Modified same as that of source rep.
                .with_opt::<KLastModified>(inner_rep.metadata().get_rv::<KLastModified>().copied())
                // Derive new etag.
                .with_opt::<KDerivedETag>(
                    inner_rep
                        .metadata()
                        .get_rv::<KDerivedETag>()
                        .map(|etag| etag.derived_rep_etag("constant_overriding")),
                )
                // Content-Type to constant rep content-type.
                .with::<KContentType>(config.constant_rep_content_type.clone());

            // Construct overriding rep.
            let overriding_rep = BasicRepresentation {
                metadata: overriding_rep_metadata,
                data: config.constant_rep_complete_data.clone(),
                base_uri: inner_rep.base_uri().clone(),
            }
            .into_binary();

            // Return new response with overriden rep.
            Ok(inner_response.map_representation::<_, _>(|_| overriding_rep))
        })
    }
}

impl<R, Inner> FlexibleResourceReader<R, BinaryRepresentation>
    for ConstantOverrideNegotiatingResourceReader<R, Inner>
where
    R: Repo,
    R::Representation: Into<BinaryRepresentation>,
    Inner: FlexibleResourceReader<R, R::Representation>,
{
}

/// An implementation of [`DContentTypeNegotiator`], that
/// negotiates based on overrides preferences.
#[derive(Debug, Clone)]
pub struct ConstantOverridingContentTypeNegotiator {
    /// `Accept` header against which to negotiate.
    pub accept: Option<Accept>,

    /// Constant overriding conneg config.
    pub config: ConstantOverrideNegotiationConfig,
}

impl DContentTypeNegotiator for ConstantOverridingContentTypeNegotiator {
    fn resolve_pref_derived_content_type(self, src_content_type: &MediaType) -> MediaType {
        // Try to satisfy accept.
        if let Some(mut accept) = self.accept {
            accept.sort_accept_values_by_precedence();

            let overriding_preferences = &self.config.overriding_preferences;

            debug!(
                "accept: {:?}, overriding_preferences: {:?}",
                accept, overriding_preferences
            );

            if_chain! {
                // It only checks, against accept_value with highest precedence.
                if let Some(accept_val) = accept.accept_values.first();

                // If accept_val has exact match specificity.
                if let &MediaRangeSpecificity::EXACT { .. } = &accept_val.precedence().media_range_specificity;

                // If accept value weight is greater than configured minium.
               if accept_val.weight() >= &overriding_preferences.min_quality;

               // If constant rep content-type matches accept val.
               if accept_val.matches(self.config.constant_rep_content_type.deref(), false);

                // If content-type matches any of enabled media ranges,
                if overriding_preferences.enabled_src_media_ranges.
                    iter().
                    any(|mr| src_content_type.is_in_range(mr));

                // If content-type doesn't matches any of disabled media ranges,
                if !overriding_preferences.disabled_src_media_ranges.
                    iter().
                    any(|mr| src_content_type.is_in_range(mr));

                then {
                    return self.config.constant_rep_content_type;
                }
            }
        }

        src_content_type.clone()
    }
}

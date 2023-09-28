//! I provide an implementation of [`DirectRepPatcher`] that patches
//! binary rdf documents.
//!

use std::{collections::HashSet, marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture, ProbResult, Problem};
use futures::TryFutureExt;
use manas_http::{
    header::common::media_type::TEXT_TURTLE,
    representation::{
        impl_::{
            basic::BasicRepresentation,
            binary::BinaryRepresentation,
            common::data::{bytes_inmem::BytesInmem, quads_inmem::QuadsInmem},
        },
        metadata::{KContentType, RepresentationMetadata},
        Representation,
    },
};
use manas_repo::service::{
    patcher_resolver::impl_::UnsupportedRepPatcher,
    resource_operator::common::rep_patcher::{
        RepPatcher, INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE, INVALID_ENCODED_SOURCE_REP,
    },
};
use manas_space::{
    resource::{operation::SolidResourceOperation, state::SolidResourceState},
    SolidStorageSpace,
};
use rdf_dynsyn::{syntax::invariant::serializable::DynSynSerializableSyntax, DynSynFactorySet};
use rdf_utils::model::dataset::InfallibleMutableDataset;
use tower::{Service, ServiceExt};
use tracing::{error, warn};

use crate::patching::patcher::DirectRepPatcher;

/// An implementation of [`DirectRepPatcher`] that patches
/// rdf binary documents using inner dataset patcher.
#[derive(Debug)]
pub struct BinaryRdfDocPatcher<StSpace, Inner, D> {
    /// DynSyn Factories.
    dynsyn_factories: Arc<DynSynFactorySet>,

    inner: Inner,

    _phantom: PhantomData<fn(StSpace, D)>,
}

impl<StSpace, Inner: Clone, D> Clone for BinaryRdfDocPatcher<StSpace, Inner, D> {
    fn clone(&self) -> Self {
        Self {
            dynsyn_factories: self.dynsyn_factories.clone(),
            inner: self.inner.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<StSpace, Inner, D> From<BinaryRdfDocPatcher<StSpace, Inner, D>> for UnsupportedRepPatcher {
    fn from(_value: BinaryRdfDocPatcher<StSpace, Inner, D>) -> Self {
        warn!("Into unsupported patcher called.");
        unimplemented!()
    }
}

impl<StSpace, Inner, D> BinaryRdfDocPatcher<StSpace, Inner, D> {
    /// Create a new [`BinaryRdfDocPatcher`].
    #[inline]
    pub fn new(dynsyn_factories: Arc<DynSynFactorySet>, inner: Inner) -> Self {
        Self {
            dynsyn_factories,
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<StSpace, Inner, D> RepPatcher for BinaryRdfDocPatcher<StSpace, Inner, D>
where
    StSpace: SolidStorageSpace,
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>>,
    D: InfallibleMutableDataset + Default + Send + 'static,
{
    fn effective_ops(&self) -> HashSet<SolidResourceOperation> {
        self.inner.effective_ops()
    }
}

impl<StSpace, Inner, D> BinaryRdfDocPatcher<StSpace, Inner, D>
where
    StSpace: SolidStorageSpace,
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>> + Clone,
    D: InfallibleMutableDataset + Default + Send + 'static,
{
    async fn _call(
        res_state: SolidResourceState<StSpace, BinaryRepresentation<BytesInmem>>,
        dynsyn_factories: Arc<DynSynFactorySet>,
        mut inner: Inner,
    ) -> ProbResult<BinaryRepresentation<BytesInmem>> {
        let rep = res_state
            .representation
            .unwrap_or_else(|| Self::void_ttl_rep());

        // Get serializable syntax.
        let serializable_syntax = rep
            .metadata()
            .rdf_syntax::<DynSynSerializableSyntax>()
            .ok_or_else(|| {
                error!("Patch base rep content type is not serializable from quads.");
                INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE.new_problem()
            })?
            .value;

        // Parse quads.
        let quads_inmem = rep
            .try_parse_quads::<D>(dynsyn_factories.parser.clone())
            .await
            .ok_or_else(|| {
                error!("Patch base rep content type is not quads parsable.");
                INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE.new_problem()
            })?
            .map_err(|e| {
                error!("Error in parsing quads from target rep. Error:\n {}", e);
                INVALID_ENCODED_SOURCE_REP.new_problem()
            })?;

        // Apply inner patcher.
        let patched_quads_inmem = inner
            .ready()
            .and_then(|svc| {
                svc.call(SolidResourceState {
                    slot: res_state.slot,
                    representation: Some(BasicRepresentation {
                        data: quads_inmem,
                        metadata: Default::default(),
                        base_uri: rep.base_uri().clone(),
                    }),
                })
            })
            .inspect_err(|_| error!("Error in calling inner service."))
            .await?
            .data;

        // Serialize resultant quads.
        BinaryRepresentation::try_from_wrap_serializing_quads(
            patched_quads_inmem,
            dynsyn_factories.serializer.clone(),
            serializable_syntax,
        )
        .await
        .map_err(|e| {
            error!("Error in serializing the patched rep.");
            UNKNOWN_IO_ERROR
                .new_problem_builder()
                .message("Error in serializing the patched rep.")
                .source(e)
                .finish()
        })
    }

    fn void_ttl_rep<Dt>() -> BinaryRepresentation<Dt>
    where
        BasicRepresentation<Dt>: Into<BinaryRepresentation<Dt>>,
        Dt: Default,
    {
        BasicRepresentation {
            data: Default::default(),
            metadata: RepresentationMetadata::new().with::<KContentType>((TEXT_TURTLE).clone()),
            base_uri: None,
        }
        .into()
    }
}

impl<StSpace, Inner, D> Service<SolidResourceState<StSpace, BinaryRepresentation<BytesInmem>>>
    for BinaryRdfDocPatcher<StSpace, Inner, D>
where
    StSpace: SolidStorageSpace,
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>> + Clone,
    D: InfallibleMutableDataset + Default + Send + 'static,
{
    type Response = BinaryRepresentation<BytesInmem>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        res_state: SolidResourceState<StSpace, BinaryRepresentation<BytesInmem>>,
    ) -> Self::Future {
        let dynsyn_factories = self.dynsyn_factories.clone();
        let inner_svc = self.inner.clone();
        Box::pin(Self::_call(res_state, dynsyn_factories, inner_svc))
    }
}

impl<StSpace, Inner, D> DirectRepPatcher<StSpace, BinaryRepresentation<BytesInmem>>
    for BinaryRdfDocPatcher<StSpace, Inner, D>
where
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>> + Clone,
    D: InfallibleMutableDataset + Default + Send + 'static,
    StSpace: SolidStorageSpace,
{
    type ResolutionConfig = BinaryRdfDocPatcherResolutionConfig<Inner::ResolutionConfig>;

    fn try_resolve(
        patch_doc_rep: BinaryRepresentation,
        config: Arc<Self::ResolutionConfig>,
    ) -> ProbFuture<'static, Self> {
        Box::pin(async move {
            Ok(Self::new(
                config.dynsyn_factories.clone(),
                Inner::try_resolve(patch_doc_rep, config.inner.clone()).await?,
            ))
        })
    }
}

impl<StSpace, Inner, D> Service<SolidResourceState<StSpace, BinaryRepresentation>>
    for BinaryRdfDocPatcher<StSpace, Inner, D>
where
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>> + Clone,
    D: InfallibleMutableDataset + Default + Send + 'static,
    StSpace: SolidStorageSpace,
{
    type Response = BinaryRepresentation;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        res_state: SolidResourceState<StSpace, BinaryRepresentation>,
    ) -> Self::Future {
        let dynsyn_factories = self.dynsyn_factories.clone();
        let inner_svc = self.inner.clone();

        Box::pin(async move {
            // Convert into inmemory rep.
            // TODO size capping here?
            let rep_inmem: Option<BinaryRepresentation<BytesInmem>> =
                if let Some(rep) = res_state.representation {
                    Some(async_convert::TryFrom::try_from(rep).await.map_err(|e| {
                        error!("Error in converting patch source rep into inmem rep. {e}");
                        UNKNOWN_IO_ERROR
                            .new_problem_builder()
                            .source_in_a_box(e)
                            .finish()
                    })?)
                } else {
                    None
                };

            Ok(Self::_call(
                SolidResourceState {
                    representation: rep_inmem,
                    slot: res_state.slot,
                },
                dynsyn_factories,
                inner_svc,
            )
            .await?
            .into())
        })
    }
}

impl<StSpace, Inner, D> DirectRepPatcher<StSpace, BinaryRepresentation>
    for BinaryRdfDocPatcher<StSpace, Inner, D>
where
    Inner: DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>> + Clone,
    D: InfallibleMutableDataset + Default + Send + 'static,
    StSpace: SolidStorageSpace,
{
    type ResolutionConfig = BinaryRdfDocPatcherResolutionConfig<Inner::ResolutionConfig>;

    fn try_resolve(
        patch_doc_rep: BinaryRepresentation,
        config: Arc<Self::ResolutionConfig>,
    ) -> ProbFuture<'static, Self> {
        Box::pin(async move {
            Ok(Self::new(
                config.dynsyn_factories.clone(),
                Inner::try_resolve(patch_doc_rep, config.inner.clone()).await?,
            ))
        })
    }
}

/// Resolution config for [`BinaryRdfDocPatcher`].
#[derive(Debug, Clone)]
pub struct BinaryRdfDocPatcherResolutionConfig<Inner> {
    /// Dynsyn factories.
    pub dynsyn_factories: Arc<DynSynFactorySet>,

    /// Inner config.
    pub inner: Arc<Inner>,
}

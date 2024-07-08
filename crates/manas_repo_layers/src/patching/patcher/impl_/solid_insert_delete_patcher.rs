//! I define an implementation of [`DirectRepPatcher`]
//! confirming to solid's insert-delete n3 patch.
//!

use std::{collections::HashSet, io::BufReader, marker::PhantomData, sync::Arc, task::Poll};

use capped_stream::OutOfSizeLimitError;
use dyn_problem::{
    type_::{INTERNAL_ERROR, UNKNOWN_IO_ERROR},
    ProbFuture, Problem,
};
use manas_http::{
    representation::{
        impl_::{
            basic::BasicRepresentation,
            binary::BinaryRepresentation,
            common::data::{bytes_inmem::BytesInmem, quads_inmem::QuadsInmem},
        },
        Representation,
    },
    BoxError,
};
use manas_repo::service::{
    patcher_resolver::{
        impl_::UnsupportedRepPatcher, INVALID_ENCODED_PATCH, UNKNOWN_PATCH_DOC_CONTENT_TYPE,
    },
    resource_operator::common::{
        problem::PAYLOAD_TOO_LARGE,
        rep_patcher::{RepPatcher, PATCH_SEMANTICS_ERROR},
    },
};
use manas_space::{
    resource::{operation::SolidResourceOperation, state::SolidResourceState},
    SolidStorageSpace,
};
use rdf_dynsyn::parser::DynSynParserFactorySet;
use rdf_utils::{
    model::{dataset::InfallibleMutableDataset, quad::ArcQuad},
    patch::{
        solid_insert_delete::{SolidInsertDeletePatchDoc, TEXT_N3},
        PatchEffectiveOperation,
    },
};
use sophia_api::dataset::SetDataset;
use tokio::task::spawn_blocking;
use tower::Service;
use tracing::{error, warn};

use crate::patching::patcher::DirectRepPatcher;

/// An implementation of [`RepPatcher`] svc that patches
/// an rdf source representation using configured solid
/// insert-delete patch doc.
///
pub struct SolidInsertDeletePatcher<StSpace, SD> {
    /// Patch doc.
    // TODO Should change to indexed dataset.
    pub patch_doc: SolidInsertDeletePatchDoc<HashSet<ArcQuad>>,

    _phantom: PhantomData<fn(StSpace, SD)>,
}

impl<StSpace, SD> std::fmt::Debug for SolidInsertDeletePatcher<StSpace, SD> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolidInsertDeletePatcher")
            .field("patch_doc", &self.patch_doc)
            .finish()
    }
}

impl<StSpace, SD> Clone for SolidInsertDeletePatcher<StSpace, SD> {
    fn clone(&self) -> Self {
        Self {
            patch_doc: self.patch_doc.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<StSpace, SD> From<SolidInsertDeletePatcher<StSpace, SD>> for UnsupportedRepPatcher {
    fn from(_value: SolidInsertDeletePatcher<StSpace, SD>) -> Self {
        warn!("Into unsupported patcher called.");
        unimplemented!()
    }
}

impl<StSpace, SD: InfallibleMutableDataset + SetDataset + Send + 'static>
    SolidInsertDeletePatcher<StSpace, SD>
{
    /// Create a new [`SolidInsertDeletePatcher`] with given patch doc.
    #[inline]
    pub fn new(patch_doc: SolidInsertDeletePatchDoc<HashSet<ArcQuad>>) -> Self {
        Self {
            patch_doc,
            _phantom: PhantomData,
        }
    }
}

impl<StSpace, D> RepPatcher for SolidInsertDeletePatcher<StSpace, D>
where
    StSpace: SolidStorageSpace,
    D: Send + 'static,
{
    #[inline]
    fn effective_ops(&self) -> HashSet<SolidResourceOperation> {
        self.patch_doc
            .patch()
            .effective_ops()
            .iter()
            .map(|op| match op {
                PatchEffectiveOperation::Read => SolidResourceOperation::READ,
                PatchEffectiveOperation::Append => SolidResourceOperation::APPEND,
                PatchEffectiveOperation::Write => SolidResourceOperation::WRITE,
            })
            .collect()
    }
}

impl<StSpace, D> Service<SolidResourceState<StSpace, BasicRepresentation<QuadsInmem<D>>>>
    for SolidInsertDeletePatcher<StSpace, D>
where
    StSpace: SolidStorageSpace,
    D: Default + InfallibleMutableDataset + SetDataset + Send + 'static,
{
    type Response = BasicRepresentation<QuadsInmem<D>>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        res_state: SolidResourceState<StSpace, BasicRepresentation<QuadsInmem<D>>>,
    ) -> Self::Future {
        let mut rep = res_state.representation.unwrap_or_default();

        let patch = self.patch_doc.patch().clone();

        Box::pin(async move {
            // Apply patch.
            let (patched_dataset, _applied_patch) =
                patch.apply(rep.data.into_inner()).map_err(|e| {
                    error!("Error in patching target rep dataset. Error:\n {}", e);
                    PATCH_SEMANTICS_ERROR
                        .new_problem_builder()
                        .source(e)
                        .finish()
                })?;

            // TODO should validate applied_patch here?
            // Or leave validation of complete patched rep to higher layers? (As of now.)

            rep.data = QuadsInmem::new(patched_dataset);
            Ok(rep)
        })
    }
}

impl<StSpace, D> DirectRepPatcher<StSpace, BasicRepresentation<QuadsInmem<D>>>
    for SolidInsertDeletePatcher<StSpace, D>
where
    StSpace: SolidStorageSpace,
    D: Default + InfallibleMutableDataset + SetDataset + Send + 'static,
{
    type ResolutionConfig = SolidInsertDeletePatcherResolutionConfig;

    fn try_resolve(
        mut patch_doc_rep: BinaryRepresentation,
        config: Arc<Self::ResolutionConfig>,
    ) -> ProbFuture<'static, Self> {
        Box::pin(async move {
            // If it is not n3 document, return error.
            if patch_doc_rep.metadata().content_type().essence_str() != TEXT_N3.essence_str() {
                error!("Patch doc content type is not that of n3.");
                return Err(UNKNOWN_PATCH_DOC_CONTENT_TYPE.new_problem());
            }

            // Size cap the patch doc rep.
            if let Some(max_patch_doc_payload_size) = config.max_patch_doc_payload_size {
                patch_doc_rep = patch_doc_rep.into_stream_size_capped(max_patch_doc_payload_size);
            }

            // Convert into inmem rep.
            let patch_rep_inmem: BinaryRepresentation<BytesInmem> =
                async_convert::TryFrom::try_from(patch_doc_rep)
                    .await
                    .map_err(|e: BoxError| {
                        error!("Error in loading patch body into memory. Error:\n {}", e);
                        if e.downcast_ref::<OutOfSizeLimitError>().is_some() {
                            PAYLOAD_TOO_LARGE
                                .new_problem_builder()
                                .message("Patch payload too large.")
                                .finish()
                        } else {
                            UNKNOWN_IO_ERROR
                                .new_problem_builder()
                                .message("Unknown io error in loading patch rep.")
                                .source_in_a_box(e)
                                .finish()
                        }
                    })?;

            // Parse [`SolidInsertDeletePatchDoc`].
            let patch_doc = spawn_blocking(move || {
                SolidInsertDeletePatchDoc::<HashSet<ArcQuad>>::parse(
                    BufReader::new(patch_rep_inmem.data().as_read()),
                    patch_rep_inmem.base_uri().as_ref().map(Into::into),
                )
            })
            .await
            .map_err(|e| {
                error!("Error in spawned parse worker. Error:\n {}", e);
                INTERNAL_ERROR.new_problem()
            })?
            .map_err(|e| {
                error!("Invalid encoded patch doc. Error:\n {}", e);
                INVALID_ENCODED_PATCH
                    .new_problem_builder()
                    .source(e)
                    .finish()
            })?;

            Ok(SolidInsertDeletePatcher::new(patch_doc))
        })
    }
}

/// Resolution config for [`SolidInsertDeletePatcher`].
#[derive(Debug, Clone)]
pub struct SolidInsertDeletePatcherResolutionConfig {
    /// Dynsyn parser factories.
    pub dynsyn_parser_factories: Arc<DynSynParserFactorySet>,

    /// Maximum patch doc payload size.
    pub max_patch_doc_payload_size: Option<u64>,
}

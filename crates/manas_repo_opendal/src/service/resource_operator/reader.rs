//! I provide an implementation of [`ResourceReader``] for ODR.
//!

use std::{marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::{
    define_anon_problem_types, type_::UNKNOWN_IO_ERROR, ProbFuture, Problem, ProblemBuilderExt,
};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::service::resource_operator::{
    common::{
        preconditions::{KEvaluatedRepValidators, KPreconditionsEvalResult},
        problem::{PRECONDITIONS_NOT_SATISFIED, UNSUPPORTED_OPERATION},
        status_token::RepoResourceStatusTokenBase,
    },
    reader::{
        FlexibleResourceReader, ResourceReadRequest, ResourceReadResponse, ResourceReadTokenSet,
        ResourceReader,
    },
};
use tower::Service;
use tracing::error;

use super::common::status_token::variant::ODRResourceStateResolutionError;
use crate::{context::ODRContext, setup::ODRSetup, OpendalRepo};

/// An implementation of [`ResourceReader``] for ODR.
#[derive(Debug, Clone)]
pub struct ODRResourceReader<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup> Default for ODRResourceReader<Setup> {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<Setup: ODRSetup> Service<ResourceReadRequest<OpendalRepo<Setup>>>
    for ODRResourceReader<Setup>
{
    type Response = ResourceReadResponse<OpendalRepo<Setup>, BinaryRepresentation>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[tracing::instrument(skip_all, name = "ODRResourceReader::call", fields(req))]
    fn call(&mut self, req: ResourceReadRequest<OpendalRepo<Setup>>) -> Self::Future {
        Box::pin(async move {
            let er_token = req.tokens.res_token;

            // Ensure backend has required capabilities.
            Self::ensure_backend_caps(er_token.repo_context())?;

            // Evaluate preconditions.
            let rep_validators = er_token.resolve_rep_validators();

            let pc_eval_result = req.preconditions.evaluate(Some(&rep_validators));

            // Return error if preconditions are not satisfied.
            if !pc_eval_result.are_satisfied() {
                return Err(PRECONDITIONS_NOT_SATISFIED
                    .new_problem_builder()
                    .extend_with::<KPreconditionsEvalResult>(pc_eval_result)
                    .extend_with::<KEvaluatedRepValidators>(Some(rep_validators))
                    .finish());
            }

            // Resolve resource state.
            let state = er_token
                .try_resolve_resource_state(req.rep_preferences)
                .await
                .map_err(Self::map_state_resolution_err)?;

            Ok(ResourceReadResponse {
                state,
                aux_links_index: er_token.res_context.supported_aux_links().collect(),
                tokens: ResourceReadTokenSet::new(er_token),
                extensions: Default::default(),
            })
        })
    }
}

impl<Setup: ODRSetup> ODRResourceReader<Setup> {
    /// Ensure required backend capabilities.
    #[allow(clippy::result_large_err)]
    pub(crate) fn ensure_backend_caps(
        repo_context: &Arc<ODRContext<Setup>>,
    ) -> Result<(), Problem> {
        // Ensure backend has required capabilities.
        let backend_caps = repo_context.backend_caps();

        if !(backend_caps.stat && backend_caps.read && backend_caps.list) {
            error!("ODR backend doesn't have required capabilities to support read operation.");
            return Err(UNSUPPORTED_OPERATION.new_problem());
        }

        Ok(())
    }

    /// Map state resolution error to problem.
    pub(crate) fn map_state_resolution_err(err: ODRResourceStateResolutionError) -> Problem {
        match err {
            ODRResourceStateResolutionError::EffectiveAltMetadataResolutionError(ie) => {
                EFFECTIVE_ALT_META_RESOLUTION_ERROR
                    .new_problem_builder()
                    .source(ie)
            }
            ODRResourceStateResolutionError::InvalidUserSuppliedContainerRep => {
                INVALID_USER_SUPPLIED_CONTAINER_REP.new_problem_builder()
            }
            ODRResourceStateResolutionError::UnknownIoError(ie) => {
                UNKNOWN_IO_ERROR.new_problem_builder().source(ie)
            }
        }
        .finish()
    }
}

impl<Setup: ODRSetup> FlexibleResourceReader<OpendalRepo<Setup>, BinaryRepresentation>
    for ODRResourceReader<Setup>
{
}

impl<Setup: ODRSetup> ResourceReader for ODRResourceReader<Setup> {
    type Repo = OpendalRepo<Setup>;
}

define_anon_problem_types!(
    /// Invalid associated alt metadata.
    INVALID_ASSOC_ALT_METADATA: ("Invalid associated alt metadata.");

    /// Effective alt metadata resolution error.
    EFFECTIVE_ALT_META_RESOLUTION_ERROR: ("Effective alt metadata resolution error.");

    /// Invalid user supplied container representation.
    INVALID_USER_SUPPLIED_CONTAINER_REP: ("Invalid user supplied container representation.");
);

//! I provide an implementation of [`RepUpdateValidator`] that
//! protects aux resource semantics.
//!

use std::{marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::Repo;
use tower::Service;

use super::common::RdfSourceRepUpdateValidatorConfig;
use crate::validating::update_validator::{update_context::RepUpdateContext, RepUpdateValidator};

/// An implementation of [`RepUpdateValidator`] that
/// protects auxiliary resource semantics.
///
/// In particular, it ensures that representation is a valid rdf
/// source representation for configured aux rel types.
pub struct AuxProtectingRepUpdateValidator<R, Rep> {
    /// Config.
    config: Arc<RdfSourceRepUpdateValidatorConfig>,
    _phantom: PhantomData<fn(R, Rep)>,
}

impl<R, Rep> Clone for AuxProtectingRepUpdateValidator<R, Rep> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<R, Rep> std::fmt::Debug for AuxProtectingRepUpdateValidator<R, Rep> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuxProtectingRepUpdateValidator")
            .field("config", &self.config)
            .finish()
    }
}

impl<R> Service<RepUpdateContext<R>> for AuxProtectingRepUpdateValidator<R, BinaryRepresentation>
where
    R: Repo<Representation = BinaryRepresentation>,
{
    type Response = RepUpdateContext<R>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, update_context: RepUpdateContext<R>) -> Self::Future {
        // If resource is not rdf source aux, then skip validation.
        if !update_context.res_slot.is_rdf_source_aux_res_slot() {
            return Box::pin(async { Ok(update_context) });
        }

        let parse_fut = update_context.try_resolve_effective_rep_quads(
            self.config.dynsyn_parser_factories.clone(),
            self.config.max_user_supplied_rep_size,
        );
        Box::pin(async move {
            // Ensure representation is quads parsable.
            let (update_context, _) = parse_fut.await?;
            Ok(update_context)
        })
    }
}

impl<R> RepUpdateValidator<R> for AuxProtectingRepUpdateValidator<R, BinaryRepresentation>
where
    R: Repo<Representation = BinaryRepresentation>,
{
    type Config = RdfSourceRepUpdateValidatorConfig;

    fn new(config: Arc<Self::Config>) -> Self {
        AuxProtectingRepUpdateValidator {
            config,
            _phantom: PhantomData,
        }
    }
}

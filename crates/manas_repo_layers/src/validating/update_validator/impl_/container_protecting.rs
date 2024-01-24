//! I provide an implementation of [`RepUpdateValidator`] that
//! protects container semantics.
//!

use std::{marker::PhantomData, ops::Deref, sync::Arc, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use manas_http::{
    representation::{
        impl_::{binary::BinaryRepresentation, common::data::quads_inmem::EcoQuadsInmem},
        Representation,
    },
    uri::{invariant::AbsoluteHttpUri, predicate::is_normal::Normalization},
};
use manas_repo::{
    context::RepoContextual,
    policy::uri::RepoUriPolicy,
    service::resource_operator::common::problem::{
        INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA, INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES,
    },
    Repo,
};
use manas_space::{
    resource::{
        kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
        slot_path::SolidResourceSlotPath, slot_rel_type::SlotRelationType,
        slot_rev_link::SlotRevLink,
    },
    RelativeSolidStorageSpace,
};
use rdf_utils::model::term::ArcTerm;
use rdf_vocabularies::ns;
use sophia_api::{
    quad::Quad,
    term::{Term, TermKind},
};
use tower::Service;
use tracing::error;

use super::common::RdfSourceRepUpdateValidatorConfig;
use crate::validating::update_validator::{update_context::RepUpdateContext, RepUpdateValidator};

/// An implementation of [`RepUpdateValidator`] that
/// protects container semantics.
///
/// It ensures that user supplied representation doesn't contain
/// containment statements or contained resources metadata
/// statements.
pub struct ContainerProtectingRepUpdateValidator<R, Rep> {
    /// Config.
    config: Arc<RdfSourceRepUpdateValidatorConfig>,
    _phantom: PhantomData<fn(R, Rep)>,
}

impl<R, Rep> Clone for ContainerProtectingRepUpdateValidator<R, Rep> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<R, Rep> std::fmt::Debug for ContainerProtectingRepUpdateValidator<R, Rep> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerProtectingRepUpdateValidator")
            .field("config", &self.config)
            .finish()
    }
}

impl<R, Rep> ContainerProtectingRepUpdateValidator<R, Rep>
where
    R: Repo<Representation = Rep>,
    Rep: Representation,
{
    /// Validate quads of user supplied container rep.
    /// NOTE: representation should include only user supplied quads.
    #[allow(clippy::result_large_err)]
    fn validate_container_rep_data(
        container_slot: &SolidResourceSlot<R::StSpace>,
        rep_data: EcoQuadsInmem,
        repo_context: Arc<R::Context>,
    ) -> Result<(), Problem> {
        let repo_uri_policy = R::UriPolicy::new_with_context(repo_context);

        let container_relative_space = Arc::new(RelativeSolidStorageSpace {
            base_res_slot_id: container_slot.id().clone(),
        });

        let container_term = container_slot.id().uri.deref().into_term::<ArcTerm>();

        let rep_quads = rep_data.into_inner().0;

        for quad in rep_quads.as_slice().iter() {
            // Ensure not a containment statement.
            // Req: Servers MUST NOT allow HTTP PUT or PATCH on
            // a container to update its containment triples; if
            // the server receives such a request, it MUST
            // respond with a 409 status code. [Source]
            if Term::eq(quad.s(), &container_term) && Term::eq(quad.p(), ns::ldp::contains) {
                error!("User supplied container representation contains containment statement");

                // TODO attach much user context.
                return Err(INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES.new_problem());
            }

            // Ensure not a contained resource metadata
            // statement.
            // Req: Servers MUST NOT allow HTTP POST, PUT and
            // PATCH to update a container’s resource metadata
            // statements; if the server receives such a
            // request, it MUST respond with a 409 status code.
            if_chain::if_chain! {
                // If predicate matches any of specified.
                if [ns::rdf::type_, ns::stat::size, ns::dcterms::modified, ns::stat::mtime,]
                    .iter()
                    .any(|p| Term::eq(quad.p(), p));

                // If subject term is an iri
                if quad.s().kind() == TermKind::Iri;

                // If subject uri is a valid http absolute uri,
                if let Ok(subject_uri) = AbsoluteHttpUri::try_new_from(quad.s().iri().expect("Checked").as_str());

                let normal_subject_uri = subject_uri.and_infer::<Normalization<_>>(Default::default());

                // If subject is not container itself.
                if normal_subject_uri != container_slot.id().uri;

                // If subject uri can be valid uri for a
                // contained resource.
                // TODO bring it to sanity.
                if SolidResourceKind::ALL.iter().any(|res_kind| repo_uri_policy.is_allowed_relative_slot_path(
                        &SolidResourceSlotPath::try_new([
                            SolidResourceSlot::root_slot(container_relative_space.clone()),
                            SolidResourceSlot::try_new(
                                SolidResourceSlotId::new(container_relative_space.clone(), normal_subject_uri.clone()),
                                *res_kind,
                                Some(SlotRevLink {
                                    target: container_slot.id().uri.clone(),
                                    rev_rel_type: SlotRelationType::Contains
                                })
                            ).expect("Must be valid.")
                        ].as_slice()).expect("Must be valid")
                    )
                );

                then {
                    error!("User supplied container representation contains contained resource metadata statement.");
                    return Err(INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA.new_problem());
                }
            }
        }

        Ok(())
    }
}

impl<R> Service<RepUpdateContext<R>>
    for ContainerProtectingRepUpdateValidator<R, BinaryRepresentation>
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
        // If resource is not a container, then skip validation.
        if !update_context.res_slot.is_container_slot() {
            return Box::pin(async { Ok(update_context) });
        }

        let parse_fut = update_context.try_resolve_effective_rep_quads(
            self.config.dynsyn_parser_factories.clone(),
            self.config.max_user_supplied_rep_size,
        );
        Box::pin(async move {
            // Parse quads.
            let (update_context, effective_rep_quads) = parse_fut.await?;

            // Validate.
            Self::validate_container_rep_data(
                &update_context.res_slot,
                effective_rep_quads,
                update_context.repo_context.clone(),
            )?;

            Ok(update_context)
        })
    }
}

impl<R> RepUpdateValidator<R> for ContainerProtectingRepUpdateValidator<R, BinaryRepresentation>
where
    R: Repo<Representation = BinaryRepresentation>,
{
    type Config = RdfSourceRepUpdateValidatorConfig;

    fn new(config: Arc<Self::Config>) -> Self {
        ContainerProtectingRepUpdateValidator {
            config,
            _phantom: PhantomData,
        }
    }
}

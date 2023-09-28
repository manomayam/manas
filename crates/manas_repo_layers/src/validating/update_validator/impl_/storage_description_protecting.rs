// //! I provide an implementation of [`RepUpdateValidator`] that
// //! protects storage description resource semantics.
// //!

// use std::{marker::PhantomData, ops::Deref, sync::Arc, task::Poll};

// use dyn_problem::{ProbFuture, Problem};
// use manas_http::{
//     representation::{
//         impl_::{binary::BinaryRepresentation, common::data::quads_inmem::EcoQuadsInmem},
//         Representation,
//     },
//     uri::{invariant::AbsoluteHttpUri, predicate::is_normal::Normalization},
// };
// use manas_space::{
//     resource::{
//         kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
//         slot_path::SolidResourceSlotPath, slot_rel_type::SlotRelationType,
//         slot_rev_link::SlotRevLink,
//     },
//     RelativeSolidStorageSpace,
// };
// use rdf_dynsyn::parser::DynSynParserFactorySet;
// use rdf_utils::model::term::ArcTerm;
// use rdf_vocabularies::ns;
// use sophia_api::{
//     quad::Quad,
//     term::{Term, TermKind},
// };
// use tower::Service;
// use tracing::error;

// use manas_repo::{
//     context::RepoContextual,
//     layered::validating::update_validator::{update_context::RepUpdateContext, RepUpdateValidator},
//     policy::uri::RepoUriPolicy,
//     service::resource_operator::common::problem::{
//         INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA, INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES,
//     },
//     Repo,
// };

// use super::common::RdfSourceRepUpdateValidatorConfig;

// /// An implementation of [`RepUpdateValidator`] that
// /// protects container semantics.
// ///
// /// It ensures storage root uri and owner id to be not tampered.
// pub struct StorageDescriptionProtectingRepUpdateValidator<R, Rep> {
//     /// Config.
//     config: Arc<RdfSourceRepUpdateValidatorConfig>,
//     _phantom: PhantomData<fn(R, Rep)>,
// }

// impl<R, Rep> Clone for StorageDescriptionProtectingRepUpdateValidator<R, Rep> {
//     fn clone(&self) -> Self {
//         Self {
//             config: self.config.clone(),
//             _phantom: self._phantom,
//         }
//     }
// }

// impl<R, Rep> std::fmt::Debug for StorageDescriptionProtectingRepUpdateValidator<R, Rep> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("StorageDescriptionProtectingRepUpdateValidator")
//             .field("config", &self.config)
//             .finish()
//     }
// }

// impl<R, Rep> StorageDescriptionProtectingRepUpdateValidator<R, Rep>
// where
//     R: Repo<Representation = Rep>,
//     Rep: Representation,
// {
//     /// Validate quads of user supplied storage description rep.
//     fn validate_storage_description_rep_data(
//         res_slot: &SolidResourceSlot<R::StSpace>,
//         rep_data: EcoQuadsInmem,
//     ) -> Result<(), Problem> {
//         todo!()
//         Ok(())
//     }
// }

// impl<R> Service<RepUpdateContext<R>>
//     for StorageDescriptionProtectingRepUpdateValidator<R, BinaryRepresentation>
// where
//     R: Repo<Representation = BinaryRepresentation>,
// {
//     type Response = RepUpdateContext<R>;

//     type Error = Problem;

//     type Future = ProbFuture<'static, Self::Response>;

//     #[inline]
//     fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, update_context: RepUpdateContext<R>) -> Self::Future {
//         let parse_fut = update_context.try_resolve_effective_rep_quads(
//             self.config.dynsyn_parser_factories.clone(),
//             self.config.max_user_supplied_rep_size,
//         );
//         Box::pin(async move {
//             // Parse quads.
//             let (update_context, effective_rep_quads) = parse_fut.await?;

//             // Validate.
//             Self::validate_storage_description_rep_data(
//                 &&update_context.res_slot,
//                 effective_rep_quads,
//             )?;

//             Ok(update_context)
//         })
//     }
// }

// impl<R> RepUpdateValidator<R>
//     for StorageDescriptionProtectingRepUpdateValidator<R, BinaryRepresentation>
// where
//     R: Repo<Representation = BinaryRepresentation>,
// {
//     type Config = RdfSourceRepUpdateValidatorConfig;

//     fn new(config: Arc<Self::Config>) -> Self {
//         StorageDescriptionProtectingRepUpdateValidator {
//             config,
//             _phantom: PhantomData,
//         }
//     }
// }

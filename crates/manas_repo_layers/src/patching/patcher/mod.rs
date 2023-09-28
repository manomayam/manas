//! I provide trait and few implementations for direct rep patching.
//!

use std::{fmt::Debug, sync::Arc};

use dyn_problem::{ProbFuture, Problem};
use manas_http::representation::{impl_::binary::BinaryRepresentation, Representation};
use manas_repo::service::{
    patcher_resolver::impl_::UnsupportedRepPatcher,
    resource_operator::common::rep_patcher::RepPatcher,
};
use manas_space::{resource::state::SolidResourceState, SolidStorageSpace};
use tower::Service;

pub mod impl_;
pub mod util;

/// A trait for direct rep patchers. A direct rep patcher is a
/// [`RepPatcher`] that takes existing resource state and returns patched representation.
///
/// ### Errors:
///
/// An implementation must return following kinds of problems
/// for specified cases.
///
/// - [`INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE`](manas_repo::service::resource_operator::common::rep_patcher::INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE):
/// If target representation content-type is not compatible with
/// the patcher.
///
/// - [`INVALID_ENCODED_SOURCE_REP`](manas_repo::service::resource_operator::common::rep_patcher::INVALID_ENCODED_SOURCE_REP):
/// If target rep is invalid encoded.
///
/// - [`PATCH_SEMANTICS_ERROR`](manas_repo::service::resource_operator::common::rep_patcher::PATCH_SEMANTICS_ERROR):
/// If any semantic error in patch application.
pub trait DirectRepPatcher<StSpace, Rep>:
    RepPatcher
    + Service<
        SolidResourceState<StSpace, Rep>,
        Response = Rep,
        Error = Problem,
        Future = ProbFuture<'static, Rep>,
    > + Sized
    + Debug
    + Clone
    + Into<UnsupportedRepPatcher>
where
    StSpace: SolidStorageSpace,
    Rep: Representation + Send + 'static,
{
    /// Type of resolution config.
    type ResolutionConfig: Debug + Send + Sync + 'static;

    /// Try to resolve the patcher from given patch doc representation.
    fn try_resolve(
        patch_doc_rep: BinaryRepresentation,
        config: Arc<Self::ResolutionConfig>,
    ) -> ProbFuture<'static, Self>;
}

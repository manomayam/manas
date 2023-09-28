//! I define trait for resource updaters of a repo.
//!

mod message;

use std::fmt::Debug;

use dyn_problem::{ProbFuture, Problem};
pub use message::*;
use tower::Service;

use crate::Repo;

pub mod impl_;

/// A trait for resource updater of a repo.
///
/// ## Service contract:
///
/// - Service MUST update resource corresponding to given
/// params if valid.
///
/// - Service SHOULD mint auxiliary resources of updated
/// resource with out representations if not already.
///
/// ### Errors:
///
/// Service MUST return errors with following problem types in
/// specified cases.
///
/// - [`UNSUPPORTED_OPERATION`](super::common::problem::UNSUPPORTED_OPERATION):
/// If operation is not supported.
///
/// - [`PRECONDITIONS_NOT_SATISFIED`](super::common::problem::PRECONDITIONS_NOT_SATISFIED):
/// If preconditions not satisfied.
///
/// - [`UNSUPPORTED_MEDIA_TYPE`](super::common::problem::UNSUPPORTED_MEDIA_TYPE):
/// If representation content-type is not supported.
///
/// - [`INVALID_RDF_SOURCE_REPRESENTATION`](super::common::problem::INVALID_RDF_SOURCE_REPRESENTATION):
/// if resource to be updated is a container, and effective
/// supplied representation is not a valid rdf document.
///
/// - [`INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES`](super::common::problem::INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES):
/// If resource to be updated is a container, and effective
/// supplied rdf representation has any containment triples.
///
/// - [`INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA`](super::common::problem::INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA):
/// If resource to be updated is a container, and effective
/// supplied rdf representation has any contained resource
/// metadata triples.
///
/// - [`INVALID_RDF_SOURCE_REPRESENTATION`](super::common::problem::INVALID_RDF_SOURCE_REPRESENTATION):
/// If resource to be updated is an aux resource with aux rel
/// type that targets rdf sources, and effective supplied rdf
/// representation is not a valid rdf document.
///
/// - [`PATCH_SEMANTICS_ERROR`](super::common::rep_patcher::PATCH_SEMANTICS_ERROR):
/// If representation is being updated through patch, and if
/// there is a patch semantics error.
///
/// - [`INVALID_ENCODED_SOURCE_REP`](super::common::rep_patcher::INVALID_ENCODED_SOURCE_REP):
/// If representation is being updated through patch, and
/// existing representation is invalid encoded.
///
/// - [`INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE`](super::common::rep_patcher::INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE):
/// If representation is being updated through patch, and target
/// representation is not compatible with patch.
///
/// - [`PAYLOAD_TOO_LARGE`](super::common::problem::PAYLOAD_TOO_LARGE):
/// If representation data payload is too large.
///
pub trait ResourceUpdater:
    Default
    + Service<
        ResourceUpdateRequest<Self::Repo>,
        Response = ResourceUpdateResponse,
        Error = Problem,
        Future = ProbFuture<'static, ResourceUpdateResponse>,
    > + Send
    + Clone
    + Debug
    + 'static
{
    /// Type of the repo
    type Repo: Repo;
}

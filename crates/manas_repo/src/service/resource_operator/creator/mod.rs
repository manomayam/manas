//! I define trait for resource creators of a repo.
//!

mod message;

use std::fmt::Debug;

use dyn_problem::{define_anon_problem_types, ProbFuture, Problem};
pub use message::*;
use tower::Service;

use crate::Repo;

pub mod impl_;

/// A trait for resource creator of a repo.
///
/// ## Service contract:
///
/// - Service MUST create resource corresponding to given
/// params if valid, and return resource slot of the created
/// resource in response. It MUST NOT create any intermediate
/// containers.
///
/// - If created resource is a contained resource, MUST update
/// container's representation by adding corresponding
/// containment triple.
///
/// - Service SHOULD mint auxiliary resources of created
/// resource with out representations.
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
/// - [`URI_POLICY_VIOLATION`](super::common::problem::URI_POLICY_VIOLATION):
/// If there is any uri policy violation.
///
/// - [`UNSUPPORTED_MEDIA_TYPE`](super::common::problem::UNSUPPORTED_MEDIA_TYPE):
/// If representation content-type is not supported.
///
/// - [`INVALID_RDF_SOURCE_REPRESENTATION`](super::common::problem::INVALID_RDF_SOURCE_REPRESENTATION):
/// if resource to be created is a container, and effective
/// supplied representation is not a valid rdf document.
///
/// - [`INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES`](super::common::problem::INVALID_USER_SUPPLIED_CONTAINMENT_TRIPLES):
/// If resource to be created is a container, and effective
/// supplied rdf representation has any containment triples.
///
/// - [`INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA`](super::common::problem::INVALID_USER_SUPPLIED_CONTAINED_RES_METADATA):
/// If resource to be created is a container, and effective
/// supplied rdf representation has any contained resource
/// metadata triples.
///
/// - [`INVALID_RDF_SOURCE_REPRESENTATION`](super::common::problem::INVALID_RDF_SOURCE_REPRESENTATION):
/// If resource to be created is an aux resource with aux rel
/// type that targets rdf sources, and effective supplied rdf
/// representation is not a valid rdf document.
///
/// - [`PATCH_SEMANTICS_ERROR`](super::common::rep_patcher::PATCH_SEMANTICS_ERROR):
/// If representation is being created through patch, and if
/// there is a patch semantics error.
///
/// - [`PAYLOAD_TOO_LARGE`](super::common::problem::PAYLOAD_TOO_LARGE):
/// If representation data payload is too large.
///
/// - [`SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION`]:
/// If slot relation subject constrains are violated.
///
/// - [`SLOT_REL_TARGET_CONSTRAIN_VIOLATION`]:
/// If slot relation target constrains are violated.
///
pub trait ResourceCreator:
    Default
    + Service<
        ResourceCreateRequest<Self::Repo>,
        Response = ResourceCreateResponse<Self::Repo>,
        Error = Problem,
        Future = ProbFuture<'static, ResourceCreateResponse<Self::Repo>>,
    > + Send
    + Clone
    + Debug
    + 'static
{
    /// Type of the repo
    type Repo: Repo;
}

define_anon_problem_types!(
    /// slot relation subject constrains violation.
    SLOT_REL_SUBJECT_CONSTRAIN_VIOLATION: (
        "slot relation subject constrains violation."
    );

    /// slot relation target constrains violation.
    SLOT_REL_TARGET_CONSTRAIN_VIOLATION: (
        "slot relation target constrains violation."
    );
);

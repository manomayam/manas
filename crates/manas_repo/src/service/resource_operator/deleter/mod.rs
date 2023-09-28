//! I define trait for resource deleter of a repo.
//!

mod message;

use std::fmt::Debug;

use dyn_problem::{define_anon_problem_types, ProbFuture, Problem};
pub use message::*;
use tower::Service;

use crate::Repo;

pub mod impl_;

/// A trait for resource deleter of a repo.
///
/// ## Service contract:
///
/// - MUST delete target resource with given uri, and it;s
/// entire aux tree.
///
/// - If deleted resource is a contained resource, MUST update
/// container representation by removing corresponding
/// containment triple.
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
/// - [`DELETE_TARGETS_STORAGE_ROOT`]:
/// If target resource is a storage root or it's acl.
///
/// - [DELETE_TARGETS_NON_EMPTY_CONTAINER`]:
/// If target resource is a container and is non empty.
///
pub trait ResourceDeleter:
    Default
    + Service<
        ResourceDeleteRequest<Self::Repo>,
        Response = ResourceDeleteResponse<Self::Repo>,
        Error = Problem,
        Future = ProbFuture<'static, ResourceDeleteResponse<Self::Repo>>,
    > + Send
    + Clone
    + Debug
    + 'static
{
    /// Type of the repo
    type Repo: Repo;
}

define_anon_problem_types!(
    /// Delete targets storage root.
    DELETE_TARGETS_STORAGE_ROOT: ("Delete targets storage root.");

    /// Delete targets non empty container.
    DELETE_TARGETS_NON_EMPTY_CONTAINER: ("Delete targets non empty container.");
);

//! I define trait for resource readers of a repo.
//!

use std::fmt::Debug;

use dyn_problem::{define_anon_problem_types, ProbFuture, Problem};
use manas_http::representation::Representation;
use tower::Service;

pub use self::message::*;
use crate::Repo;

pub mod impl_;
mod message;

/// A trait for resource readers of a repo with flexibility
/// on rep data type.
///
/// ## Operation contract:
///
/// - Service must return [`ResourceReadResponse`]
/// corresponding to resource uri.
///
/// - When non-container rep-range-negotiator requests for
/// complete representation, service MUST return complete
/// representation.
///
/// - When non-container rep-range-negotiator requests for
/// partial representation, service SHOULD return partial
/// representation.
///
/// ### Errors:
///
/// Service MUST return errors with following problem types in
/// specified cases.
///
/// - [`UNSUPPORTED_OPERATION`](super::common::problem::UNSUPPORTED_OPERATION): If
/// operation is not supported.
///
/// - [`PRECONDITIONS_NOT_SATISFIED`](super::common::problem::PRECONDITIONS_NOT_SATISFIED): If
/// preconditions are not satisfied.
///
/// - [`RANGE_NOT_SATISFIABLE`]:
/// If service chose to honour requested rep-range preference,
/// and requested range is not satisfiable.
///
pub trait FlexibleResourceReader<R, Rep>:
    Service<
        ResourceReadRequest<R>,
        Response = ResourceReadResponse<R, Rep>,
        Error = Problem,
        Future = ProbFuture<'static, ResourceReadResponse<R, Rep>>,
    > + Send
    + Sized
    + Clone
    + Debug
    + 'static
where
    R: Repo,
    Rep: Representation + Send + 'static,
{
}

/// A trait for resource readers of a repo.
pub trait ResourceReader:
    FlexibleResourceReader<Self::Repo, <Self::Repo as Repo>::Representation> + Default
{
    /// Type of the repo
    type Repo: Repo;
}

define_anon_problem_types!(
    ///Range not satisfiable.
    RANGE_NOT_SATISFIABLE: ("Range not satisfiable.");
);

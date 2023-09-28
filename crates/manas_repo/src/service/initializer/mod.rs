use std::fmt::Debug;

use dyn_problem::{define_anon_problem_types, ProbFuture, Problem};
use tower::Service;

use crate::RepoContextual;

pub mod impl_;

/// A contract trait for repo initializer services.
///
/// An initializer must ensure that the storage root container
/// to have a representation after initialization. An
/// initializer must be idempotent. Calling it must not cause
/// any other state change if repo is already initialized. It
/// should return Ok(false), if it is already initialized.
///
/// ### Errors:
///
/// Service MUST return errors with following problem types in
/// specified cases.
///
/// - [`INVALID_STORAGE_ROOT_URI`]: If
/// storage root uri specified in context deemed invalid for any
/// policy reason.

pub trait RepoInitializer:
    RepoContextual
    + Service<(), Response = bool, Error = Problem, Future = ProbFuture<'static, bool>>
    + Debug
    + Send
    + 'static
{
}

define_anon_problem_types!(
    /// Invalid storage root uri.
    INVALID_STORAGE_ROOT_URI: ("Invalid storage root uri.");
);

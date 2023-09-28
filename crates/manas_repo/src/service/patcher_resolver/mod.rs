use std::fmt::Debug;

use dyn_problem::{define_anon_problem_types, ProbFuture, Problem};
use manas_http::representation::impl_::binary::BinaryRepresentation;
use tower::Service;

use crate::{Repo, RepoContextual};

pub mod impl_;

/// A [`RepPatcherResolver`] takes bytes representation of a
/// patch document, and resolves effective patcher. It can be
/// instantiated from repo context.
///
/// ### Errors:
///
/// Service MUST return errors of following kinds in specified
/// error cases.
///
/// - [`UNKNOWN_PATCH_DOC_CONTENT_TYPE`]: If patch doc
/// content-type is unknown to resolver.
///
/// - [`INVALID_ENCODED_PATCH`]: If patch doc encoding is invalid.
pub trait RepPatcherResolver:
    RepoContextual
    + Service<
        BinaryRepresentation,
        Response = <Self::Repo as Repo>::RepPatcher,
        Error = Problem,
        Future = ProbFuture<'static, <Self::Repo as Repo>::RepPatcher>,
    >
    + Send
    + Sync
    + 'static
    + Clone
    + Debug
{
}

define_anon_problem_types!(
    /// Unknown patch doc content type.
    UNKNOWN_PATCH_DOC_CONTENT_TYPE: ("Unknown patch doc content type.");

    /// Invalid encoded patch.
    INVALID_ENCODED_PATCH: ("Invalid encoded patch.");
);

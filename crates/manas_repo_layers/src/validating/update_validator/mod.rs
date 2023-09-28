//! I define trait and few implementations of representation
//! update validators.
//!

use std::{fmt::Debug, sync::Arc};

use dyn_problem::{ProbFuture, Problem};
use manas_repo::Repo;
use tower::Service;

use self::update_context::RepUpdateContext;

pub mod update_context;

pub mod impl_;

/// A trait for representation update validator.
pub trait RepUpdateValidator<R: Repo>:
    Debug
    + Send
    + 'static
    + Service<
        RepUpdateContext<R>,
        Response = RepUpdateContext<R>,
        Error = Problem,
        Future = ProbFuture<'static, RepUpdateContext<R>>,
    >
{
    /// Type of validator config.
    type Config: Debug + Send + Sync + 'static;

    /// Create a new validator with given config.
    fn new(config: Arc<Self::Config>) -> Self;
}

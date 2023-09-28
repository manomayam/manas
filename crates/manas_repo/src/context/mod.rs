//! I define traits for repo context types.
//!

use std::{fmt::Debug, sync::Arc};

use crate::Repo;

pub mod impl_;

/// A trait for representing context associated with a repo.
pub trait RepoContext: Debug + Send + Sync + 'static {
    /// Type of the repo.
    type Repo: Repo;

    /// Get a shared reference to repo's storage space.
    fn storage_space(&self) -> &Arc<<Self::Repo as Repo>::StSpace>;
}

/// A trait for types, which can be instantiated with a repo context.
pub trait RepoContextual {
    /// Type of the repo.
    type Repo: Repo;

    /// Create new instance with given repo context.
    fn new_with_context(context: Arc<<Self::Repo as Repo>::Context>) -> Self;

    /// Get the repo context.
    fn repo_context(&self) -> &Arc<<Self::Repo as Repo>::Context>;
}

/// A trait for layered repo contexts.
pub trait LayeredRepoContext: RepoContext {
    /// Type of the inner repo.
    type InnerRepo: Repo<StSpace = <Self::Repo as Repo>::StSpace>;

    /// Get a shared ref to inner context.
    fn inner(&self) -> &Arc<<Self::InnerRepo as Repo>::Context>;
}

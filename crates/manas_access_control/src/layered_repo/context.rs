//! I define [`RepoContext`] for access controlled repo.
//!

use std::{fmt::Debug, sync::Arc};

use manas_repo::{
    context::{LayeredRepoContext, RepoContext},
    Repo,
};
use manas_space::resource::uri::SolidResourceUri;

use super::AccessControlledRepo;
use crate::model::pep::PolicyEnforcementPoint;

/// Type of initial root acr factory.
pub type InitialRootAcrRepFactory<IR> =
    dyn Fn(SolidResourceUri) -> Option<<IR as Repo>::Representation> + Send + Sync + 'static;

/// An implementation of [`RepoContext`] for
/// [`AccessControlledRepo`] repo implementation.
#[derive(Clone)]
pub struct AccessControlledRepoContext<IR: Repo, PEP> {
    /// IR context.
    pub inner: Arc<IR::Context>,

    /// Policy enforcement point.
    pub pep: Arc<PEP>,

    /// Initial storage root acr in ttl.
    pub initial_root_acr_rep_factory: Arc<InitialRootAcrRepFactory<IR>>,
}

impl<IR: Repo, PEP: Debug> Debug for AccessControlledRepoContext<IR, PEP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledRepoContext")
            .field("inner", &self.inner)
            .field("pep", &self.pep)
            .finish()
    }
}

impl<IR, PEP> RepoContext for AccessControlledRepoContext<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;

    #[inline]
    fn storage_space(&self) -> &Arc<IR::StSpace> {
        self.inner.storage_space()
    }
}

impl<IR: Repo, PEP> AccessControlledRepoContext<IR, PEP> {
    /// Create a new instance of [`AccessControlledRepoContext`].
    #[inline]
    pub fn new(
        inner: Arc<IR::Context>,
        pep: Arc<PEP>,
        initial_root_acr_rep_factory: Arc<InitialRootAcrRepFactory<IR>>,
    ) -> Self {
        Self {
            inner,
            pep,
            initial_root_acr_rep_factory,
        }
    }
}

impl<IR, PEP> LayeredRepoContext for AccessControlledRepoContext<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type InnerRepo = IR;

    #[inline]
    fn inner(&self) -> &Arc<IR::Context> {
        &self.inner
    }
}

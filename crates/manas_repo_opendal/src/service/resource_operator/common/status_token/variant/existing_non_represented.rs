//! I provide existing-non-represented resource status token
//! implementation for ODR.
//!

use std::{ops::Deref, sync::Arc};

use manas_repo::service::resource_operator::common::status_token::{
    ExistingNonRepresentedResourceToken, RepoResourceStatusTokenBase,
};
use manas_space::resource::slot::SolidResourceSlot;

use crate::{
    context::ODRContext,
    service::resource_operator::common::status_token::inputs::ODRResourceStatusTokenInputs,
    setup::ODRSetup, OpendalRepo,
};

/// A struct to represent existing-non-represented resource status token for odr.
#[derive(Debug, Clone)]
pub struct ODRExistingNonRepresentedResourceToken<Setup: ODRSetup>(
    pub(in super::super) ODRResourceStatusTokenInputs<Setup>,
);

impl<Setup: ODRSetup> Deref for ODRExistingNonRepresentedResourceToken<Setup> {
    type Target = ODRResourceStatusTokenInputs<Setup>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Setup: ODRSetup> From<ODRExistingNonRepresentedResourceToken<Setup>>
    for ODRResourceStatusTokenInputs<Setup>
{
    #[inline]
    fn from(value: ODRExistingNonRepresentedResourceToken<Setup>) -> Self {
        value.0
    }
}

impl<Setup: ODRSetup> RepoResourceStatusTokenBase
    for ODRExistingNonRepresentedResourceToken<Setup>
{
    type Repo = OpendalRepo<Setup>;

    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        self.0.res_context.repo_context()
    }
}

impl<Setup: ODRSetup> ExistingNonRepresentedResourceToken
    for ODRExistingNonRepresentedResourceToken<Setup>
{
    #[inline]
    fn slot(&self) -> &SolidResourceSlot<Setup::StSpace> {
        self.0.res_context.slot()
    }
}

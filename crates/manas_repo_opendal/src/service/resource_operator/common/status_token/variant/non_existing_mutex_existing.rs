//! I provide non-existing--mutex-existing resource status token
//! implementation for ODR.
//!

use std::sync::Arc;

use manas_repo::service::resource_operator::common::status_token::{
    NonExistingMutexExistingResourceToken, RepoResourceStatusTokenBase,
};
use manas_space::resource::{slot::SolidResourceSlot, uri::SolidResourceUri};

use crate::{
    context::ODRContext,
    service::resource_operator::common::status_token::inputs::ODRResourceStatusTokenInputs,
    setup::ODRSetup, OpendalRepo,
};

/// A struct to represent non-existing--mutex-existing
/// resource status token for odr.
#[derive(Debug, Clone)]
pub struct ODRNonExistingMutexExistingResourceToken<Setup: ODRSetup> {
    /// Uri of the resource.
    pub(in super::super) uri: SolidResourceUri,

    /// Repo context.
    pub(in super::super) repo_context: Arc<ODRContext<Setup>>,

    /// Own status inputs.
    /// If is `None`, then slot couldn't be assigned either.
    pub(in super::super) own_inputs: Option<ODRResourceStatusTokenInputs<Setup>>,

    /// Mutex resource's status inputs.
    pub(in super::super) mutex_inputs: ODRResourceStatusTokenInputs<Setup>,
}

impl<Setup: ODRSetup> RepoResourceStatusTokenBase
    for ODRNonExistingMutexExistingResourceToken<Setup>
{
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup: ODRSetup> NonExistingMutexExistingResourceToken
    for ODRNonExistingMutexExistingResourceToken<Setup>
{
    #[inline]
    fn uri(&self) -> &SolidResourceUri {
        &self.uri
    }

    #[inline]
    fn mutex_slot(&self) -> &SolidResourceSlot<Setup::StSpace> {
        self.mutex_inputs.res_context.slot()
    }
}

impl<Setup: ODRSetup> ODRNonExistingMutexExistingResourceToken<Setup> {
    /// Get own status inputs.
    #[inline]
    pub fn own_status_inputs(&self) -> Option<&ODRResourceStatusTokenInputs<Setup>> {
        self.own_inputs.as_ref()
    }
}

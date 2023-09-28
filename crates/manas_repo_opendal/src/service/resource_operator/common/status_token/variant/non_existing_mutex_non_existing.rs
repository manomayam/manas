//! I provide non-existing--mutex-non-existing resource
//! status token implementation for ODR.
//!

use std::sync::Arc;

use manas_repo::service::resource_operator::common::status_token::{
    NonExistingMutexNonExistingResourceToken, RepoResourceStatusTokenBase,
};
use manas_space::resource::uri::SolidResourceUri;

use crate::{
    context::ODRContext,
    service::resource_operator::common::status_token::inputs::ODRResourceStatusTokenInputs,
    setup::ODRSetup, OpendalRepo,
};

/// A struct to represent non-existing--mutex-non-existing
/// resource status token for odr.
#[derive(Debug, Clone)]
pub struct ODRNonExistingMutexNonExistingResourceToken<Setup: ODRSetup> {
    /// Uri of the resource.
    pub(in super::super) uri: SolidResourceUri,

    /// Repo context.
    pub(in super::super) repo_context: Arc<ODRContext<Setup>>,

    /// Own status inputs.
    /// If is `None`, then slot couldn't be assigned either.
    pub(in super::super) own_inputs: Option<ODRResourceStatusTokenInputs<Setup>>,

    /// Mutex resource's status inputs.
    /// If is `None`, then slot couldn't be assigned either.
    pub(in super::super) _mutex_inputs: Option<ODRResourceStatusTokenInputs<Setup>>,
}

impl<Setup: ODRSetup> RepoResourceStatusTokenBase
    for ODRNonExistingMutexNonExistingResourceToken<Setup>
{
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup: ODRSetup> NonExistingMutexNonExistingResourceToken
    for ODRNonExistingMutexNonExistingResourceToken<Setup>
{
    #[inline]
    fn uri(&self) -> &SolidResourceUri {
        &self.uri
    }

    #[inline]
    fn was_existing(&self) -> bool {
        // Currently ODR doesn't track past history.
        false
    }
}

impl<Setup: ODRSetup> ODRNonExistingMutexNonExistingResourceToken<Setup> {
    /// Get own status inputs.
    #[inline]
    pub fn own_status_inputs(&self) -> Option<&ODRResourceStatusTokenInputs<Setup>> {
        self.own_inputs.as_ref()
    }
}

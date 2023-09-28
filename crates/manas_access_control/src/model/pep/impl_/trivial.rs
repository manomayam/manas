//! I define a trivial implementation of
//! [`PolicyEnforcementPoint`] that grants access to every
//! operation on every resource.
//!

use std::marker::PhantomData;

use acp::model::access_mode::{H_APPEND, H_CONTROL, H_READ, H_WRITE};
use dyn_problem::ProbFuture;
use manas_authentication::common::credentials::{impl_::void::VoidCredentials, RequestCredentials};
use manas_space::SolidStorageSpace;

use crate::model::{
    pep::{PolicyEnforcementPoint, ResolvedAccessControl, ResolvedAccessControlResponse},
    AccessGrantSet, ActionOpList, Authorization,
};

/// A trivial implementation of
/// [`PolicyEnforcementPoint`] that grants access to every
/// operation on every resource.
#[derive(Debug, Clone)]
pub struct TrivialPolicyEnforcementPoint<Space, Credentials> {
    /// Set of all known access modes.
    pub all_access_modes: AccessGrantSet,
    _phantom: PhantomData<fn(Space, Credentials)>,
}

impl<Space, Credentials> Default for TrivialPolicyEnforcementPoint<Space, Credentials> {
    fn default() -> Self {
        Self {
            all_access_modes: [&H_READ, &H_APPEND, &H_WRITE, &H_CONTROL]
                .into_iter()
                .cloned()
                .map(|m| m.map_term())
                .collect(),
            _phantom: Default::default(),
        }
    }
}

impl<Space, Credentials> PolicyEnforcementPoint
    for TrivialPolicyEnforcementPoint<Space, Credentials>
where
    Space: SolidStorageSpace,
    Credentials: RequestCredentials + Clone + Into<VoidCredentials>,
{
    type StSpace = Space;

    type Credentials = Credentials;

    #[inline]
    fn resolve_access_control(
        &self,
        action_op_list: ActionOpList,
        credentials: Self::Credentials,
    ) -> ProbFuture<'static, ResolvedAccessControlResponse<Credentials>> {
        Box::pin(futures::future::ready(Ok(ResolvedAccessControlResponse {
            resolved: ResolvedAccessControl::Allow {
                authorization: Authorization {
                    target: action_op_list.on.clone(),
                    credentials,
                    grants: self.all_access_modes.clone(),
                },
            },
            action_op_list,
        })))
    }
}

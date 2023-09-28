//! I define interfaces and implementations for
//! access control policy-enforcement-points.
//!

use std::fmt::Debug;

use dyn_problem::ProbFuture;
use manas_authentication::common::credentials::{impl_::void::VoidCredentials, RequestCredentials};
use manas_space::SolidStorageSpace;

use super::{ActionOpList, ResolvedAccessControl};

pub mod impl_;

/// A type to represent resolved access control response.
#[derive(Debug, Clone)]
pub struct ResolvedAccessControlResponse<C: RequestCredentials> {
    /// Requested
    pub action_op_list: ActionOpList,

    /// Resolved access control.
    pub resolved: ResolvedAccessControl<C>,
}

/// A trait for access control policy enforcement points.
/// A policy enforcement point resolves access control for an
/// action on a resource.
pub trait PolicyEnforcementPoint: Debug + Send + Sync + 'static {
    /// Type of solid storage space.
    type StSpace: SolidStorageSpace;

    /// Type of credentials this PEP supports.
    type Credentials: RequestCredentials + Clone + Into<VoidCredentials>;

    /// Resolve access control for the given op list with
    /// given credentials.
    fn resolve_access_control(
        &self,
        action_op_list: ActionOpList,
        credentials: Self::Credentials,
    ) -> ProbFuture<'static, ResolvedAccessControlResponse<Self::Credentials>>;
}

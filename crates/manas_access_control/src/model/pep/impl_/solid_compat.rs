//! I define a solid compatible implementation of [`PolicyEnforcementPoint`].
//!

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

use acp::model::{
    access_mode::{H_APPEND, H_CONTROL, H_READ, H_WRITE},
    context::HContext,
};
use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbFuture};
use futures::TryFutureExt;
use manas_authentication::common::credentials::{
    impl_::{basic::BasicRequestCredentials, void::VoidCredentials},
    AgentCredentials, RequestCredentials, ToContext,
};
use manas_space::{resource::operation::SolidResourceOperation, SolidStorageSpace};
use once_cell::sync::Lazy;
use rdf_utils::model::{
    description::DescriptionExt, graph::InfallibleMutableGraph, handle::Handle, triple::ArcTriple,
};
use rdf_vocabularies::ns;
use sophia_api::term::{BnodeId, Term};
use tracing::{error, warn};

use crate::model::{
    pdp::{
        AccessGrantResponse, PolicyDecisionPoint, ResourceAccessContext, UNKNOWN_TARGET_RESOURCE,
    },
    pep::{
        ActionOpList, PolicyEnforcementPoint, ResolvedAccessControl, ResolvedAccessControlResponse,
    },
    prp::PolicyRetrievalPoint,
    AccessGrantSet, Authorization,
};

/// Type of least privilege map. It maps a resource operation to
/// minimal required set of access modes required for it.
pub type LeastPrivilegeMap = HashMap<SolidResourceOperation, AccessGrantSet>;

/// Default least privilege map.
pub static DEFAULT_LEAST_PRIVILEGE_MAP: Lazy<Arc<LeastPrivilegeMap>> = Lazy::new(|| {
    let mut map = HashMap::new();

    for (op, privs) in [
        (SolidResourceOperation::READ, [&H_READ]),
        (SolidResourceOperation::APPEND, [&H_APPEND]),
        (SolidResourceOperation::WRITE, [&H_WRITE]),
        // (SolidResourceOperation::UPDATE, [&H_WRITE]),
        (SolidResourceOperation::CREATE, [&H_WRITE]),
        (SolidResourceOperation::DELETE, [&H_WRITE]),
    ] {
        map.insert(op, privs.iter().map(|h| (*h).clone().map_term()).collect());
    }

    Arc::new(map)
});

/// A trait for setup of [`SolidCompatPolicyEnforcementPoint`].
pub trait SolidCompatPolicyEnforcementPointSetup: Debug + Send + 'static {
    /// Type of solid storage space.
    type StSpace: SolidStorageSpace;

    /// Type of graph.
    type Graph: InfallibleMutableGraph + Default + Send + Sync + 'static;

    /// Type of policy retrieval point.
    type PRP: PolicyRetrievalPoint<
            StSpace = Self::StSpace,
            Graph = Self::Graph,
            WGraph = Arc<Self::Graph>,
        > + Send
        + Sync
        + 'static;

    /// Type of policy decision point.
    type PDP: PolicyDecisionPoint<StSpace = Self::StSpace, Graph = Self::Graph>
        + Send
        + Sync
        + 'static;

    /// Type of credentials it supports.
    type Credentials: ToContext + Clone + Into<VoidCredentials>;

    /// Get the least privilege map of this setup.
    /// Implementations must return same static value
    ///  on every call.
    fn least_privilege_map() -> &'static LeastPrivilegeMap;

    /// Default grant for owner on storage root.
    fn storage_root_owner_grant() -> &'static AccessGrantSet;
}

/// A solid storage space compatible implementation of
/// [`PolicyEnforcementPoint`]. It delegates policy retrieval
/// to a solid-aware policy retrieval point, and policy
/// decisions to a policy decision point to resolve access
/// control..
#[derive(Debug)]
pub struct SolidCompatPolicyEnforcementPoint<Setup: SolidCompatPolicyEnforcementPointSetup> {
    /// Storage space.
    pub storage_space: Arc<Setup::StSpace>,

    /// Policy retrieval point.
    pub prp: Arc<Setup::PRP>,

    /// Policy decision point.
    pub pdp: Arc<Setup::PDP>,
}

impl<Setup: SolidCompatPolicyEnforcementPointSetup> Clone
    for SolidCompatPolicyEnforcementPoint<Setup>
{
    fn clone(&self) -> Self {
        Self {
            storage_space: self.storage_space.clone(),
            prp: self.prp.clone(),
            pdp: self.pdp.clone(),
        }
    }
}

impl<Setup: SolidCompatPolicyEnforcementPointSetup> PolicyEnforcementPoint
    for SolidCompatPolicyEnforcementPoint<Setup>
{
    type StSpace = Setup::StSpace;

    type Credentials = Setup::Credentials;

    #[tracing::instrument(
        skip_all,
        name = "SolidCompatPolicyEnforcementPoint::resolve_access_control",
        fields(action_op_list, credentials)
    )]
    fn resolve_access_control(
        &self,
        action_op_list: ActionOpList,
        credentials: Setup::Credentials,
    ) -> ProbFuture<'static, ResolvedAccessControlResponse<Setup::Credentials>> {
        // Convert credentials to context.
        let mut context = credentials.to_context::<Setup::Graph>(
            HContext::try_new(BnodeId::new_unchecked("access_context").into_term())
                .expect("Blank node must be valid context handle term"),
        );

        // Set target.
        context.set(&ns::acp::target, action_op_list.on.deref());

        // If agent is the storage owner.
        let agent_is_storage_owner = credentials
            .of_agent()
            .map(|agent_creds| agent_creds.webid() == self.storage_space.owner_id())
            .unwrap_or(false);

        // TODO setup resource creator,

        let res_access_context =
            ResourceAccessContext::new_unchecked(action_op_list.on.clone(), context);

        let prp = self.prp.clone();
        let pdp = self.pdp.clone();

        Box::pin(async move {
            // Retrieve acr chain from prp.
            let acr_chain = prp
                .retrieve(res_access_context.target_uri().clone(), false)
                .await
                .map_err(|e| {
                    if UNKNOWN_TARGET_RESOURCE.is_type_of(&e) {
                        // TODO
                        error!("Target resource is unknown to prp. Error:\n {}", e);
                        UNKNOWN_TARGET_RESOURCE.new_problem_builder()
                    } else {
                        error!(
                            "Unknown io error in retrieving policies from prp. Error:\n {}",
                            e
                        );
                        UNKNOWN_IO_ERROR.new_problem_builder()
                    }
                    .source(e)
                    .finish()
                })?;

            // Get resolved access grant from pdp.
            let mut access_grant_response: AccessGrantResponse<Setup::StSpace> = pdp
                .resolve_grants(res_access_context, acr_chain)
                .inspect_err(|e| {
                    error!("Error in resolving access grant by prp. Error:\n {}", e);
                })
                .await?;

            // Extend grants for storage owner on storage root and acl.
            if agent_is_storage_owner {
                // If resource exists and,
                if let Some(res_slot) = &access_grant_response.res_slot {
                    // If resource is the acr of storage root
                    if res_slot.is_root_acl_slot()
                        // And default owner grant on storage root includes `Control`
                        && Setup::storage_root_owner_grant().contains(&H_CONTROL.clone().map_term())
                    {
                        // Then grant all supported access modes on acl.
                        access_grant_response
                            .access_grant_set
                            .extend(pdp.supported_access_modes().iter().cloned())
                    } else if res_slot.is_root_slot() {
                        // If resource is the storage root, then
                        access_grant_response
                            .access_grant_set
                            .extend(Setup::storage_root_owner_grant().iter().cloned())
                    }
                }
            }

            // Resolve list of denied ops.
            let denied_ops = action_op_list
                .ops
                .iter()
                .filter(|op| {
                    // Op is allowed, if any of it's generalized op
                    // is allowed.
                    !op.op.generalized().any(|gop| {
                        Setup::least_privilege_map()
                            .get(&gop)
                            .map(|required_modes| {
                                access_grant_response
                                    .access_grant_set
                                    .is_superset(required_modes)
                            })
                            .unwrap_or_else(|| {
                                warn!("Least privilege set is not configured for op. Op: {}", gop);
                                false
                            })
                    })
                })
                .cloned()
                .collect::<Vec<_>>();

            let authorization = Authorization {
                target: action_op_list.on.clone(),
                credentials,
                grants: access_grant_response.access_grant_set,
            };

            Ok(ResolvedAccessControlResponse {
                action_op_list,
                resolved: if let Ok(denied_ops1) = denied_ops.try_into() {
                    ResolvedAccessControl::Deny {
                        authorization,
                        denied_ops: denied_ops1,
                    }
                } else {
                    ResolvedAccessControl::Allow { authorization }
                },
            })
        })
    }
}

/// Default owner storage root grant.
static DEFAULT_STORAGE_ROOT_OWNER_GRANT: Lazy<AccessGrantSet> = Lazy::new(|| {
    [&H_READ, &H_CONTROL]
        .into_iter()
        .map(|m| m.clone().map_term())
        .collect()
});

/// A simple implementation of [`SolidCompatPolicyEnforcementPointSetup`].
pub struct SimplePolicyEnforcementPointSetup<StSpace, PRP, PDP> {
    _phantom: PhantomData<fn(StSpace, PRP, PDP)>,
}

/// Alias for type of [`SolidCompatPolicyEnforcementPoint`]
/// with simple setup.
pub type SimplePolicyEnforcementPoint<StSpace, PRP, PDP> =
    SolidCompatPolicyEnforcementPoint<SimplePolicyEnforcementPointSetup<StSpace, PRP, PDP>>;

impl<StSpace, PRP, PDP> Debug for SimplePolicyEnforcementPointSetup<StSpace, PRP, PDP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimplePolicyEnforcementPointSetup").finish()
    }
}

impl<StSpace, PRP, PDP> SolidCompatPolicyEnforcementPointSetup
    for SimplePolicyEnforcementPointSetup<StSpace, PRP, PDP>
where
    StSpace: SolidStorageSpace,
    PRP: PolicyRetrievalPoint<
            StSpace = StSpace,
            Graph = HashSet<ArcTriple>,
            WGraph = Arc<HashSet<ArcTriple>>,
        > + Send
        + Sync
        + 'static,

    PDP: PolicyDecisionPoint<StSpace = StSpace, Graph = HashSet<ArcTriple>> + Send + Sync + 'static,
{
    type StSpace = StSpace;

    // TODO should be indexed versions, ensuring `Extend` api.
    type Graph = HashSet<ArcTriple>;

    type PRP = PRP;

    type PDP = PDP;

    type Credentials = BasicRequestCredentials;

    #[inline]
    fn least_privilege_map() -> &'static LeastPrivilegeMap {
        &DEFAULT_LEAST_PRIVILEGE_MAP
    }

    #[inline]
    fn storage_root_owner_grant() -> &'static AccessGrantSet {
        &DEFAULT_STORAGE_ROOT_OWNER_GRANT
    }
}

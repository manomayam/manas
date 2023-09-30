//! I define message types for [`ResourceCreator`](super::ResourceCreator).
//!

use std::sync::Arc;

use manas_space::{
    resource::{
        kind::SolidResourceKind,
        slot::{InvalidSolidResourceSlot, SolidResourceSlot},
        slot_id::SolidResourceSlotId,
        slot_rel_type::SlotRelationType,
        slot_rev_link::SlotRevLink,
    },
    SpcKnownAuxRelType,
};
use typed_record::ClonableTypedRecord;

use crate::{
    context::RepoContext,
    layer::LayeredRepo,
    service::resource_operator::common::{
        preconditions::Preconditions,
        rep_update_action::RepUpdateAction,
        status_token::{
            impl_::layered::Layered, ExistingRepresentedResourceToken, InconsistentTokenSet,
            NonExistingMutexNonExistingResourceToken, RepoResourceStatusTokenBase,
        },
    },
    Repo, RepoConflictFreeResourceStatusToken, RepoRepresentedResourceToken,
};

/// A type to represent set of tokens required for resource
/// creation.
#[derive(Debug)]
pub struct ResourceCreateTokenSet<R: Repo> {
    /// Repo context.
    repo_context: Arc<R::Context>,

    /// Conflict free status token for target resource.
    res_token: RepoConflictFreeResourceStatusToken<R>,

    /// Represented status token for host resource.
    host_token: RepoRepresentedResourceToken<R>,
}

impl<R: Repo> ResourceCreateTokenSet<R> {
    /// Try to create a new [`ResourceCreateTokenSet`] from
    /// given params.
    pub fn try_new(
        res_token: RepoConflictFreeResourceStatusToken<R>,
        host_token: RepoRepresentedResourceToken<R>,
    ) -> Result<Self, InconsistentTokenSet> {
        // Ensure that both tokens have same repo context.
        Arc::ptr_eq(res_token.repo_context(), host_token.repo_context())
            .then(|| Self {
                repo_context: res_token.repo_context().clone(),
                host_token,
                res_token,
            })
            .ok_or(InconsistentTokenSet)
    }

    /// Get the repo context from tokens.
    #[inline]
    pub fn repo_context(&self) -> &Arc<R::Context> {
        &self.repo_context
    }

    /// Get the storage space from tokens.
    #[inline]
    pub fn storage_space(&self) -> &Arc<R::StSpace> {
        self.repo_context.storage_space()
    }

    /// Get resource token
    #[inline]
    pub fn res_token(&self) -> &RepoConflictFreeResourceStatusToken<R> {
        &self.res_token
    }

    /// Get host resource token
    #[inline]
    pub fn host_token(&self) -> &RepoRepresentedResourceToken<R> {
        &self.host_token
    }

    /// Convert into parts.
    /// (resource_token, host_token)
    #[inline]
    pub fn into_parts(
        self,
    ) -> (
        RepoConflictFreeResourceStatusToken<R>,
        RepoRepresentedResourceToken<R>,
    ) {
        (self.res_token, self.host_token)
    }
}

/// A struct to represent resource create request.
#[derive(Debug)]
pub struct ResourceCreateRequest<R: Repo> {
    /// Set of required tokens.
    pub tokens: ResourceCreateTokenSet<R>,

    /// Kind of resource to be created.
    pub resource_kind: SolidResourceKind,

    /// slot reverse relation type.
    pub slot_rev_rel_type: SlotRelationType<SpcKnownAuxRelType<R::StSpace>>,

    /// Representation update action.
    pub rep_update_action: RepUpdateAction<R>,

    /// Preconditions for the operation against the host
    /// resource.
    pub host_preconditions: Box<dyn Preconditions>,

    /// Request credentials.
    pub credentials: R::Credentials,

    /// Any extensions.
    pub extensions: ClonableTypedRecord,
}

impl<R: Repo> ResourceCreateRequest<R> {
    /// Map tokens in the request.
    pub fn map_tokens<R2, F>(self, f: F) -> ResourceCreateRequest<R2>
    where
        R2: Repo<StSpace = R::StSpace>,
        R::Representation: Into<R2::Representation>,
        R::RepPatcher: Into<R2::RepPatcher>,
        R::Credentials: Into<R2::Credentials>,
        F: FnOnce(ResourceCreateTokenSet<R>) -> ResourceCreateTokenSet<R2>,
    {
        ResourceCreateRequest {
            tokens: f(self.tokens),
            resource_kind: self.resource_kind,
            slot_rev_rel_type: self.slot_rev_rel_type,
            rep_update_action: self.rep_update_action.map_repo(),
            host_preconditions: self.host_preconditions,
            credentials: self.credentials.into(),
            extensions: self.extensions,
        }
    }

    /// Unlayer the tokens.
    pub fn unlayer_tokens<IR>(self) -> ResourceCreateRequest<IR>
    where
        IR: Repo,
        R: LayeredRepo<IR>,
        R::Representation: Into<IR::Representation>,
        R::RepPatcher: Into<IR::RepPatcher>,
        R::Credentials: Into<IR::Credentials>,
    {
        self.map_tokens(|tokens| Layered::from(tokens).inner)
    }

    /// Try to get equivalent resource slot.
    pub fn try_equivalent_res_slot(
        &self,
    ) -> Result<SolidResourceSlot<R::StSpace>, InvalidSolidResourceSlot> {
        SolidResourceSlot::try_new(
            SolidResourceSlotId::new(
                self.tokens.storage_space().clone(),
                self.tokens.res_token().uri().clone(),
            ),
            self.resource_kind,
            Some(SlotRevLink {
                rev_rel_type: self.slot_rev_rel_type.clone(),
                target: self.tokens.host_token().slot().id().uri.clone(),
            }),
        )
    }
}

/// A struct to represent resource create response.
#[derive(Debug)]
pub struct ResourceCreateResponse<R: Repo> {
    /// Created resource slot.
    pub created_resource_slot: SolidResourceSlot<R::StSpace>,

    /// Extensions.
    pub extensions: http::Extensions,
}

impl<R: Repo> ResourceCreateResponse<R> {
    /// Map repo.
    #[inline]
    pub fn map_repo<R2: Repo<StSpace = R::StSpace>>(self) -> ResourceCreateResponse<R2> {
        ResourceCreateResponse {
            created_resource_slot: self.created_resource_slot,
            extensions: self.extensions,
        }
    }
}

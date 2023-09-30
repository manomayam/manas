//! I provide an implementation of [`Repo`] that layers
//! access control over an inner repo.
//!

use std::{marker::PhantomData, sync::Arc};

use manas_repo::{
    context::LayeredRepoContext,
    layer::RepoLayer,
    policy::uri::impl_::DelegatedUriPolicy,
    service::{
        patcher_resolver::impl_::DelegatedRepPatcherResolver,
        resource_operator::{
            common::status_token::impl_::layered::LayeredResourceStatusTokenTypes,
            status_token_resolver::impl_::LayeredResourceStatusTokenResolver,
        },
    },
    Repo, RepoRepPatcherResolver, RepoResourceStatusTokenResolver, RepoServices,
};

use self::{
    context::{AccessControlledRepoContext, InitialRootAcrRepFactory},
    service::{
        initializer::AccessControlledRepoInitializer,
        resource_operator::{
            creator::AccessControlledResourceCreator, deleter::AccessControlledResourceDeleter,
            reader::AccessControlledResourceReader, updater::AccessControlledResourceUpdater,
        },
    },
};
use crate::model::pep::PolicyEnforcementPoint;

pub mod context;
pub mod service;

/// An implementation of [`Repo`] that layers  access control
/// over an inner repo
#[derive(Debug, Clone)]
pub struct AccessControlledRepo<IR: Repo, PEP> {
    /// Inner repo.
    _inner: IR,

    /// Context.
    context: Arc<AccessControlledRepoContext<IR, PEP>>,
}

impl<IR, PEP> Repo for AccessControlledRepo<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = AccessControlledRepoContext<IR, PEP>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = IR::RepPatcher;

    type Services = AccessControlledRepoServices<IR, PEP>;

    type Credentials = PEP::Credentials;

    #[inline]
    fn new(context: Arc<Self::Context>) -> Self {
        Self {
            _inner: IR::new(context.inner().clone()),
            context,
        }
    }

    #[inline]
    fn context(&self) -> &Arc<Self::Context> {
        &self.context
    }
}

/// [`RepoServices`] implementations for [`AccessControlledRepo`].
#[derive(Debug, Clone)]
pub struct AccessControlledRepoServices<IR: Repo, PEP> {
    _phantom: PhantomData<fn(IR, PEP)>,
}

impl<IR, PEP> RepoServices for AccessControlledRepoServices<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type Repo = AccessControlledRepo<IR, PEP>;

    type Initializer = AccessControlledRepoInitializer<IR, PEP>;

    type RepPatcherResolver =
        DelegatedRepPatcherResolver<RepoRepPatcherResolver<IR>, AccessControlledRepo<IR, PEP>>;

    type ResourceStatusTokenResolver = LayeredResourceStatusTokenResolver<
        RepoResourceStatusTokenResolver<IR>,
        AccessControlledRepo<IR, PEP>,
    >;

    type ResourceReader = AccessControlledResourceReader<IR, PEP>;

    type ResourceCreator = AccessControlledResourceCreator<IR, PEP>;

    type ResourceUpdater = AccessControlledResourceUpdater<IR, PEP>;

    type ResourceDeleter = AccessControlledResourceDeleter<IR, PEP>;
}

/// An implementation of [`RepoLayer`] that layers access control
/// functionality over repos.
#[derive(Clone)]
pub struct AccessControlledRepoLayer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    /// Policy enforcement point.
    pub pep: Arc<PEP>,

    /// Initial storage root acr in ttl.
    pub initial_root_acr_rep_factory: Arc<InitialRootAcrRepFactory<IR>>,

    _phantom: PhantomData<fn(IR, PEP)>,
}

impl<IR, PEP> std::fmt::Debug for AccessControlledRepoLayer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessControlledRepoLayer")
            .field("pep", &self.pep)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<IR, PEP> AccessControlledRepoLayer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    /// Create a new [`AccessControlledRepoLayer`].
    #[inline]
    pub fn new(
        pep: Arc<PEP>,
        initial_root_acr_rep_factory: Arc<InitialRootAcrRepFactory<IR>>,
    ) -> Self {
        Self {
            pep,
            initial_root_acr_rep_factory,
            _phantom: PhantomData,
        }
    }
}

impl<IR, PEP> RepoLayer<IR> for AccessControlledRepoLayer<IR, PEP>
where
    IR: Repo,
    PEP: PolicyEnforcementPoint<StSpace = IR::StSpace>,
    PEP::Credentials: Into<IR::Credentials>,
{
    type LayeredRepo = AccessControlledRepo<IR, PEP>;

    #[inline]
    fn layer_context(
        &self,
        inner_context: Arc<<IR as Repo>::Context>,
    ) -> <Self::LayeredRepo as Repo>::Context {
        AccessControlledRepoContext {
            inner: inner_context,
            pep: self.pep.clone(),
            initial_root_acr_rep_factory: self.initial_root_acr_rep_factory.clone(),
        }
    }
}

//! I provide a layered implementation of [`Repo`] that
//! resolves rep patch operations to set_with operations.
//!

use std::{marker::PhantomData, sync::Arc};

use manas_repo::{
    policy::uri::impl_::DelegatedUriPolicy,
    service::{
        initializer::impl_::DelegatedRepoInitializer,
        patcher_resolver::impl_::UnsupportedRepPatcher,
        resource_operator::{
            common::{
                impl_::DelegatingOperator,
                status_token::impl_::layered::LayeredResourceStatusTokenTypes,
            },
            status_token_resolver::impl_::LayeredResourceStatusTokenResolver,
        },
    },
    Repo, RepoInitializerService, RepoResourceDeleter, RepoResourceReader,
    RepoResourceStatusTokenResolver, RepoServices,
};

use self::{
    context::PatchingRepoContext,
    patcher::DirectRepPatcher,
    service::{
        patcher_resolver::DirectRepPatcherResolver,
        resource_operator::{
            creator::PatchingRepoResourceCreator, updater::PatchingRepoResourceUpdater,
        },
    },
};

pub mod context;
pub mod patcher;
pub mod service;

/// A layered implementation of [`Repo`] that that
/// resolves rep patch operations to set_with operations
///
/// NOTE: The inner repo must be access control free for this layer to work.
/// Thus any access control layer must wrap outside of this layer.
#[derive(Debug, Clone)]
pub struct PatchingRepo<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    context: Arc<PatchingRepoContext<IR, P>>,
}

impl<IR, P> Repo for PatchingRepo<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = PatchingRepoContext<IR, P>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = P;

    type Services = PatchingRepoServices<IR, P>;

    type Credentials = IR::Credentials;

    #[inline]
    fn new(context: Arc<Self::Context>) -> Self {
        Self { context }
    }

    #[inline]
    fn context(&self) -> &Arc<Self::Context> {
        &self.context
    }
}

/// Quick alias for `PatchingRepo`
pub(crate) type MRepo<IR, P> = PatchingRepo<IR, P>;

/// Services for [`PatchingRepo`].
#[derive(Debug, Clone)]
pub struct PatchingRepoServices<IR, P> {
    _phantom: PhantomData<fn(IR, P)>,
}

impl<IR, P> RepoServices for PatchingRepoServices<IR, P>
where
    IR: Repo<RepPatcher = UnsupportedRepPatcher>,
    P: DirectRepPatcher<IR::StSpace, IR::Representation>,
{
    type Repo = MRepo<IR, P>;

    type Initializer = DelegatedRepoInitializer<RepoInitializerService<IR>, MRepo<IR, P>>;

    type RepPatcherResolver = DirectRepPatcherResolver<IR, P>;

    type ResourceStatusTokenResolver =
        LayeredResourceStatusTokenResolver<RepoResourceStatusTokenResolver<IR>, MRepo<IR, P>>;

    type ResourceReader = DelegatingOperator<RepoResourceReader<IR>, MRepo<IR, P>>;

    type ResourceCreator = PatchingRepoResourceCreator<IR, P>;

    type ResourceUpdater = PatchingRepoResourceUpdater<IR, P>;

    type ResourceDeleter = DelegatingOperator<RepoResourceDeleter<IR>, MRepo<IR, P>>;
}

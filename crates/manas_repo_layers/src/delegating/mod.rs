//! I provide a layered implementation of [`Repo`] that
//! simply delegates all operations to inner repo.
//!

use std::{marker::PhantomData, sync::Arc};

use manas_repo::{
    policy::uri::impl_::DelegatedUriPolicy,
    service::{
        initializer::impl_::DelegatedRepoInitializer,
        patcher_resolver::impl_::DelegatedRepPatcherResolver,
        resource_operator::{
            common::{
                impl_::DelegatingOperator,
                status_token::impl_::layered::LayeredResourceStatusTokenTypes,
            },
            status_token_resolver::impl_::LayeredResourceStatusTokenResolver,
        },
    },
    Repo, RepoInitializerService, RepoRepPatcherResolver, RepoResourceCreator, RepoResourceDeleter,
    RepoResourceReader, RepoResourceStatusTokenResolver, RepoResourceUpdater, RepoServices,
};

use self::context::DelegatingRepoContext;

pub mod context;
// pub mod service;

/// A layered implementation of [`Repo`] that that
/// simply delegates all operations to inner repo.
///
/// It is intended as starting template for new repo layers.
#[derive(Clone)]
pub struct DelegatingRepo<IR, V>
where
    IR: Repo,
{
    context: Arc<DelegatingRepoContext<IR, V>>,
}

impl<IR, V> std::fmt::Debug for DelegatingRepo<IR, V>
where
    IR: Repo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingRepo")
            .field("context", &self.context)
            .finish()
    }
}

impl<IR, V> Repo for DelegatingRepo<IR, V>
where
    IR: Repo,
    V: 'static,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = DelegatingRepoContext<IR, V>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = IR::RepPatcher;

    type Services = DelegatingRepoServices<IR, V>;

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

/// Quick alias for `DelegatingRepo`
pub(crate) type MRepo<IR, V> = DelegatingRepo<IR, V>;

/// Services for [`DelegatingRepo`].
#[derive(Debug, Clone)]
pub struct DelegatingRepoServices<IR, V> {
    _phantom: PhantomData<fn(IR, V)>,
}

impl<IR, V> RepoServices for DelegatingRepoServices<IR, V>
where
    IR: Repo,
    V: 'static,
{
    type Repo = MRepo<IR, V>;

    type Initializer = DelegatedRepoInitializer<RepoInitializerService<IR>, MRepo<IR, V>>;

    type RepPatcherResolver = DelegatedRepPatcherResolver<RepoRepPatcherResolver<IR>, MRepo<IR, V>>;

    type ResourceStatusTokenResolver =
        LayeredResourceStatusTokenResolver<RepoResourceStatusTokenResolver<IR>, MRepo<IR, V>>;

    type ResourceReader = DelegatingOperator<RepoResourceReader<IR>, MRepo<IR, V>>;

    type ResourceCreator = DelegatingOperator<RepoResourceCreator<IR>, MRepo<IR, V>>;

    type ResourceUpdater = DelegatingOperator<RepoResourceUpdater<IR>, MRepo<IR, V>>;

    type ResourceDeleter = DelegatingOperator<RepoResourceDeleter<IR>, MRepo<IR, V>>;
}

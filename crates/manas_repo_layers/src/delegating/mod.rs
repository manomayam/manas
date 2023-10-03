//! I provide a layered implementation of [`Repo`] that
//! simply delegates all operations to inner repo.
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use manas_repo::{
    layer::RepoLayer,
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
pub struct DelegatingRepo<IR, DLR>
where
    IR: Repo,
{
    context: Arc<DelegatingRepoContext<IR, DLR>>,
}

impl<IR, DLR> Debug for DelegatingRepo<IR, DLR>
where
    IR: Repo,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatingRepo")
            .field("context", &self.context)
            .finish()
    }
}

impl<IR, DLR> Repo for DelegatingRepo<IR, DLR>
where
    IR: Repo,
    DLR: 'static,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = DelegatingRepoContext<IR, DLR>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = IR::RepPatcher;

    type Services = DelegatingRepoServices<IR, DLR>;

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
pub(crate) type MRepo<IR, DLR> = DelegatingRepo<IR, DLR>;

/// Services for [`DelegatingRepo`].
#[derive(Debug, Clone)]
pub struct DelegatingRepoServices<IR, DLR> {
    _phantom: PhantomData<fn(IR, DLR)>,
}

impl<IR, DLR> RepoServices for DelegatingRepoServices<IR, DLR>
where
    IR: Repo,
    DLR: 'static,
{
    type Repo = MRepo<IR, DLR>;

    type Initializer = DelegatedRepoInitializer<RepoInitializerService<IR>, MRepo<IR, DLR>>;

    type RepPatcherResolver =
        DelegatedRepPatcherResolver<RepoRepPatcherResolver<IR>, MRepo<IR, DLR>>;

    type ResourceStatusTokenResolver =
        LayeredResourceStatusTokenResolver<RepoResourceStatusTokenResolver<IR>, MRepo<IR, DLR>>;

    type ResourceReader = DelegatingOperator<RepoResourceReader<IR>, MRepo<IR, DLR>>;

    type ResourceCreator = DelegatingOperator<RepoResourceCreator<IR>, MRepo<IR, DLR>>;

    type ResourceUpdater = DelegatingOperator<RepoResourceUpdater<IR>, MRepo<IR, DLR>>;

    type ResourceDeleter = DelegatingOperator<RepoResourceDeleter<IR>, MRepo<IR, DLR>>;
}

/// An implementation of [`RepoLayer`] that is no op.
#[derive(Debug, Clone)]
pub struct DelegatingRepoLayer<IR, DLR>
where
    IR: Repo,
{
    layer_config: Arc<PhantomData<fn(DLR)>>,
    _phantom: PhantomData<fn(IR, DLR)>,
}

impl<IR, DLR> DelegatingRepoLayer<IR, DLR>
where
    IR: Repo,
{
    /// Create a new [`DelegatingRepoLayer`].
    #[inline]
    pub fn new(layer_config: Arc<PhantomData<fn(DLR)>>) -> Self {
        Self {
            layer_config,
            _phantom: PhantomData,
        }
    }
}

impl<IR, DLR> RepoLayer<IR> for DelegatingRepoLayer<IR, DLR>
where
    IR: Repo,
    DLR: Debug + Send + 'static,
{
    type LayeredRepo = DelegatingRepo<IR, DLR>;

    #[inline]
    fn layer_context(
        &self,
        inner_context: Arc<<IR as Repo>::Context>,
    ) -> <Self::LayeredRepo as Repo>::Context {
        DelegatingRepoContext {
            inner: inner_context,
            layer_config: self.layer_config.clone(),
        }
    }
}

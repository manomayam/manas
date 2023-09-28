//! I provide an implementation of [`Repo`] that performs
//! validation of representation update operations.
//!

use std::{marker::PhantomData, sync::Arc};

use manas_repo::{
    policy::uri::impl_::DelegatedUriPolicy,
    service::{
        initializer::impl_::DelegatedRepoInitializer,
        patcher_resolver::impl_::{UnsupportedRepPatcher, UnsupportedRepPatcherResolver},
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
    context::ValidatingRepoContext,
    service::resource_operator::{
        creator::ValidatingRepoResourceCreator, updater::ValidatingRepoResourceUpdater,
    },
    update_validator::RepUpdateValidator,
};

pub mod context;
pub mod service;
pub mod update_validator;

/// A layered implementation of [`Repo`] that performs
/// validation of representation update operations
/// before passing to inner repo.
///
/// NOTE: This layer must be wrapped by a patching layer to
/// support patching. It rejects any patch requests.
#[derive(Debug, Clone)]
pub struct ValidatingRepo<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    context: Arc<ValidatingRepoContext<IR, V>>,
}

impl<IR, V> Repo for ValidatingRepo<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = ValidatingRepoContext<IR, V>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = UnsupportedRepPatcher;

    type Services = ValidatingRepoServices<IR, V>;

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

/// Quick alias for `ValidatingRepo`
pub(crate) type MRepo<IR, V> = ValidatingRepo<IR, V>;

/// Services for [`ValidatingRepo`].
#[derive(Debug, Clone)]
pub struct ValidatingRepoServices<IR, V> {
    _phantom: PhantomData<fn(IR, V)>,
}

impl<IR, V> RepoServices for ValidatingRepoServices<IR, V>
where
    IR: Repo,
    V: RepUpdateValidator<IR>,
{
    type Repo = MRepo<IR, V>;

    type Initializer = DelegatedRepoInitializer<RepoInitializerService<IR>, MRepo<IR, V>>;

    type RepPatcherResolver = UnsupportedRepPatcherResolver<MRepo<IR, V>>;

    type ResourceStatusTokenResolver =
        LayeredResourceStatusTokenResolver<RepoResourceStatusTokenResolver<IR>, MRepo<IR, V>>;

    type ResourceReader = DelegatingOperator<RepoResourceReader<IR>, MRepo<IR, V>>;

    type ResourceCreator = ValidatingRepoResourceCreator<IR, V>;

    type ResourceUpdater = ValidatingRepoResourceUpdater<IR, V>;

    type ResourceDeleter = DelegatingOperator<RepoResourceDeleter<IR>, MRepo<IR, V>>;
}

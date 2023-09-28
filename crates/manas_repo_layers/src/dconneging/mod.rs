//! I provide an implementation of [`Repo`] that performs
//! content negotiation over derivable representations of
//! original representation resolved by inner repo.
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

use self::{
    conneg_layer::DerivedContentNegotiationLayer, context::DerivedContentNegotiatingRepoContext,
    service::resource_operator::reader::DerivedContentNegotiatingResourceReader,
};

pub mod conneg_layer;
pub mod context;
pub mod service;

/// A layered implementation of [`Repo`] that performs
/// content negotiation over derivable representations of
/// original representation resolved by inner repo.
#[derive(Debug, Clone)]
pub struct DerivedContentNegotiatingRepo<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    context: Arc<DerivedContentNegotiatingRepoContext<IR, CNL>>,
}

impl<IR, CNL> Repo for DerivedContentNegotiatingRepo<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type StSpace = IR::StSpace;

    type Representation = IR::Representation;

    type Context = DerivedContentNegotiatingRepoContext<IR, CNL>;

    type UriPolicy = DelegatedUriPolicy<IR::UriPolicy, Self>;

    type ResourceStatusTokenTypes =
        LayeredResourceStatusTokenTypes<IR::ResourceStatusTokenTypes, Self>;

    type RepPatcher = IR::RepPatcher;

    type Services = DerivedContentNegotiatingRepoServices<IR, CNL>;

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

/// Quick alias for `DerivedContentNegotiatingRepo`
pub(crate) type MRepo<IR, CNL> = DerivedContentNegotiatingRepo<IR, CNL>;

/// Services for [`DerivedContentNegotiatingRepo`].
#[derive(Debug, Clone)]
pub struct DerivedContentNegotiatingRepoServices<IR, CNL> {
    _phantom: PhantomData<fn(IR, CNL)>,
}

impl<IR, CNL> RepoServices for DerivedContentNegotiatingRepoServices<IR, CNL>
where
    IR: Repo,
    CNL: DerivedContentNegotiationLayer<IR, IR::Representation, RepoResourceReader<IR>>,
{
    type Repo = MRepo<IR, CNL>;

    type Initializer = DelegatedRepoInitializer<RepoInitializerService<IR>, MRepo<IR, CNL>>;

    type RepPatcherResolver =
        DelegatedRepPatcherResolver<RepoRepPatcherResolver<IR>, MRepo<IR, CNL>>;

    type ResourceStatusTokenResolver =
        LayeredResourceStatusTokenResolver<RepoResourceStatusTokenResolver<IR>, MRepo<IR, CNL>>;

    type ResourceReader = DerivedContentNegotiatingResourceReader<IR, CNL>;

    type ResourceCreator = DelegatingOperator<RepoResourceCreator<IR>, MRepo<IR, CNL>>;

    type ResourceUpdater = DelegatingOperator<RepoResourceUpdater<IR>, MRepo<IR, CNL>>;

    type ResourceDeleter = DelegatingOperator<RepoResourceDeleter<IR>, MRepo<IR, CNL>>;
}

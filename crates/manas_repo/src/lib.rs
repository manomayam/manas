//! This crate defines definition traits for manas storage repositories and their services.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{fmt::Debug, sync::Arc};

use context::{RepoContext, RepoContextual};
use manas_authentication::common::credentials::RequestCredentials;
use manas_http::representation::Representation;
use manas_space::{resource::state::SolidResourceState, SolidStorageSpace};
use policy::uri::RepoUriPolicy;
use service::{
    initializer::RepoInitializer,
    resource_operator::{
        common::rep_patcher::RepPatcher, creator::ResourceCreator, deleter::ResourceDeleter,
        reader::ResourceReader, status_token_resolver::ResourceStatusTokenResolver,
        updater::ResourceUpdater,
    },
};

use self::service::{
    patcher_resolver::RepPatcherResolver,
    resource_operator::common::status_token::{
        ExistingResourceToken, NonExistingResourceToken, ResourceStatusToken,
        ResourceStatusTokenTypes,
    },
};

pub mod context;
pub mod layer;
pub mod policy;
pub mod service;

/// A repo manages resources in a storage space.
pub trait Repo: Debug + Send + Sync + 'static + Unpin + Sized {
    /// Type of storage space of this repo type.
    type StSpace: SolidStorageSpace;

    /// Type of representations used by this repo.
    type Representation: Representation + Send + 'static;

    /// Type of repo context.
    type Context: RepoContext<Repo = Self>;

    /// Type of uri policy.
    type UriPolicy: RepoUriPolicy<Repo = Self>;

    /// Type of resource status token types.
    type ResourceStatusTokenTypes: ResourceStatusTokenTypes<Repo = Self>;

    /// Type of the rep patcher.
    type RepPatcher: RepPatcher + Clone;

    /// Type of repo services.
    type Services: RepoServices<Repo = Self>;

    /// Type of request credentials for this repo.
    type Credentials: RequestCredentials;

    /// Create a new repo with given context.
    fn new(context: Arc<Self::Context>) -> Self;

    /// Get repo context.
    fn context(&self) -> &Arc<Self::Context>;
}

/// A trait for defining services of a repo.
pub trait RepoServices: Send + Sync + 'static + Unpin {
    /// Type of the repo.
    type Repo: Repo;

    /// Type of initializer service.
    type Initializer: RepoInitializer<Repo = Self::Repo>;

    /// Type of rep patcher resolver.
    type RepPatcherResolver: RepPatcherResolver<Repo = Self::Repo>;

    /// Type of the resource status token resolver.
    type ResourceStatusTokenResolver: ResourceStatusTokenResolver<Repo = Self::Repo>;

    /// Type of the resource reader.
    type ResourceReader: ResourceReader<Repo = Self::Repo>;

    /// Type of the resource creator.
    type ResourceCreator: ResourceCreator<Repo = Self::Repo>;

    /// Type of the resource updater.
    type ResourceUpdater: ResourceUpdater<Repo = Self::Repo>;

    /// Type of the resource deleter.
    type ResourceDeleter: ResourceDeleter<Repo = Self::Repo>;
}

/// Alias for type of the space of the repo.
pub type RepoSpace<R> = <R as Repo>::StSpace;

/// Alias for type of representation for a repo.
pub type RepoRepresentation<R> = <R as Repo>::Representation;

/// Alias for type of resource state for a repo
pub type RepoResourceState<R> =
    SolidResourceState<<R as Repo>::StSpace, <R as Repo>::Representation>;

/// Alias for type of initializers of a repo
pub type RepoInitializerService<R> = <<R as Repo>::Services as RepoServices>::Initializer;

/// Alias for type of rep patcher resolvers supported by a
/// repo.
pub type RepoRepPatcherResolver<R> = <<R as Repo>::Services as RepoServices>::RepPatcherResolver;

/// Alias for type of rep patchers supported by a repo.
pub type RepoRepPatcher<R> = <R as Repo>::RepPatcher;

/// Alias for type of resource status token types of a repo.
pub type RepoResourceStatusTokenTypes<R> = <R as Repo>::ResourceStatusTokenTypes;

/// Alias for type of represented resource status tokens of
/// the repo.
pub type RepoRepresentedResourceToken<R> =
    <RepoResourceStatusTokenTypes<R> as ResourceStatusTokenTypes>::ExistingRepresented;

/// Alias for type of existing resource status tokens of
/// the repo.
pub type RepoExistingResourceToken<R> = ExistingResourceToken<RepoResourceStatusTokenTypes<R>>;

/// Alias for type of non-existing resource status tokens of
/// the repo.
pub type RepoNonExistingResourceStatusToken<R> =
    NonExistingResourceToken<RepoResourceStatusTokenTypes<R>>;

/// Alias for type of non-existing + mutex-non-existing resource
/// status tokens of the repo
pub type RepoConflictFreeResourceStatusToken<R> =
    <RepoResourceStatusTokenTypes<R> as ResourceStatusTokenTypes>::NonExistingMutexNonExisting;

/// Alias for type of resource status tokens of the repo.
pub type RepoResourceStatusToken<R> = ResourceStatusToken<RepoResourceStatusTokenTypes<R>>;

/// Alias for type of resource status token resolver for the
/// repo.
pub type RepoResourceStatusTokenResolver<R> =
    <<R as Repo>::Services as RepoServices>::ResourceStatusTokenResolver;

/// Alias for type of resource reader for the repo.
pub type RepoResourceReader<R> = <<R as Repo>::Services as RepoServices>::ResourceReader;

/// Alias for type of resource creator for the repo.
pub type RepoResourceCreator<R> = <<R as Repo>::Services as RepoServices>::ResourceCreator;

/// Alias for type of resource updater for the repo.
pub type RepoResourceUpdater<R> = <<R as Repo>::Services as RepoServices>::ResourceUpdater;

/// Alias for type of resource deleter for the repo.
pub type RepoResourceDeleter<R> = <<R as Repo>::Services as RepoServices>::ResourceDeleter;

pub use repo_ext::RepoExt;

mod repo_ext {
    use dyn_problem::ProbFuture;
    use futures::TryFutureExt;
    use manas_space::resource::uri::SolidResourceUri;
    use tower::{Service, ServiceExt};
    use tracing::{error, info};

    use super::*;
    use crate::service::resource_operator::{
        reader::{
            rep_preferences::RepresentationPreferences, ResourceReadRequest, ResourceReadResponse,
            ResourceReadTokenSet,
        },
        status_token_resolver::ResourceStatusTokenRequest,
    };

    mod seal {
        use crate::Repo;

        pub trait Sealed {}

        impl<R: Repo> Sealed for R {}
    }

    /// An extension trait for [`Repo`].
    pub trait RepoExt: Repo + seal::Sealed + Sized {
        /// Create a new instance of repo contextual of given type.
        #[inline]
        fn contextual<T: RepoContextual<Repo = Self>>(&self) -> T {
            T::new_with_context(self.context().clone())
        }

        /// Get uri policy of the repo.
        #[inline]
        fn uri_policy(&self) -> Self::UriPolicy {
            self.contextual()
        }

        /// Get rep patcher resolver of the repo.
        #[inline]
        fn rep_patcher_resolver(&self) -> RepoRepPatcherResolver<Self> {
            self.contextual()
        }

        /// Get resource status token resolver of the repo.
        #[inline]
        fn resource_status_token_resolver(&self) -> RepoResourceStatusTokenResolver<Self> {
            self.contextual()
        }

        /// Call associated [`RepoInitializer`] implementation.
        fn initialize(&self) -> ProbFuture<'static, bool> {
            let mut op = self.contextual::<RepoInitializerService<Self>>();
            Box::pin(async move { op.ready().await?.call(()).await })
        }

        /// Resolve resource status token.
        fn resolve_status_token(
            &self,
            res_uri: SolidResourceUri,
        ) -> ProbFuture<'static, RepoResourceStatusToken<Self>> {
            let context = self.context().clone();
            Box::pin(async move {
                Ok(
                    RepoResourceStatusTokenResolver::<Self>::new_with_context(context.clone())
                        .ready()
                        .and_then(|svc| {
                            svc.call(ResourceStatusTokenRequest {
                                resource_uri: res_uri.clone(),
                            })
                        })
                        .await?
                        .token,
                )
            })
        }

        /// Do basic read operation with given credentials.
        /// It calls reader with default params.
        fn read_basic_with_token(
            res_token: RepoRepresentedResourceToken<Self>,
            credentials: Self::Credentials,
            rep_preferences: RepresentationPreferences,
        ) -> ProbFuture<'static, ResourceReadResponse<Self, Self::Representation>> {
            Box::pin(async move {
                RepoResourceReader::<Self>::default()
                    .ready()
                    .and_then(|svc| {
                        svc.call(ResourceReadRequest {
                            tokens: ResourceReadTokenSet::new(res_token),
                            credentials,
                            preconditions: Box::new(()),
                            rep_conneg_params: Default::default(),
                            rep_preferences,
                            extensions: Default::default(),
                        })
                    })
                    .await
            })
        }

        /// Do basic read operation with given credentials.
        /// It first resolves status token.If resource is
        /// represented, then calls reader with default params.
        fn read_basic(
            &self,
            res_uri: SolidResourceUri,
            credentials: Self::Credentials,
            rep_preferences: RepresentationPreferences,
        ) -> ProbFuture<'static, Option<ResourceReadResponse<Self, Self::Representation>>> {
            let status_token_fut = self.resolve_status_token(res_uri.clone());

            Box::pin(async move {
                // Resolve the status token.
                let status_token: RepoResourceStatusToken<Self> = status_token_fut
                    .inspect_err(|e| {
                        error!("Error in resolving resource status token. Error:\n {}", e)
                    })
                    .await?;

                let er_token = if let Some(v) = status_token.existing_represented() {
                    v
                } else {
                    info!("Resource is not represented.");
                    return Ok(None);
                };

                Self::read_basic_with_token(er_token, credentials, rep_preferences)
                    .map_ok(Some)
                    .await
            })
        }
    }

    impl<R: Repo> RepoExt for R {}
}

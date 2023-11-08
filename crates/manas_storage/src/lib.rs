//! This crate provides traits and implementations for
//! `SolidStorage`, and `SolidStorageService`, (a
//! solid-protocol compatible http service over a storage).
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::sync::Arc;

use dyn_problem::Problem;
use futures::future::BoxFuture;
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::{
    context::RepoContext,
    service::resource_operator::common::status_token::ResourceStatusTokenTypes, Repo,
    RepoRepPatcher, RepoResourceCreator, RepoResourceDeleter, RepoResourceReader,
    RepoResourceState, RepoResourceStatusToken, RepoResourceStatusTokenTypes, RepoResourceUpdater,
};
use manas_space::SolidStorageSpace;
use name_locker::NameLocker;
use policy::method::MethodPolicy;

pub mod policy;
pub mod service;

/// A trait for solid storages.
pub trait SolidStorage: Send + Sync + 'static + Unpin {
    /// Type of the storage space.
    type StSpace: SolidStorageSpace;

    /// Type of method policy.
    type MethodPolicy: MethodPolicy;

    /// Type of the repo.
    type Repo: Repo<StSpace = Self::StSpace, Representation = BinaryRepresentation>;

    /// Type of resource locker, used to manage concurrent
    /// access to individual resources.
    type ResourceLocker: NameLocker<Name = String>;

    /// Get method policy.
    fn method_policy(&self) -> &Self::MethodPolicy;

    /// Get a reference to repo.
    fn repo(&self) -> &Self::Repo;

    /// Get resource locker.
    fn resource_locker(&self) -> &Self::ResourceLocker;

    /// Get any extensions.
    fn extensions(&self) -> &http::Extensions;

    /// Initialize the storage.
    fn initialize(&self) -> BoxFuture<'static, Result<(), Problem>>;
}

/// Alias for type of storage's space.
pub type SgStSpace<Storage> = <Storage as SolidStorage>::StSpace;

/// Alias for type of repo of a storage.
pub type SgRepo<Storage> = <Storage as SolidStorage>::Repo;

/// Type of storage credentials.
pub type SgCredentials<Storage> = <SgRepo<Storage> as Repo>::Credentials;

/// Alias for type of resource status token types of
/// storage's repo.
pub type SgResourceStatusTokenTypes<Storage> = RepoResourceStatusTokenTypes<SgRepo<Storage>>;

/// Alias for type of resource status tokens of storage's
/// repo.
pub type SgResourceStatusToken<Storage> = RepoResourceStatusToken<SgRepo<Storage>>;

/// Alias for resource non-existing--mutex-non-existing
/// status token type.
pub type SgResourceConflictFreeToken<Storage> =
    <SgResourceStatusTokenTypes<Storage> as ResourceStatusTokenTypes>::NonExistingMutexNonExisting;

/// Alias for type of repo context of a storage.
pub type SgRepoContext<Storage> = <<Storage as SolidStorage>::Repo as Repo>::Context;

/// Alias for type of rep patcher used by repo of the storage.
pub type SgRepPatcher<Storage> = RepoRepPatcher<SgRepo<Storage>>;

/// Alias for type of resource state of storage's repo.
pub type SgResourceState<Storage> = RepoResourceState<SgRepoContext<Storage>>;

/// Alias for type of the storage's repo resource reader.
pub type SgResourceReader<Storage> = RepoResourceReader<SgRepo<Storage>>;

/// Alias for type of the storage's repo resource creator.
pub type SgResourceCreator<Storage> = RepoResourceCreator<SgRepo<Storage>>;

/// Alias for type of the storage's repo resource deleter.
pub type SgResourceDeleter<Storage> = RepoResourceDeleter<SgRepo<Storage>>;

/// Alias for type of the storage's repo resource updater.
pub type SgResourceUpdater<Storage> = RepoResourceUpdater<SgRepo<Storage>>;

mod seal {
    use crate::SolidStorage;

    pub trait Sealed {}

    impl<S: SolidStorage> Sealed for S {}
}

/// An extension trait for [`SolidStorage`].
pub trait SolidStorageExt: SolidStorage + seal::Sealed {
    /// Get storage space pf this service.
    #[inline]
    fn space(&self) -> &Arc<Self::StSpace> {
        self.repo().context().storage_space()
    }
}

impl<S: SolidStorage> SolidStorageExt for S {}

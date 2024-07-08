//! I define traits to define solid compliant storage services.
//!

use std::sync::Arc;

use dyn_problem::Problem;
use futures::future::BoxFuture;
use manas_http::{body::Body, service::namespaced::NamespacedHttpService};
use tower::Service;

use crate::SolidStorage;

pub mod cors;
pub mod impl_;
pub mod method;

/// A trait for solid compliant storage services.
pub trait SolidStorageService: NamespacedHttpService<Body, Body> + StorageInitializer {
    /// Type of the storage, this service serves.
    type Storage: SolidStorage;

    /// Get storage of this service.
    fn storage(&self) -> &Arc<Self::Storage>;
}

/// A [`SolidStorageServiceFactory`] resolves storage service for each storage.
pub trait SolidStorageServiceFactory: Clone + Send + Sync + Unpin + 'static {
    /// Type of storage
    type Storage: SolidStorage;

    /// Type of storage service
    type Service: SolidStorageService<Storage = Self::Storage>;

    /// Get a new service for given storage.
    fn new_service(&self, storage: Arc<Self::Storage>) -> Self::Service;
}

/// A contract trait for storage initializer.
///
/// Service must ensure storage root is existing and represented.
/// Service must be idempotent.
/// It should return Ok(false), if it is already initialized.
pub trait StorageInitializer:
    Service<(), Response = bool, Error = Problem, Future = BoxFuture<'static, Result<bool, Problem>>>
{
}

impl<S> StorageInitializer for S where
    S: Service<
        (),
        Response = bool,
        Error = Problem,
        Future = BoxFuture<'static, Result<bool, Problem>>,
    >
{
}

//! This crate provides default repository implementation on
//! top of [OpenDAL](https://docs.rs/opendal/latest/opendal/)
//! object store abstraction layer.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{marker::PhantomData, sync::Arc};

use context::ODRContext;
use manas_authentication::common::credentials::impl_::void::VoidCredentials;
use manas_http::representation::impl_::binary::BinaryRepresentation;
use manas_repo::{
    service::patcher_resolver::impl_::{UnsupportedRepPatcher, UnsupportedRepPatcherResolver},
    Repo, RepoServices,
};
use policy::uri::ODRUriPolicy;
pub use service::resource_operator::common::status_token;
use service::{
    initializer::ODRInitializer,
    resource_operator::{
        common::status_token::ODRResourceStatusTokenTypes, creator::ODRResourceCreator,
        deleter::ODRResourceDeleter, reader::ODRResourceReader,
        status_token_resolver::ODRResourceStatusTokenResolver, updater::ODRResourceUpdater,
    },
};
use setup::ODRSetup;

pub mod config;
pub mod context;
pub mod object_store;
pub mod policy;
pub mod resource_context;
pub mod service;
pub mod setup;

mod util;

/// An implementation of [`Repo`] on top of
/// [OpenDAL](https://docs.rs/opendal/latest/opendal/) object
/// store abstraction layer.
pub struct OpendalRepo<Setup: ODRSetup> {
    /// Context of the repo.
    pub context: Arc<ODRContext<Setup>>,
}

impl<Setup: ODRSetup> Clone for OpendalRepo<Setup> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

impl<Setup: ODRSetup> std::fmt::Debug for OpendalRepo<Setup> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpendalRepo").finish()
    }
}

impl<Setup: ODRSetup> Repo for OpendalRepo<Setup> {
    type StSpace = Setup::StSpace;

    type Representation = BinaryRepresentation;

    type Context = ODRContext<Setup>;

    type UriPolicy = ODRUriPolicy<Setup>;

    type ResourceStatusTokenTypes = ODRResourceStatusTokenTypes<Setup>;

    type RepPatcher = UnsupportedRepPatcher;

    type Services = ODRServices<Setup>;

    type Credentials = VoidCredentials;

    #[inline]
    fn context(&self) -> &Arc<Self::Context> {
        &self.context
    }

    #[inline]
    fn new(context: Arc<Self::Context>) -> Self {
        Self { context }
    }
}

/// An implementation of [`RepoServices`] for odr.
pub struct ODRServices<Setup> {
    _phantom: PhantomData<fn(Setup)>,
}

impl<Setup: ODRSetup> RepoServices for ODRServices<Setup> {
    type Repo = OpendalRepo<Setup>;

    type Initializer = ODRInitializer<Setup>;

    type RepPatcherResolver = UnsupportedRepPatcherResolver<OpendalRepo<Setup>>;

    type ResourceStatusTokenResolver = ODRResourceStatusTokenResolver<Setup>;

    type ResourceReader = ODRResourceReader<Setup>;

    type ResourceCreator = ODRResourceCreator<Setup>;

    type ResourceUpdater = ODRResourceUpdater<Setup>;

    type ResourceDeleter = ODRResourceDeleter<Setup>;
}

// /// I define few utils to mock with [`OpendalRepo`].
// #[cfg(feature = "test-utils")]
// pub mod mock {
//     use crate::{setup::mock::MockODRSetup, OpendalRepo};

//     /// A type alias for [`OpendalRepo`] with moc setup.
//     pub type MockOpendalRepo<const MAX_AUX_LINKS: usize> = OpendalRepo<MockODRSetup<MAX_AUX_LINKS>>;
// }

//! I define message types for [`ResourceDeleter`](super::ResourceDeleter).
//!

use manas_space::resource::{slot::SolidResourceSlot, slot_link::AuxLink};
use typed_record::ClonableTypedRecord;

use crate::{
    layer::LayeredRepo,
    service::resource_operator::common::{
        preconditions::Preconditions, status_token::impl_::layered::Layered,
    },
    Repo, RepoRepresentedResourceToken,
};

/// A type to represent set of tokens required for resource
/// deletion.
#[derive(Debug)]
pub struct ResourceDeleteTokenSet<R: Repo> {
    /// Represented status token of the resource.
    pub res_token: RepoRepresentedResourceToken<R>,
}

impl<R: Repo> ResourceDeleteTokenSet<R> {
    /// Get a new [`ResourceDeleteTokenSet`] from given params.
    #[inline]
    pub fn new(res_token: RepoRepresentedResourceToken<R>) -> Self {
        Self { res_token }
    }
}

/// A struct to represent resource delete request.
#[derive(Debug)]
pub struct ResourceDeleteRequest<R: Repo> {
    /// Required token set.
    pub tokens: ResourceDeleteTokenSet<R>,

    /// Preconditions for the operation against the resource.
    pub preconditions: Box<dyn Preconditions>,

    /// Request credentials.
    pub credentials: R::Credentials,

    /// Any extensions.
    pub extensions: ClonableTypedRecord,
}

impl<R: Repo> ResourceDeleteRequest<R> {
    /// Map the tokens in the request.
    #[inline]
    pub fn map_tokens<R2, F>(self, f: F) -> ResourceDeleteRequest<R2>
    where
        R2: Repo,
        F: FnOnce(ResourceDeleteTokenSet<R>) -> ResourceDeleteTokenSet<R2>,
        R::Credentials: Into<R2::Credentials>,
    {
        ResourceDeleteRequest {
            tokens: f(self.tokens),
            preconditions: self.preconditions,
            credentials: self.credentials.into(),
            extensions: self.extensions,
        }
    }

    /// Unlayer the tokens.
    pub fn unlayer_tokens<IR>(self) -> ResourceDeleteRequest<IR>
    where
        IR: Repo,
        R: LayeredRepo<IR>,
        R::Credentials: Into<IR::Credentials>,
    {
        self.map_tokens(|tokens| Layered::from(tokens).inner)
    }
}

/// A struct to represent resource delete response.
#[derive(Debug)]
pub struct ResourceDeleteResponse<R: Repo> {
    /// Deleted resource slot.
    pub deleted_res_slot: SolidResourceSlot<R::StSpace>,

    /// Deleted aux resource links.
    pub deleted_aux_res_links: Vec<AuxLink<R::StSpace>>,

    /// Extensions.
    pub extensions: http::Extensions,
}

impl<R: Repo> ResourceDeleteResponse<R> {
    /// Map repo.
    #[inline]
    pub fn map_repo<R2: Repo<StSpace = R::StSpace>>(self) -> ResourceDeleteResponse<R2> {
        ResourceDeleteResponse {
            deleted_res_slot: self.deleted_res_slot,
            deleted_aux_res_links: self.deleted_aux_res_links,
            extensions: self.extensions,
        }
    }
}

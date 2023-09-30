//! I define message types for [`ResourceUpdater`](super::ResourceUpdater).
//!

use typed_record::ClonableTypedRecord;

use crate::{
    layer::LayeredRepo,
    service::resource_operator::common::{
        preconditions::Preconditions, rep_update_action::RepUpdateAction,
        status_token::impl_::layered::Layered,
    },
    Repo, RepoExistingResourceToken,
};

/// A type to represent set of tokens required for resource
/// update operation.
#[derive(Debug)]
pub struct ResourceUpdateTokenSet<R: Repo> {
    /// Existing status token of the resource.
    pub res_token: RepoExistingResourceToken<R>,
}

impl<R: Repo> ResourceUpdateTokenSet<R> {
    /// Get a new [`ResourceUpdateTokenSet`] from given params.
    #[inline]
    pub fn new(res_token: RepoExistingResourceToken<R>) -> Self {
        Self { res_token }
    }
}

/// A struct to represent resource update request.
#[derive(Debug)]
pub struct ResourceUpdateRequest<R: Repo> {
    /// Required token set.
    pub tokens: ResourceUpdateTokenSet<R>,

    /// Representation update action.
    pub rep_update_action: RepUpdateAction<R>,

    /// Preconditions for the operation against the resource.
    pub preconditions: Box<dyn Preconditions>,

    /// Request credentials.
    pub credentials: R::Credentials,

    /// Any extensions.
    pub extensions: ClonableTypedRecord,
}

impl<R: Repo> ResourceUpdateRequest<R> {
    /// Map the tokens in the request.
    #[inline]
    pub fn map_tokens<R2, F>(self, f: F) -> ResourceUpdateRequest<R2>
    where
        R2: Repo,
        F: FnOnce(ResourceUpdateTokenSet<R>) -> ResourceUpdateTokenSet<R2>,
        R::Representation: Into<R2::Representation>,
        R::RepPatcher: Into<R2::RepPatcher>,
        R::Credentials: Into<R2::Credentials>,
    {
        ResourceUpdateRequest {
            tokens: f(self.tokens),
            rep_update_action: self.rep_update_action.map_repo(),
            preconditions: self.preconditions,
            credentials: self.credentials.into(),
            extensions: self.extensions,
        }
    }

    /// Unlayer the tokens.
    pub fn unlayer_tokens<IR>(self) -> ResourceUpdateRequest<IR>
    where
        IR: Repo,
        R: LayeredRepo<IR>,
        R::Representation: Into<IR::Representation>,
        R::RepPatcher: Into<IR::RepPatcher>,
        R::Credentials: Into<IR::Credentials>,
    {
        self.map_tokens(|tokens| Layered::from(tokens).inner)
    }
}

/// A struct to represent resource update response.
#[derive(Debug)]
pub struct ResourceUpdateResponse {
    /// Extensions.
    pub extensions: http::Extensions,
}

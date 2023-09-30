//! I define message types for [`ResourceStateResolver`](super:ResourceStateResolver).
//!

use std::sync::Arc;

use manas_http::{header::accept::Accept, representation::Representation};
use manas_space::resource::{slot_link::AuxLink, state::invariant::RepresentedSolidResourceState};
use typed_record::ClonableTypedRecord;

use self::rep_preferences::RepresentationPreferences;
use crate::{
    layer::LayeredRepo,
    service::resource_operator::common::{
        preconditions::Preconditions, status_token::impl_::layered::Layered,
    },
    Repo, RepoRepresentedResourceToken,
};

pub mod rep_preferences;

/// A type to represent set of tokens required for resource
/// read operation.
#[derive(Debug)]
pub struct ResourceReadTokenSet<R: Repo> {
    /// Represented status token of the resource.
    pub res_token: RepoRepresentedResourceToken<R>,
}

impl<R: Repo> ResourceReadTokenSet<R> {
    /// Get a new [`ResourceReadTokenSet`] from given params.
    #[inline]
    pub fn new(res_token: RepoRepresentedResourceToken<R>) -> Self {
        Self { res_token }
    }
}

/// Params for content negotiation.
// TODO include other conneg headers.
// TODO support conneg for profile.
#[derive(Debug, Clone, Default)]
pub struct ConnegParams {
    /// Accept conneg param.
    pub accept: Option<Accept>,
}

/// A struct to represent resource read request.
#[derive(Debug)]
pub struct ResourceReadRequest<R: Repo> {
    /// Required token set.
    pub tokens: ResourceReadTokenSet<R>,

    /// Representation preferences.
    pub rep_preferences: RepresentationPreferences,

    /// Content negotiation params.
    pub rep_conneg_params: ConnegParams,

    /// Preconditions for the operation against the target
    /// resource.
    pub preconditions: Box<dyn Preconditions>,

    /// Request credentials.
    pub credentials: R::Credentials,

    /// Any extensions.
    pub extensions: ClonableTypedRecord,
}

impl<R: Repo> ResourceReadRequest<R> {
    /// Map the tokens in the request.
    #[inline]
    pub fn map_tokens<R2, F>(self, f: F) -> ResourceReadRequest<R2>
    where
        R2: Repo,
        F: FnOnce(ResourceReadTokenSet<R>) -> ResourceReadTokenSet<R2>,
        R::Credentials: Into<R2::Credentials>,
    {
        ResourceReadRequest {
            tokens: f(self.tokens),
            rep_preferences: self.rep_preferences,
            rep_conneg_params: self.rep_conneg_params,
            preconditions: self.preconditions,
            credentials: self.credentials.into(),
            extensions: self.extensions,
        }
    }

    /// Unlayer the tokens.
    pub fn unlayer_tokens<IR>(self) -> ResourceReadRequest<IR>
    where
        IR: Repo,
        R: LayeredRepo<IR>,
        R::Credentials: Into<IR::Credentials>,
    {
        self.map_tokens(|tokens| Layered::from(tokens).inner)
    }
}

/// A struct to represent resource read response.
#[derive(Debug)]
pub struct ResourceReadResponse<R, Rep>
where
    R: Repo,
    Rep: Representation,
{
    /// Resolved resource state.
    pub state: RepresentedSolidResourceState<R::StSpace, Rep>,

    /// Aux links index.
    pub aux_links_index: Vec<AuxLink<R::StSpace>>,

    /// Tokens for further actions.
    pub tokens: ResourceReadTokenSet<R>,

    /// Any extensions.
    pub extensions: http::Extensions,
}

impl<R, Rep> ResourceReadResponse<R, Rep>
where
    R: Repo,
    Rep: Representation,
{
    /// Unlayer tokens.
    #[inline]
    pub fn layer_tokens<LR>(self, layer_context: Arc<LR::Context>) -> ResourceReadResponse<LR, Rep>
    where
        LR: LayeredRepo<R>,
        LR::Credentials: Into<R::Credentials>,
    {
        ResourceReadResponse {
            state: self.state,
            aux_links_index: self.aux_links_index,
            tokens: Layered::new(self.tokens, layer_context).into(),
            extensions: self.extensions,
        }
    }

    /// Unlayer tokens.
    #[inline]
    pub fn unlayer_tokens<IR>(self) -> ResourceReadResponse<IR, Rep>
    where
        IR: Repo,
        R: LayeredRepo<IR>,
        R::Credentials: Into<IR::Credentials>,
    {
        ResourceReadResponse {
            state: self.state,
            aux_links_index: self.aux_links_index,
            tokens: Layered::from(self.tokens).inner,
            extensions: self.extensions,
        }
    }

    /// Map the resource state.
    pub fn map_state<Rep2, F>(self, f: F) -> ResourceReadResponse<R, Rep2>
    where
        Rep2: Representation,
        F: FnOnce(
            RepresentedSolidResourceState<R::StSpace, Rep>,
        ) -> RepresentedSolidResourceState<R::StSpace, Rep2>,
    {
        ResourceReadResponse {
            state: f(self.state),
            aux_links_index: self.aux_links_index,
            tokens: self.tokens,
            extensions: self.extensions,
        }
    }

    /// Map the representation.
    #[inline]
    pub fn map_representation<Rep2, F>(self, f: F) -> ResourceReadResponse<R, Rep2>
    where
        F: FnOnce(Rep) -> Rep2,
        Rep2: Representation,
    {
        self.map_state(|st| st.map_representation(f))
    }
}

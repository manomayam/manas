//! I define service trait for resource status token resolvers.
//!

use std::fmt::Debug;

use dyn_problem::{ProbFuture, Problem};
use manas_http::uri::invariant::NormalAbsoluteHttpUri;
use tower::Service;

use crate::{Repo, RepoContextual, RepoResourceStatusToken};

pub mod impl_;

/// Resource status token request.
#[derive(Debug, Clone)]
pub struct ResourceStatusTokenRequest {
    /// Resource uri.
    pub resource_uri: NormalAbsoluteHttpUri,
}

/// Resource status token response.
#[derive(Debug)]
pub struct ResourceStatusTokenResponse<R: Repo> {
    /// Status token.
    pub token: RepoResourceStatusToken<R>,
}

/// A trait for resource status token resolvers.
/// ## Operation contract:
///
/// - Service must return [`ResourceStatusTokenResponse`]
/// corresponding to resource uri in the request.
///
/// - Through out it's lifetime it MUST maintain same shared
/// repo context pointer.
pub trait ResourceStatusTokenResolver:
    Service<
        ResourceStatusTokenRequest,
        Response = ResourceStatusTokenResponse<Self::Repo>,
        Error = Problem,
        Future = ProbFuture<'static, ResourceStatusTokenResponse<Self::Repo>>,
    > + RepoContextual
    + Send
    + Sized
    + Clone
    + Debug
    + 'static
{
}

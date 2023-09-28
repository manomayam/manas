//! I define traits for defining http challenge-response authn-schemes.
//!

use std::fmt::Debug;

use dyn_problem::Problem;
use either::Either;
use futures::future::BoxFuture;
use http::{HeaderMap, Method};
use http_uri::invariant::AbsoluteHttpUri;
use manas_http::header::www_authenticate::WWWAuthenticate;

use crate::common::credentials::RequestCredentials;

pub mod impl_;

/// A trait for representing challenge-response authentication scheme.
pub trait CRAuthenticationScheme: Debug + Send + Sync + 'static {
    /// Type of request credentials.
    type Credentials: RequestCredentials;

    /// Resolve the credentials or challenge.
    fn resolve_or_challenge(
        &self,
        uri: &AbsoluteHttpUri,
        method: &Method,
        headers: &HeaderMap,
    ) -> BoxFuture<'static, CRResolutionResult<Self::Credentials>>;
}

/// A type for representing challenge in challenge-response auth framework.
#[derive(Debug, Clone)]
pub struct CRAuthenticationChallenge {
    /// WWW-Authenticate header.
    pub www_authenticate: WWWAuthenticate,

    /// Any other serialized headers.
    pub ext_headers: HeaderMap,
}

/// An alias for authentication-scheme trait object.
pub type DynCRAuthenticationScheme<C> = dyn CRAuthenticationScheme<Credentials = C>;

/// An alias for challenge-response auth resolution result.
pub type CRResolutionResult<Creds> = Result<Creds, Either<CRAuthenticationChallenge, Problem>>;

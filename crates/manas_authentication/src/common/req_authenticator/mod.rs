//! I define trait for request authenticators.
//!

use std::fmt::Debug;

use http::Request;

use super::credentials::RequestCredentials;

pub mod impl_;

/// A trait for request authenticators.
/// An authenticator takes request and resolved credentials,
/// and returns authenticated request as per it's policy.
pub trait RequestAuthenticator: Debug + Send + Sync + 'static {
    /// Type of credentials.
    type Credentials: RequestCredentials;

    /// Resolve authenticated request from raw request and
    /// resolved credentials.
    #[inline]
    fn authenticated<B>(mut req: Request<B>, credentials: Self::Credentials) -> Request<B> {
        req.extensions_mut().insert(credentials);
        req
    }
}

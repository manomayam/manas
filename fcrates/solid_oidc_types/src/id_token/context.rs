//! I define types to represent context of an id-token.
//!

use picky::jose::jwk::JwkSet;

/// A type for representing context of an id-token.
// TODO improve context to implement <https://openid.net/specs/openid-connect-core-1_0.html#IDTokenValidation>.
#[derive(Debug, Clone)]
pub struct IdTokenContext {
    /// Current time.
    pub current_time: u64,

    /// Oidc issuer jwks.
    pub issuer_jwks: JwkSet,
    // Todo max age
}

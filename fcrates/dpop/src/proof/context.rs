//! I define types to represent context of a dpop-proof.
//!

use std::time::{SystemTime, UNIX_EPOCH};

use http::{Method, Request};
use http_uri::{HttpUri, InvalidHttpUri};

use super::payload::{jkt::Jkt, nonce::Nonce};

/// A type for representing context of a dpop-proof.
pub struct DPoPProofContext {
    /// Method of the http request in which the dpop-proof
    /// JWT was received.
    pub req_method: Method,

    /// HTTP URI value for the HTTP request in which the
    /// dpop-proof JWT was received.
    pub req_uri: HttpUri,

    /// Time of the request reception as jwt `NumericDate`.
    pub req_time: i64,

    /// Current active nonce value, if any is provided by the
    /// server to the client.
    pub active_nonce: Option<Nonce>,

    /// Optional server managed timestamp via the nonce claim.
    pub nonce_timestamp: Option<i64>,

    /// Leeway for time comparisons.
    pub time_leeway: u16,

    /// Key bound access token, if presented to a protected
    /// resource in conjunction.with.
    pub key_bound_access_token: Option<KeyBoundAccessToken>,
}

/// A struct for representing a key bound access token.
#[derive(Debug, Clone)]
pub struct KeyBoundAccessToken {
    /// Access token
    pub access_token: String,

    /// Jkt of the public key to which access token is bound to.
    pub bound_key_jkt: Jkt,
}

/// Constant for default time leeway.
const DEFAULT_TIME_LEEWAY: u16 = 240;

impl DPoPProofContext {
    /// Glean the context from request and given key-bound-access-token.
    /// NOTE: Authorization header will be ignored whole gleaning,
    /// in favour of provided key-bound-access-token.
    pub fn glean<B>(
        req: &Request<B>,
        key_bound_access_token: Option<KeyBoundAccessToken>,
    ) -> Result<Self, DPoPProofContextGleanError> {
        Ok(Self {
            req_method: req.method().clone(),
            req_uri: HttpUri::try_from(req.uri().to_string().as_str())?,
            req_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            active_nonce: None,
            nonce_timestamp: None,
            time_leeway: DEFAULT_TIME_LEEWAY,
            key_bound_access_token,
        })
    }
}

/// A type for representing errors in gleaning dpop-proof context from inputs.
#[derive(Debug, thiserror::Error)]
pub enum DPoPProofContextGleanError {
    /// Invalid request uri.
    #[error("Invalid request uri.\n{0}")]
    InvalidRequestUri(#[from] InvalidHttpUri),
}

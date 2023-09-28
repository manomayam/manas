//! I define type for defining dpop-proof payload.
//!

use std::collections::HashMap;

use http::Method;
use serde::{Deserialize, Serialize};

use self::{ath::Ath, htu::Htu, nonce::Nonce};

pub mod ath;
pub mod htu;
pub mod jkt;
pub mod jti;
pub mod nonce;

pub mod common;

/// A struct representing the payload claims of dpop-proof jwt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPoPProofClaims {
    /// Unique identifier for the DPoP proof JWT.
    pub jti: String,

    /// The value of the HTTP method of the request to which the JWT is attached.
    #[serde(with = "http_serde::method")]
    pub htm: Method,

    /// The HTTP target URI, without
    /// query and fragment parts, of the request to which the JWT is attached.
    pub htu: Htu,

    /// Creation timestamp of the JWT.
    pub iat: i64,

    /// Hash of the access token.
    ///
    /// > When the DPoP proof is used in conjunction with the presentation
    /// > of an access token in protected resource access, see Section 7,
    /// > the DPoP proof MUST also contain the ath claim:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ath: Option<Ath>,

    /// A recent nonce provided via the DPoP-Nonce HTTP header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Nonce>,

    /// Any additional claims.
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

//! I define a jwt codec that handles encoding and decoding
//! of id-token compact jwts.
//!

use std::borrow::Cow;

use picky::{
    jose::jws::{Jws, JwsError},
    key::PrivateKey,
};

use super::{
    essence::{IdTokenEssence, InvalidIdTokenJws},
    raw::RawIdToken,
};

/// A struct representing a jwt codec that handles encoding and decoding of id-token compact jwts.
pub struct IdTokenJwtCodec {}

impl IdTokenJwtCodec {
    /// Encode the given proof essence as compact jwt using given private key for signing.
    pub fn encode(essence: IdTokenEssence, private_key: &PrivateKey) -> Result<String, JwsError> {
        let jws: Jws = essence.into();
        jws.encode(private_key)
    }

    /// Decode id-token that is encoded in given compact jwt.
    ///
    /// NOTE: This doesn't perform any kind of non-structural validation against id-token.
    #[inline]
    pub fn decode(encoded_token: Cow<'_, str>) -> Result<RawIdToken<'_>, IdTokenJwtDecodeError> {
        RawIdToken::decode(encoded_token)
    }
}

/// Type for representing errors in decoding id-token jwts.
#[derive(Debug, thiserror::Error)]
pub enum IdTokenJwtDecodeError {
    /// Invalid encoded jws.
    #[error("Invalid encoded jws.\n{0}")]
    InvalidEncodedJws(#[from] JwsError),

    /// Invalid id-token jws.
    #[error("Invalid id-token jws.\n{0}")]
    InvalidIdTokenJws(#[from] InvalidIdTokenJws),
}

//! I define a jwt codec that handles encoding and decoding of dpop-proof compact jwts.
//!

use std::borrow::Cow;

use picky::{
    jose::jws::{Jws, JwsError},
    key::PrivateKey,
};

use super::{
    essence::{DPoPProofEssence, InvalidDPoPProofJws},
    raw::RawDPoPProof,
};

/// A struct representing a jwt codec that handles encoding and decoding of dpop-proof compact jwts.
pub struct DPoPProofJwtCodec {}

impl DPoPProofJwtCodec {
    /// Encode the given proof essence as compact jwt using given private key for signing.
    pub fn encode(essence: DPoPProofEssence, private_key: &PrivateKey) -> Result<String, JwsError> {
        let jws: Jws = essence.into();
        jws.encode(private_key)
    }

    /// Decode dpop-proof that is encoded in given compact jwt.
    ///
    /// NOTE: This doesn't perform any kind of non-structural validation against dpop-proof.
    #[inline]
    pub fn decode(
        encoded_token: Cow<'_, str>,
    ) -> Result<RawDPoPProof<'_>, DPoPProofJwtDecodeError> {
        RawDPoPProof::decode(encoded_token)
    }
}

/// Type for representing errors in decoding dpop-proof jwts.
#[derive(Debug, thiserror::Error)]
pub enum DPoPProofJwtDecodeError {
    /// Invalid encoded jws.
    #[error("Invalid encoded jws.\n{0}")]
    InvalidEncodedJws(#[from] JwsError),

    /// Invalid dpop-proof jws.
    #[error("Invalid dpop-proof jws.\n{0}")]
    InvalidDPoPProofJws(#[from] InvalidDPoPProofJws),
}

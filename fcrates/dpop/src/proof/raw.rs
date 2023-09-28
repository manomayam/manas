//! I define types for representing not-yet-validated dpop-proofs.
//!

use std::borrow::Cow;

use picky::jose::jws::RawJws;

use super::{codec::DPoPProofJwtDecodeError, essence::DPoPProofEssence};

/// A struct to represent raw, not-yet-validated dpop proofs.
pub struct RawDPoPProof<'repr> {
    compact_repr: Cow<'repr, str>,

    /// Decoded essence.
    decoded_essence: DPoPProofEssence,

    /// Decoded signature.
    decoded_signature: Vec<u8>,
}

impl<'repr> RawDPoPProof<'repr> {
    /// Get compact representation of the dpop-proof.
    #[inline]
    pub fn compact_repr(&self) -> &str {
        &self.compact_repr
    }

    /// Convert into compact representation of the dpop-proof.
    #[inline]
    pub fn into_compact_repr(self) -> Cow<'repr, str> {
        self.compact_repr
    }

    /// Convert into decoded parts of the dpop-proof.
    /// NOTE: decoded parts are not validated.
    #[inline]
    pub fn into_decoded_parts(self) -> (DPoPProofEssence, Vec<u8>) {
        (self.decoded_essence, self.decoded_signature)
    }

    /// Get decoded essence of the dpop-proof.
    #[inline]
    pub fn decoded_essence(&self) -> &DPoPProofEssence {
        &self.decoded_essence
    }

    /// Get decoded signature of the dpop-proof.
    #[inline]
    pub fn decoded_signature(&self) -> &[u8] {
        &self.decoded_signature
    }

    /// Decode dpop-proof that is encoded in given compact jwt.
    ///
    /// NOTE: This doesn't perform any kind of non-structural validation against dpop-proof.
    pub fn decode(compact_repr: Cow<'repr, str>) -> Result<Self, DPoPProofJwtDecodeError> {
        // Decode raw jws.
        let mut raw_jws = RawJws::decode(compact_repr.as_ref())?;

        let decoded_signature = std::mem::take(&mut raw_jws.signature);

        // Get jws without signature check.
        let jws = raw_jws.discard_signature();

        // Parse dpop-proof essence from jws.
        Ok(RawDPoPProof {
            compact_repr,
            decoded_essence: jws.try_into()?,
            decoded_signature,
        })
    }
}

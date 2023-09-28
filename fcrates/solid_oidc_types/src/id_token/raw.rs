//! I define types for representing not-yet-validated id-tokens.
//!

use std::borrow::Cow;

use picky::jose::jws::RawJws;

use super::{codec::IdTokenJwtDecodeError, essence::IdTokenEssence};

/// A struct to represent raw, not-yet-validated id-tokens.
pub struct RawIdToken<'repr> {
    compact_repr: Cow<'repr, str>,

    /// Decoded essence.
    decoded_essence: IdTokenEssence,

    /// Decoded signature.
    decoded_signature: Vec<u8>,
}

impl<'repr> RawIdToken<'repr> {
    /// Get compact representation of the id-token.
    #[inline]
    pub fn compact_repr(&self) -> &str {
        &self.compact_repr
    }

    /// Convert into compact representation of the id-token.
    #[inline]
    pub fn into_compact_repr(self) -> Cow<'repr, str> {
        self.compact_repr
    }

    /// Convert into decoded parts of the id-token.
    /// NOTE: decoded parts are not validated.
    #[inline]
    pub fn into_decoded_parts(self) -> (IdTokenEssence, Vec<u8>) {
        (self.decoded_essence, self.decoded_signature)
    }

    /// Get decoded essence of the id-token.
    #[inline]
    pub fn decoded_essence(&self) -> &IdTokenEssence {
        &self.decoded_essence
    }

    /// Get decoded signature of the id-token.
    #[inline]
    pub fn decoded_signature(&self) -> &[u8] {
        &self.decoded_signature
    }

    /// Decode id-token that is encoded in given compact jwt.
    ///
    /// NOTE: This doesn't perform any kind of non-structural validation against id-token.
    pub fn decode(compact_repr: Cow<'repr, str>) -> Result<Self, IdTokenJwtDecodeError> {
        // Decode raw jws.
        let mut raw_jws = RawJws::decode(compact_repr.as_ref())?;

        let decoded_signature = std::mem::take(&mut raw_jws.signature);

        // Get jws without signature check.
        let jws = raw_jws.discard_signature();

        // Parse id-token essence from jws.
        Ok(RawIdToken {
            compact_repr,
            decoded_essence: jws.try_into()?,
            decoded_signature,
        })
    }
}

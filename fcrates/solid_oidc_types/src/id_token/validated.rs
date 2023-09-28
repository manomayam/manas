//! I define a type to represent id-token jwts that are validated against their context.
//!

use std::ops::Deref;

use gdp_rs::{proven::ProvenError, Proven};

pub use self::predicate::{InvalidIdToken, IsValidIdToken};
use super::{context::IdTokenContext, raw::RawIdToken};

/// Alias for type of context bounded id-tokens.
pub type ContextualIdToken = (RawIdToken<'static>, IdTokenContext);

/// A struct for representing id-tokens that are validated against a given context.
///
pub struct ValidatedIdToken(Proven<ContextualIdToken, IsValidIdToken>);

impl Deref for ValidatedIdToken {
    type Target = RawIdToken<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl ValidatedIdToken {
    /// Try to create a new [`ValidatedIdToken`] from given raw token and context.
    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn try_new(
        raw_token: RawIdToken<'static>,
        context: IdTokenContext,
    ) -> Result<Self, ProvenError<ContextualIdToken, InvalidIdToken>> {
        Ok(Self(Proven::try_new((raw_token, context))?))
    }

    /// Get dpop proof context.
    #[inline]
    pub fn context(&self) -> &IdTokenContext {
        &self.0 .1
    }
}

mod predicate {
    use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};
    use picky::{
        jose::{
            jwk::{JwkError, JwkKeyOps},
            jws::{verify_signature, JwsError},
        },
        signature::SignatureAlgorithm,
    };
    use tracing::error;

    use super::ContextualIdToken;

    /// A predicate over [`ContextualIdToken`] asserting it to be valid for known context.
    #[derive(Debug, Clone)]
    pub struct IsValidIdToken;

    impl Predicate<ContextualIdToken> for IsValidIdToken {
        fn label() -> std::borrow::Cow<'static, str> {
            "IsValidIdToken".into()
        }
    }

    impl SyncEvaluablePredicate<ContextualIdToken> for IsValidIdToken {
        type EvalError = InvalidIdToken;

        fn evaluate_for((raw_token, context): &ContextualIdToken) -> Result<(), Self::EvalError> {
            // Get decoded header and claims.
            let header = &raw_token.decoded_essence().header;
            let claims = &raw_token.decoded_essence().claims;

            // Check if expired.
            if context.current_time > claims.exp {
                error!("Id token is expired.");
                return Err(InvalidIdToken::IsExpired);
            }

            //> The audience claim MUST be an array of values.
            //> The values MUST include the authorized party claim azp and the string solid.
            if !(
                claims.aud.iter().any(|v| *v == "solid")
                // TODO MUST reenable following, after verifying with css team.
                // && claims.aud.contains(&claims.azp)
            ) {
                error!("Invalid aud claim.");
                return Err(InvalidIdToken::InvalidAudClaim);
            }

            // Ensure iat claim is not in future.
            if claims.iat > context.current_time {
                error!("iat claim is in future.");
                return Err(InvalidIdToken::InvalidIatClaim);
            }

            // Ensure alg is asymmetric and supported..
            if SignatureAlgorithm::try_from(header.alg).is_err() {
                error!("Unsupported jws sign algorithm. Alg: {:?}", header.alg);
                return Err(InvalidIdToken::UnsupportedAlg);
            }

            // Resolve issuer's jwk corresponding to it's public key used for signing.
            let mut issuer_keys = context.issuer_jwks.keys.iter();
            // If kid is configured, try find key with that kid.
            let issuer_jwk = if let Some(kid) = header.kid.as_deref() {
                issuer_keys.find(|jwk| jwk.kid.as_deref() == Some(kid))
            } else {
                // Else try find key with "verify" key_op.
                issuer_keys.find(|jwk| {
                    jwk.key_ops
                        .as_ref()
                        .map(|ops| ops.contains(&JwkKeyOps::Verify))
                        .unwrap_or(false)
                })
                // If no kid specified, get
            }
            .ok_or_else(|| {
                error!("Cannot find jwk with configured kid in issuer's jwks.");
                InvalidIdToken::UnresolvedIssuerPublicKey
            })?;

            // Get issuer public key from jwk.
            let issuer_public_key = issuer_jwk.to_public_key().map_err(|e| {
                error!("Invalid issuer public key jwk. Error:\n {}", e);
                InvalidIdToken::InvalidIssuerPublicKeyJwk(e)
            })?;

            // Verify signature.
            verify_signature(raw_token.compact_repr(), &issuer_public_key, header.alg)
                .map_err(InvalidIdToken::InvalidSignature)?;

            Ok(())
        }
    }

    impl PurePredicate<ContextualIdToken> for IsValidIdToken {}

    /// A type for representing errors of a id-token beng invalid.
    #[derive(Debug, thiserror::Error)]
    pub enum InvalidIdToken {
        /// Invalid aud claim.
        #[error("Invalid aud claim.")]
        InvalidAudClaim,

        /// Token is expired.
        #[error("Token is expired.")]
        IsExpired,

        /// Invalid iat claim. It is in future.
        #[error("Invalid iat claim. It is in future.")]
        InvalidIatClaim,

        /// Unsupported alg.
        #[error("Unsupported alg.")]
        UnsupportedAlg,

        /// Issuer public is not resolved.
        #[error("Issuer public is not resolved.")]
        UnresolvedIssuerPublicKey,

        /// Invalid public key jwk.
        #[error("Invalid issuer public key jwk.\n{0}")]
        InvalidIssuerPublicKeyJwk(#[from] JwkError),

        /// Invalid signature.
        #[error("Invalid signature.\n{0}")]
        InvalidSignature(JwsError),
    }
}

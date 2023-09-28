//! I define a type to represent dpop-proof jwts that are validated against their context.
//!

use std::ops::Deref;

use gdp_rs::{proven::ProvenError, Proven};

pub use self::predicate::{InvalidDPoPProof, IsValidDPoPProof};
use super::{context::DPoPProofContext, raw::RawDPoPProof};

/// Alias for type of context bounded dpop-proofs.
pub type ContextualDPoPProof = (RawDPoPProof<'static>, DPoPProofContext);

/// A struct for representing dpop-proofs that are validated against a given context.
///
pub struct ValidatedDPoPProof(Proven<ContextualDPoPProof, IsValidDPoPProof>);

impl Deref for ValidatedDPoPProof {
    type Target = RawDPoPProof<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl ValidatedDPoPProof {
    /// Try to create a new [`ValidatedDPoPProof`] from given raw proof and context.
    #[inline]
    #[allow(clippy::result_large_err)]
    pub fn try_new(
        raw_proof: RawDPoPProof<'static>,
        context: DPoPProofContext,
    ) -> Result<Self, ProvenError<ContextualDPoPProof, InvalidDPoPProof>> {
        Ok(Self(Proven::try_new((raw_proof, context))?))
    }

    /// Get dpop proof context.
    #[inline]
    pub fn context(&self) -> &DPoPProofContext {
        &self.0 .1
    }
}

mod predicate {
    use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};
    use picky::jose::{
        jwk::JwkError,
        jws::{verify_signature, JwsError},
    };

    use super::ContextualDPoPProof;
    use crate::proof::{
        context::KeyBoundAccessToken,
        payload::{ath::Ath, jkt::Jkt},
    };

    /// A predicate over [`ContextualDPoPProof`] asserting it to be valid.
    #[derive(Debug, Clone)]
    pub struct IsValidDPoPProof;

    impl Predicate<ContextualDPoPProof> for IsValidDPoPProof {
        fn label() -> std::borrow::Cow<'static, str> {
            "IsValidDPoPProof".into()
        }
    }

    impl SyncEvaluablePredicate<ContextualDPoPProof> for IsValidDPoPProof {
        type EvalError = InvalidDPoPProof;

        fn evaluate_for((raw_proof, context): &ContextualDPoPProof) -> Result<(), Self::EvalError> {
            let decoded_header = &raw_proof.decoded_essence().header;
            let decoded_claims = &raw_proof.decoded_essence().claims;

            // Get public key contained in `jwk` jose header field.
            let public_key = decoded_header.jwk().to_public_key()?;

            // Ensure that the JWT signature verifies with the contained public key.
            verify_signature(raw_proof.compact_repr(), &public_key, decoded_header.alg)
                .map_err(InvalidDPoPProof::InvalidSignature)?;

            // Ensure that the htm claim matches the HTTP method of the current request.
            if decoded_claims.htm != context.req_method {
                return Err(InvalidDPoPProof::HtmClaimMismatch);
            }

            // Ensure that the htu claim matches the HTTP URI value for the HTTP request
            if !decoded_claims.htu.matches(&context.req_uri) {
                return Err(InvalidDPoPProof::HtuClaimMismatch);
            }

            // Ensure that, if the server provided a nonce value to the client,
            // the nonce claim matches the server-provided nonce value,
            if context.active_nonce.is_some() && context.active_nonce != decoded_claims.nonce {
                return Err(InvalidDPoPProof::NonceClaimMismatch);
            }

            // Ensure that the creation time of the JWT, as determined by either the iat claim or
            // a server managed timestamp via the nonce claim,
            // is within an acceptable window,
            if (context.req_time - decoded_claims.iat).abs() >= context.time_leeway.into() {
                return Err(InvalidDPoPProof::TimestampOutOfWindow);
            }

            if let Some(nonce_timestamp) = context.nonce_timestamp {
                if (context.req_time - nonce_timestamp).abs() >= context.time_leeway.into() {
                    return Err(InvalidDPoPProof::TimestampOutOfWindow);
                }
            }

            // if presented to a protected resource in conjunction with an access token,
            if let Some(KeyBoundAccessToken {
                access_token,
                bound_key_jkt,
            }) = context.key_bound_access_token.as_ref()
            {
                let ath = Ath::new(access_token);

                let decoded_ath = decoded_claims
                    .ath
                    .as_ref()
                    // TODO MUST remove following feature and block.
                    .or_else(|| cfg!(feature = "unsafe-optional-ath-claim").then_some(&ath))
                    .ok_or(InvalidDPoPProof::AthClaimMismatch)?;

                // ensure that the value of the ath claim
                // equals the hash of that access token,
                if decoded_ath != &ath {
                    return Err(InvalidDPoPProof::AthClaimMismatch);
                }

                // confirm that the public key to which the access token is bound
                // matches the public key from the DPoP proof.
                if bound_key_jkt != &Jkt::new(&decoded_header.jwk()) {
                    return Err(InvalidDPoPProof::BindingKeyMisMatch);
                }
            }

            Ok(())
        }
    }

    impl PurePredicate<ContextualDPoPProof> for IsValidDPoPProof {}

    /// A type for representing errors of a dpop-proof beng invalid.
    #[derive(Debug, thiserror::Error)]
    pub enum InvalidDPoPProof {
        /// Invalid public key jwk.
        #[error("Invalid public key jwk.\n{0}")]
        InvalidPublicKeyJwk(#[from] JwkError),

        /// Invalid signature.
        #[error("Invalid signature.\n{0}")]
        InvalidSignature(JwsError),

        /// Htm claim mismatch.
        #[error("Htm claim mismatch.")]
        HtmClaimMismatch,

        /// Htu claim mismatch.
        #[error("Htu claim mismatch.")]
        HtuClaimMismatch,

        /// Nonce claim mismatch.
        #[error("Nonce claim mismatch.")]
        NonceClaimMismatch,

        /// Ath claim mismatch.
        #[error("Ath claim mismatch.")]
        AthClaimMismatch,

        /// Binding key mismatch.
        #[error("Binding key mismatch.")]
        BindingKeyMisMatch,

        /// Proof timestamp out of window.
        #[error("Proof timestamp out of window.")]
        TimestampOutOfWindow,
    }
}

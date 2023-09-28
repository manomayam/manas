//! I define a basic implementation of [`SolidOidcDpopSchemeSetup`].
//!

use std::marker::PhantomData;

use http_uri::security::transport_policy::SecureTransportPolicy;

use crate::challenge_response_framework::scheme::impl_::solid_oidc::{
    issuer_jwks::impl_::default::DefaultOidcIssuerJwksResolver, setup::SolidOidcDpopSchemeSetup,
    trusted_issuers::impl_::default::DefaultWebIdTrustedIssuersResolver,
};

/// A basic implementation of [`SolidOidcDpopSchemeSetup`].
#[derive(Debug)]
pub struct BasicSolidOidcDpopSchemeSetup<STP> {
    _phantom: PhantomData<fn(STP)>,
}

impl<STP: SecureTransportPolicy> SolidOidcDpopSchemeSetup for BasicSolidOidcDpopSchemeSetup<STP> {
    type SecureTransportPolicy = STP;

    type IssuerJwksResolver = DefaultOidcIssuerJwksResolver;

    type WebIdIssuersResolver = DefaultWebIdTrustedIssuersResolver;
}

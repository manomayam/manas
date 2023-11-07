//! I define few implementations of [`CRAuthenticationScheme`](super::CRAuthenticationScheme).
//!

pub mod union;

#[cfg(feature = "scheme-impl-solid-oidc")]
pub mod solid_oidc;

// #[cfg(feature = "scheme-impl-httpsig")]
// pub mod httpsig;

//! I define struct to represent `htu` dpop-proof claim.
//!

use std::ops::Deref;

use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};
use http_uri::{
    predicate::{has_no_query::HasNoQuery, is_absolute::IsAbsolute},
    HttpUri,
};
use serde::{Deserialize, Serialize};

/// A struct representing `htu` dpop-proof claim.
///
/// From spec:
///
/// >  htu: The HTTP target URI (Section 7.1 of RFC9110), without
/// query and fragment parts, of the request to which the JWT is attached.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[allow(clippy::type_complexity)]
pub struct Htu(Proven<HttpUri, AllOf<HttpUri, HList!(IsAbsolute, HasNoQuery)>>);

impl Deref for Htu {
    type Target = HttpUri;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl From<Htu> for HttpUri {
    #[inline]
    fn from(htu: Htu) -> Self {
        htu.0.into_subject()
    }
}

impl Htu {
    /// Check if the claim matches with http request uri, ignoring any query and fragment parts.
    /// It would employ syntax based and scheme based normalization
    /// as recommended by the spec.
    pub fn matches(&self, req_uri: &HttpUri) -> bool {
        // Normalize the request uri.
        let req_uri = if req_uri.is_http_normalized() {
            req_uri.clone()
        } else {
            req_uri.http_normalized()
        };

        req_uri.scheme_str() == self.scheme_str()
            && req_uri.authority_str() == self.authority_str()
            && req_uri.path_str() == self.path_str()
    }
}

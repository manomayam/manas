//! I define traits to define namespaced infallible http
//! services.
//!

use super::HttpService;
use crate::uri::invariant::NormalAbsoluteHttpUri;

/// A trait for infallible http services, which are defined against a bounded uri namespace.
pub trait NamespacedHttpService<ReqBody, ResBody>: HttpService<ReqBody, ResBody> {
    /// Check if given uri is in target namespace of this service.
    fn has_in_uri_ns(&self, uri: &NormalAbsoluteHttpUri) -> bool;
}

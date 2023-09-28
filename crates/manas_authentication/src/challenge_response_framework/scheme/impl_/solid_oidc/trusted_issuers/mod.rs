//! I define functionality to resolve webid trusted oidc issuers.
//!

use std::{fmt::Debug, sync::Arc};

use futures::future::BoxFuture;
use http_uri::invariant::AbsoluteHttpUri;
use tracing::error;
use webid::{profile_req_agent::ProfileDocResolutionError, WebId};

pub mod impl_;

/// A trait for webid trusted issuers resolvers.
pub trait WebIdTrustedIssuersResolver: Debug + Send + Sync + 'static {
    /// Resolve trusted issuers for the webid.
    fn resolve(
        &self,
        webid: WebId,
    ) -> BoxFuture<'_, Result<Vec<AbsoluteHttpUri>, Arc<WebIdTrustedIssuersResolutionError>>>;
}

/// An error type for representing errors in webid trusted issuers resolution.
#[derive(Debug, thiserror::Error)]
pub enum WebIdTrustedIssuersResolutionError {
    /// Error in profile doc resolution.
    #[error("Error in profile doc resolution.")]
    ProfileDocResolutionError(#[from] ProfileDocResolutionError),
}

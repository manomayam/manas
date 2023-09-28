//! I provide few snippets to deal with resource status token.
//!

use futures::TryFutureExt;
use http::StatusCode;
use http_api_problem::ApiError;
use manas_repo::{
    service::resource_operator::status_token_resolver::ResourceStatusTokenRequest, RepoExt,
};
use manas_space::resource::uri::SolidResourceUri;
use tower::{Service, ServiceExt};
use tracing::error;

use crate::{SgResourceStatusToken, SolidStorage};

/// Resolve resource status token.
pub async fn resolve_status_token<Storage: SolidStorage>(
    storage: &Storage,
    res_uri: SolidResourceUri,
) -> Result<SgResourceStatusToken<Storage>, ApiError> {
    Ok(storage
        .repo()
        .resource_status_token_resolver()
        .ready()
        .and_then(|svc| {
            svc.call(ResourceStatusTokenRequest {
                resource_uri: res_uri,
            })
        })
        .map_err(|e| {
            error!(
                "Unknown io error in resolving resource status token. Error:\n {}",
                e
            );
            ApiError::builder(StatusCode::INTERNAL_SERVER_ERROR).finish()
        })
        .await?
        .token)
}

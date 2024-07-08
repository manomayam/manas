//! I provide few common utils for recipe implementations.
//!

use std::{net::SocketAddr, sync::Arc};

use axum_server::{service::MakeService, tls_rustls::RustlsConfig};
use futures::TryFutureExt;
use http::{Method, Request};
use hyper::body::Incoming;
use manas_authentication::common::{
    credentials::impl_::basic::BasicRequestCredentials,
    req_authenticator::impl_::BasicRequestAuthenticator,
};
use manas_http::{
    body::Body,
    service::{
        adapter::AdaptIncomingBody,
        impl_::{NormalValidateTargetUri, ReconstructTargetUri, UriReconstructionParams},
        HttpService,
    },
};
use manas_storage::service::cors::LiberalCors;
use tower::{make::Shared, Layer};
use tower_http::catch_panic::CatchPanic;
use tracing::error;

use self::config::RcpServerConfig;

pub mod config;

/// Alias trait for [`MakeService`] with sendable futures.
pub trait SendMakeService:
    MakeService<SocketAddr, Request<Incoming>, MakeFuture: Send + 'static>
{
}

impl<S> SendMakeService for S where
    S: MakeService<SocketAddr, Request<Incoming>, MakeFuture: Send + 'static>
{
}

/// Resolve service maker for given podset service.
pub fn resolve_svc_maker(
    podset_svc: impl HttpService<Body, Body> + Clone,
    uri_reconstruction_params: UriReconstructionParams,
) -> impl SendMakeService {
    Shared::new(AdaptIncomingBody::new(CatchPanic::new(LiberalCors::new(
        ReconstructTargetUri::new(
            uri_reconstruction_params,
            NormalValidateTargetUri::new(podset_svc),
        ),
    ))))
}

/// Resolve service maker for given podset service.
#[cfg(feature = "layer-authentication")]
pub fn resolve_authenticating_svc_maker(
    podset_svc: impl HttpService<Body, Body> + Clone,
    uri_reconstruction_params: UriReconstructionParams,
) -> impl SendMakeService {
    resolve_svc_maker(manas_authentication::challenge_response_framework::service::HttpCRAuthenticationLayer::<
        _,
        _,
        Body,
        BasicRequestAuthenticator<BasicRequestCredentials>,
    >::new(
            manas_authentication::challenge_response_framework::scheme::impl_::solid_oidc::DefaultSolidOidcDpopScheme::default(),
            Arc::new(vec![
                Method::POST,
                Method::PATCH,
                // Method::PUT,
                Method::DELETE,
            ]),
        )
        .layer(podset_svc),
        uri_reconstruction_params
    )
}

/// Serve the recipe.
pub async fn serve_recipe(
    config: RcpServerConfig,
    make_svc: impl SendMakeService,
) -> Result<(), std::io::Error> {
    // If tls config is provided.
    if let Some(tls_config) = config.tls {
        let rustls_config = RustlsConfig::from_pem_file(tls_config.cert_path, tls_config.key_path)
            .inspect_err(|e| error!("Error in reading tls configuration. Error: \n {}", e))
            .await?;

        axum_server::bind_rustls(config.addr, rustls_config)
            .serve(make_svc)
            .await
    } else {
        axum_server::bind(config.addr).serve(make_svc).await
    }
}

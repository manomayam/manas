//! I provide few common utils for recipe implementations.
//!

use std::sync::Arc;

use axum_server::{service::MakeServiceRef, tls_rustls::RustlsConfig};
use futures::TryFutureExt;
use http::{uri::Scheme, Method, Request};
use hyper::{server::conn::AddrStream, Body};
use manas_authentication::common::{
    credentials::impl_::basic::BasicRequestCredentials,
    req_authenticator::impl_::BasicRequestAuthenticator,
};
use manas_http::service::{
    impl_::{NormalValidateTargetUri, ReconstructTargetUri},
    HttpService,
};
use manas_storage::service::cors::LiberalCors;
use tower::{make::Shared, Layer};
use tower_http::catch_panic::CatchPanic;
use tracing::error;

use self::config::RcpServerConfig;

pub mod config;

/// Alias trait for [`MakeServiceRef`] with sendable futures.
pub trait SendMakeServiceRef:
    MakeServiceRef<AddrStream, Request<Body>, MakeFuture = Self::MakeFuture_>
{
    /// Type of future yielded by this service maker.
    type MakeFuture_: Send + 'static;
}

impl<S> SendMakeServiceRef for S
where
    S: MakeServiceRef<AddrStream, Request<Body>>,
    S::MakeFuture: Send + 'static,
{
    type MakeFuture_ = S::MakeFuture;
}

/// Resolve service maker for given podset service.
pub fn resolve_svc_maker(
    podset_svc: impl HttpService<Body, Body> + Clone,
) -> impl SendMakeServiceRef {
    Shared::new(CatchPanic::new(LiberalCors::new(
        ReconstructTargetUri::new(Scheme::HTTP, NormalValidateTargetUri::new(podset_svc)),
    )))
}

/// Resolve service maker for given podset service.
#[cfg(feature = "layer-authentication")]
pub fn resolve_authenticating_svc_maker(
    podset_svc: impl HttpService<Body, Body> + Clone,
) -> impl SendMakeServiceRef {
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
            Method::PUT,
            Method::DELETE,
        ]),
    )
    .layer(podset_svc))
}

/// Serve the recipe.
pub async fn serve_recipe(
    config: RcpServerConfig,
    make_svc: impl SendMakeServiceRef,
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

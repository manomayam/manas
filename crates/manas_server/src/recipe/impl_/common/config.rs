//! I provide few common types for recipe configurations.
//!

use std::{net::SocketAddr, path::PathBuf};

use http::HeaderName;
use serde_with::{serde_as, DisplayFromStr};

/// Recipe tls config.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RcpTlsConfig {
    /// Cert pem file path.
    pub cert_path: PathBuf,

    /// Key pem file path.
    pub key_path: PathBuf,
}

/// Recipe server config.
#[serde_as]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RcpServerConfig {
    /// Socket address to bind.
    pub addr: SocketAddr,

    /// Optional tls config.
    pub tls: Option<RcpTlsConfig>,

    /// Trusted proxy headers
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub trusted_proxy_headers: Vec<HeaderName>,
}

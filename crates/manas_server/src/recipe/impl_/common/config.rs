//! I provide few common types for recipe configurations.
//!

use std::{net::SocketAddr, path::PathBuf};

/// Recipe tls config.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RcpTlsConfig {
    /// Cert pem file path.
    pub cert_path: PathBuf,

    /// Key pem file path.
    pub key_path: PathBuf,
}

/// Recipe server config.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RcpServerConfig {
    /// Socket address to bind.
    pub addr: SocketAddr,

    /// Optional tls config.
    pub tls: Option<RcpTlsConfig>,
}

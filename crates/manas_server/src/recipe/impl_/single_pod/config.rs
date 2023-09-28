//! I provide types to represent configuration for
//! single-pod, databrowser enabled recipes.
//!

use std::collections::HashMap;

use manas_http::uri::invariant::HierarchicalTrailingSlashHttpUri;
use webid::WebId;

use crate::recipe::impl_::common::config::RcpServerConfig;

/// Recipe storage space config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RcpStorageSpaceConfig {
    /// Storage space root uri.
    pub root_uri: HierarchicalTrailingSlashHttpUri,

    /// Storage space owner id.
    pub owner_id: WebId,
}

/// Recipe repo config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RcpRepoConfig {
    /// Backend config.
    // TODO must be concrete struct.
    pub backend: HashMap<String, String>,

    /// Weather databrowser is enabled.
    #[serde(default)]
    pub databrowser_enabled: bool,
}

/// Recipe storage config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RcpStorageConfig {
    /// Storage space config
    pub space: RcpStorageSpaceConfig,

    /// Storage repo config.
    pub repo: RcpRepoConfig,
}

/// Recipe storage config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RcpConfig {
    /// Recipe storage config.
    pub storage: RcpStorageConfig,

    /// Recipe server config.
    pub server: RcpServerConfig,

    /// Wether to run in dev mode.
    #[serde(default)]
    pub dev_mode: bool,
}

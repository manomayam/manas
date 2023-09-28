//! I define concrete types for the storage spaces for recipes.
//!

use std::sync::Arc;

use gdp_rs::predicate::impl_::all_of::SculptPL;
use manas_http::uri::invariant::{HierarchicalTrailingSlashHttpUri, NormalAbsoluteHttpUri};
use manas_space::impl_::DefaultSolidStorageSpace;
use webid::WebId;

use crate::CW;

/// Type of storage spaces for recipes.
pub type RcpStorageSpace = DefaultSolidStorageSpace;

impl CW<RcpStorageSpace> {
    /// Get a new [`RcpStorageSpace`] wrapped in an arc.
    pub fn new_shared(
        root_res_uri: HierarchicalTrailingSlashHttpUri,
        description_res_uri: NormalAbsoluteHttpUri,
        owner_id: WebId,
    ) -> Arc<RcpStorageSpace> {
        Arc::new(RcpStorageSpace::new(
            root_res_uri.infer::<SculptPL<_, _, _, _>>(Default::default()),
            description_res_uri,
            owner_id,
        ))
    }
}

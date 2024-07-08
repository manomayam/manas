//! I define trait for repo's uri policy.
//!

use manas_http::header::slug::Slug;
use manas_space::{
    resource::{
        kind::SolidResourceKind, slot_path::RelativeSolidResourceSlotPath, uri::SolidResourceUri,
    },
    BoxError,
};

use crate::{Repo, RepoContextual};

pub mod impl_;

/// A trait for resource uri policy of a repo.
pub trait RepoUriPolicy: Clone + Send + Sync + 'static + RepoContextual {
    /// Get the uri of mutex resource of the specified resource.
    fn mutex_res_uri(&self, res_uri: &SolidResourceUri) -> Option<SolidResourceUri>;

    /// Get the mutex normal hash for given res uri.
    /// This method doesn't guarantee res uri is conformant to
    /// repo's uri policy.
    fn mutex_normal_res_uri_hash(&self, res_uri: &SolidResourceUri) -> String;

    /// This method suggests a uri for a new child resource,
    /// given it's parent resource uri and a slug_hint.
    /// This method's semantics are purely advisory.
    /// It only guarantees that returned uri is a valid child
    /// res uri as per repo's naming policy.
    fn suggest_res_uri(
        &self,
        parent_res_uri: &SolidResourceUri,
        slug_hint: &Slug,
        res_kind: SolidResourceKind,
    ) -> Result<SolidResourceUri, BoxError>;

    /// Check if given relative resource slot path is allowed as
    /// per repo's uri policy.
    /// It only guarantees that all uris are valid and can be
    /// assigned to resources in a  slot path in that order, in
    /// conformance with naming policy of the repo.
    fn is_allowed_relative_slot_path(
        &self,
        relative_slot_path: &RelativeSolidResourceSlotPath<'_, <Self::Repo as Repo>::StSpace>,
    ) -> bool;
}

//! I provide an implementation of [`RepoUriPolicy`] that
//! delegates to an inner policy.
//!

use manas_http::header::slug::Slug;
use manas_space::resource::{
    kind::SolidResourceKind, slot_path::RelativeSolidResourceSlotPath, uri::SolidResourceUri,
};

use crate::{
    context::{impl_::DelegatedRepoContextual, LayeredRepoContext},
    policy::uri::RepoUriPolicy,
    Repo,
};

/// an implementation of [`RepoUriPolicy`] that
/// delegates to an inner policy.
pub type DelegatedUriPolicy<Inner, LR> = DelegatedRepoContextual<Inner, LR>;

impl<Inner, LR> RepoUriPolicy for DelegatedUriPolicy<Inner, LR>
where
    Inner: RepoUriPolicy,
    LR: Repo<StSpace = <Inner::Repo as Repo>::StSpace>,
    LR::Context: LayeredRepoContext<InnerRepo = Inner::Repo>,
{
    #[inline]
    fn mutex_res_uri(&self, res_uri: &SolidResourceUri) -> Option<SolidResourceUri> {
        self.inner.mutex_res_uri(res_uri)
    }

    #[inline]
    fn mutex_normal_res_uri_hash(&self, res_uri: &SolidResourceUri) -> String {
        self.inner.mutex_normal_res_uri_hash(res_uri)
    }

    #[inline]
    fn suggest_res_uri(
        &self,
        parent_res_uri: &SolidResourceUri,
        slug_hint: &Slug,
        res_kind: SolidResourceKind,
    ) -> Result<SolidResourceUri, manas_space::BoxError> {
        self.inner
            .suggest_res_uri(parent_res_uri, slug_hint, res_kind)
    }

    #[inline]
    fn is_allowed_relative_slot_path(
        &self,
        relative_slot_path: &RelativeSolidResourceSlotPath<'_, LR::StSpace>,
    ) -> bool {
        self.inner.is_allowed_relative_slot_path(relative_slot_path)
    }
}

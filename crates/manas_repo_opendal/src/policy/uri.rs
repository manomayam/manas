//! This module implements uri policy for [`OpendalRepo`](OpendalRepo).

use std::{option::Option, sync::Arc};

use gdp_rs::Proven;
use manas_http::{
    header::slug::Slug,
    uri::component::segment::{
        invariant::NonEmptyCleanSegmentStr, predicate::is_normal::PctEncodingNormalization,
    },
};
use manas_repo::{
    context::{RepoContext, RepoContextual},
    policy::uri::RepoUriPolicy,
};
use manas_semslot::{process::step::SlotPathEncodeStep, SemanticResourceSlot};
use manas_space::resource::{
    kind::SolidResourceKind, slot_id::SolidResourceSlotId,
    slot_path::RelativeSolidResourceSlotPath, uri::SolidResourceUri,
};
use tower::BoxError;
use tracing::error;

use crate::{
    context::ODRContext,
    resource_context::ODRResourceContext,
    setup::{ODRSemSlotES, ODRSetup},
    OpendalRepo,
};

/// An implementation of [`RepoUriPolicy`] for [`OpendalRepo`].
///
/// It derives it's uri policy based on resource slot encoding scheme,
/// and object association scheme of the repo.
#[derive(Debug, Clone)]
pub struct ODRUriPolicy<Setup>
where
    Setup: ODRSetup,
{
    repo_context: Arc<ODRContext<Setup>>,
}

impl<Setup> ODRUriPolicy<Setup>
where
    Setup: ODRSetup,
{
    /// Get resource slot id for given res uri in current space.
    #[inline]
    fn slot_id_for(&self, uri: &SolidResourceUri) -> SolidResourceSlotId<Setup::StSpace> {
        SolidResourceSlotId {
            space: self.repo_context.storage_space().clone(),
            uri: uri.clone(),
        }
    }
}

impl<Setup> RepoContextual for ODRUriPolicy<Setup>
where
    Setup: ODRSetup,
{
    type Repo = OpendalRepo<Setup>;

    #[inline]
    fn new_with_context(repo_context: Arc<ODRContext<Setup>>) -> Self {
        Self { repo_context }
    }

    #[inline]
    fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

impl<Setup> RepoUriPolicy for ODRUriPolicy<Setup>
where
    Setup: ODRSetup,
{
    #[inline]
    fn mutex_res_uri(&self, res_uri: &SolidResourceUri) -> Option<SolidResourceUri> {
        SemanticResourceSlot::<_, ODRSemSlotES<Setup>>::try_new_mutex(self.slot_id_for(res_uri))
            .map(|mutex_semslot| mutex_semslot.res_uri().clone())
    }

    fn mutex_normal_res_uri_hash(&self, res_uri: &SolidResourceUri) -> String {
        SemanticResourceSlot::<_, ODRSemSlotES<Setup>>::try_new(self.slot_id_for(res_uri))
            .ok()
            .and_then(|semslot| semslot.mutex_normal())
            .map(|mutex_normal_semslot| mutex_normal_semslot.res_uri().as_str().to_owned())
            .unwrap_or_else(|| res_uri.as_str().to_owned())
    }

    #[tracing::instrument]
    fn suggest_res_uri(
        &self,
        parent_res_uri: &SolidResourceUri,
        slug_hint: &Slug,
        res_kind: SolidResourceKind,
    ) -> Result<SolidResourceUri, BoxError> {
        // Get normalized slug.
        let normal_slug = Proven::void_proven(slug_hint.as_ref())
            .infer::<PctEncodingNormalization<_>>(Default::default());

        // Get semslot for resource by computing linked slot
        // path of parent resource with a mero link encoding
        // step.
        let semslot = SemanticResourceSlot::<_, ODRSemSlotES<Setup>>::try_new(
            self.slot_id_for(parent_res_uri),
        )
        .map_err(|e| {
            error!("Error in decoding resource semslot from parent resource slot id.");
            e.into()
        })?
        .linked(SlotPathEncodeStep::Mero {
            slug: NonEmptyCleanSegmentStr::try_new_from(normal_slug.into_subject())?,
            slotted_res_kind: res_kind,
        }).map_err(|e| {
            error!("Error in computing linked resource semslot from parent resource's with mero link encode step. Error: {:?}", e);
            e
        })?;

        // Ensure context can be resolvable for resource uri to be suggested.
        let _ = ODRResourceContext::<Setup>::try_new(
            semslot.res_uri().clone(),
            self.repo_context.clone(),
        )?;

        Ok(semslot.res_uri().clone())
    }

    fn is_allowed_relative_slot_path(
        &self,
        relative_slot_path: &RelativeSolidResourceSlotPath<'_, Setup::StSpace>,
    ) -> bool {
        // Ensure base space is same as repo's space.
        self.repo_context.storage_space() == &relative_slot_path.space().base_res_slot_id.space
        // Ensure, can resolve context of the resource successfully.
            && ODRResourceContext::<Setup>::try_new(
                relative_slot_path.target_res_uri().clone(),
                self.repo_context.clone(),
            )
            .ok()
        // Ensure supplied resource slot path matches with that of decoded.
            .map(|context| {
                context.semslot()
                    .path_rev_iter()
                    .zip(relative_slot_path.slots().iter().rev())
                    .all(|(s1, s2)| {
                        s1.id().uri == s2.id().uri
                            && s1.slot_rev_link().map(|l|(&l.target, &l.rev_rel_type)) == s2.slot_rev_link().map(|l|(&l.target, &l.rev_rel_type))
                    })
                && context.kind() == relative_slot_path.target_res_slot().res_kind()
            })
            .unwrap_or(false)
    }
}

// /// I define few utils to mock with [`ODRUriPolicy`].
// #[cfg(feature = "test-utils")]
// pub mod mock {
//     use manas_repo::context::RepoContextual;

//     use crate::{context::mock::MockODRContext, setup::mock::MockODRSetup};

//     use super::ODRUriPolicy;

//     /// Type alias for mock [`ODRUriPolicy`].
//     pub type MockODRUriPolicy<const MAX_AUX_LINKS: usize> =
//         ODRUriPolicy<MockODRSetup<MAX_AUX_LINKS>>;

//     impl<const MAX_AUX_LINKS: usize> MockODRUriPolicy<MAX_AUX_LINKS> {
//         /// Get a new mock odr uri policy, for mock storage space
//         /// with given root uri.
//         pub fn new_mock(storage_root_uri_str: &str) -> Self {
//             ODRUriPolicy::new_with_context(MockODRContext::<MAX_AUX_LINKS>::new_mock(
//                 storage_root_uri_str,
//             ))
//         }
//     }
// }

// #[cfg(test)]
// #[cfg(feature = "test-utils")]
// mod tests {
//     use rstest::*;

//     use super::mock::MockODRUriPolicy;
//     use super::*;

//     use manas_space::{
//         resource::{
//             slot_path::SolidResourceSlotPath,
//             slot_rel_type::{aux_rel_type::ACL_REL_TYPE, mock::SlotRelationTypeHint},
//         },
//         RelativeSolidStorageSpace,
//     };

//     #[rstest]
//     #[case::root("http://ex.org/", "http://ex.org/", "http://ex.org/")]
//     #[case::non_spaced("http://ex.org/", "http://ex2.org/a/b", "http://ex2.org/a/b")]
//     #[case::contained_atom("http://ex.org/", "http://ex.org/a/b.png", "http://ex.org/a/b.png/")]
//     #[case::contained_container("http://ex.org/", "http://ex.org/a/bc/", "http://ex.org/a/bc/")]
//     #[case::aux_atom(
//         "http://ex.org/",
//         "http://ex.org/a/b._aux/acl",
//         "http://ex.org/a/b._aux/acl"
//     )]
//     #[case::aux_container(
//         "http://ex.org/",
//         "http://ex.org/a/b._aux/_tc1/",
//         "http://ex.org/a/b._aux/_tc1/"
//     )]
//     fn mutex_normal_res_uri_hash_works_correctly(
//         #[case] storage_root_uri_str: &str,
//         #[case] res_uri_str: &str,
//         #[case] expected: &str,
//     ) {
//         let policy = MockODRUriPolicy::<0>::new_mock(storage_root_uri_str);

//         assert_eq!(
//             policy
//                 .mutex_normal_res_uri_hash(
//                     &SolidResourceUri::try_new_from(res_uri_str)
//                         .expect("Claimed valid resource uri")
//                 )
//                 .as_str(),
//             expected,
//             "Mutex normal res uri hash expectation not satisfied."
//         );
//     }

//     #[rstest]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc",
//         SolidResourceKind::Container,
//         Some("http://ex.org/a/abc/")
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc",
//         SolidResourceKind::NonContainer,
//         Some("http://ex.org/a/abc")
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org2/a/",
//         "abc",
//         SolidResourceKind::Container,
//         None
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/._aux/acl",
//         "abc",
//         SolidResourceKind::Container,
//         None
//     )]
//     #[case::with_res_id_slot_path_encoding_aux_delim(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc._aux",
//         SolidResourceKind::Container,
//         None
//     )]
//     #[case::with_rep_id_slot_path_encoding_aux_delim(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc$aux",
//         SolidResourceKind::Container,
//         None
//     )]
//     #[case::with_supplem_obj_id_encoding_delim(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc.__c",
//         SolidResourceKind::Container,
//         None
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc def",
//         SolidResourceKind::Container,
//         Some("http://ex.org/a/abc%20def/")
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc%2Fd",
//         SolidResourceKind::NonContainer,
//         Some("http://ex.org/a/abc%252Fd")
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "abc$def",
//         SolidResourceKind::Container,
//         Some("http://ex.org/a/abc$def/")
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         "राम",
//         SolidResourceKind::NonContainer,
//         Some("http://ex.org/a/%E0%A4%B0%E0%A4%BE%E0%A4%AE")
//     )]
//     fn suggest_res_uri_works_correctly(
//         #[case] storage_root_uri_str: &str,
//         #[case] parent_res_uri_str: &str,
//         #[case] slug_hint_str: &str,
//         #[case] res_kind: SolidResourceKind,
//         #[case] expected_ok_str: Option<&str>,
//     ) {
//         let policy = MockODRUriPolicy::<0>::new_mock(storage_root_uri_str);

//         assert_eq!(
//             policy
//                 .suggest_res_uri(
//                     &SolidResourceUri::try_new_from(parent_res_uri_str)
//                         .expect("Claimed valid resource uri."),
//                     &Slug::from(slug_hint_str),
//                     res_kind
//                 )
//                 .ok()
//                 .map(|uri| uri.as_str().to_owned()),
//             expected_ok_str.map(|e| e.to_owned()),
//             "Suggest res uri expectation not satisfied."
//         );
//     }

//     #[rstest]
//     #[case("http://ex.org/", "http://ex.org/", None, true)]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/", SolidResourceKind::Container),
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/b/", SolidResourceKind::Container)
//             ],
//         ),
//         true,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/", SolidResourceKind::Container),
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/b/", SolidResourceKind::NonContainer)
//             ],
//         ),
//         false,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/b", SolidResourceKind::NonContainer)
//             ],
//         ),
//         true,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/b", SolidResourceKind::Container)
//             ],
//         ),
//         false,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/a/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/b", SolidResourceKind::NonContainer)
//             ],
//         ),
//         false,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/", SolidResourceKind::Container),
//                 (SlotRelationTypeHint::Auxiliary(&ACL_REL_TYPE), "http://ex.org/a/._aux/acl", SolidResourceKind::NonContainer),
//             ],
//         ),
//         true,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/", SolidResourceKind::Container),
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/._aux/", SolidResourceKind::Container),
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/._aux/acl", SolidResourceKind::NonContainer),
//             ],
//         ),
//         false,
//     )]
//     #[case(
//         "http://ex.org/",
//         "http://ex.org/",
//         Some(
//             vec![
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a", SolidResourceKind::NonContainer),
//                 (SlotRelationTypeHint::Contains, "http://ex.org/a/b", SolidResourceKind::NonContainer)
//             ],
//         ),
//         false,
//     )]
//     fn allowed_relative_slot_path_check_works_correctly(
//         #[case] storage_root_uri_str: &str,
//         #[case] relative_space_base_uri_str: &str,
//         #[case] slot_path_hint: Option<Vec<(SlotRelationTypeHint, &str, SolidResourceKind)>>,
//         #[case] expected: bool,
//     ) {
//         let policy = MockODRUriPolicy::<0>::new_mock(storage_root_uri_str);
//         let space = policy.repo_context.storage_space().clone();

//         // Construct relative slot path.
//         let slot_path = SolidResourceSlotPath::new_mock(
//             Arc::new(RelativeSolidStorageSpace {
//                 base_res_slot_id: SolidResourceSlotId {
//                     space,
//                     uri: SolidResourceUri::try_new_from(relative_space_base_uri_str)
//                         .expect("Claimed valid"),
//                 },
//             }),
//             slot_path_hint,
//         );

//         assert_eq!(
//             policy.is_allowed_relative_slot_path(&slot_path),
//             expected,
//             "relative slot path validity check expectation not satisfied"
//         );
//     }
// }

//! I define utilities to query aux subjects index of a resource in ODR.
//!

use std::sync::Arc;

use futures::{stream::FuturesUnordered, TryStreamExt};
use manas_space::resource::slot_id::SolidResourceSlotId;
use smallvec::SmallVec;
use tracing::{error, info};

use crate::{resource_context::ODRResourceContext, setup::ODRSetup};

/// An enum representing error in resolving encoded aux subjects index.
#[derive(Debug, thiserror::Error)]
pub enum ODRAuxSubIndexResolutionError {
    /// Slot encoded aux links doesn't exists in backend.
    #[error("Slot encoded aux links doesn't exists in backend.")]
    EncodedAuxLinkSubjectsDoesNotExists,

    /// Unknown io error in resolution.
    #[error("Unknown io error in resolution.")]
    UnknownIoError(opendal::Error),
}

/// Resolve slot path aux subject index for resource with given
/// context.
pub async fn resolve_slot_path_aux_subjects_index<Setup: ODRSetup>(
    res_context: Arc<ODRResourceContext<Setup>>,
) -> Result<SmallVec<[SolidResourceSlotId<Setup::StSpace>; 1]>, ODRAuxSubIndexResolutionError> {
    // Decode aux link subject ids from resource id encoded slot path.
    let decoded_aux_link_subject_ids = res_context.semslot().path_aux_link_subject_ids();

    // Get contexts of aux subjects.
    let aux_subject_contexts =
        decoded_aux_link_subject_ids
            .clone()
            .into_iter()
            .map(|aux_subject_res_id| {
                Arc::new(
                    ODRResourceContext::try_new(
                        aux_subject_res_id.uri,
                        res_context.as_ref().repo_context().clone(),
                    )
                    .expect("Must be valid, as context encoded successfully for descendent."),
                )
            });

    // Ensure effective rep objects exists for all subject
    // resources with decoded ids.
    match aux_subject_contexts
        .map(|aux_subject_context| async move {
            aux_subject_context
                .as_ref()
                .assoc_odr_object_map()
                .base_object()
                .is_exist()
                .await
        })
        .collect::<FuturesUnordered<_>>()
        .try_fold(true, |acc, is_exist| async move { Ok(acc && is_exist) })
        .await
    {
        // On check io success.
        Ok(all_subject_indicators_exists) => {
            info!(
                "Aux subject indicator objects existence check succeeded with {}",
                all_subject_indicators_exists
            );

            if all_subject_indicators_exists {
                // If all aux subject indicator objects exists, return decoded ids.
                Ok(decoded_aux_link_subject_ids)
            } else {
                // Else,
                Err(ODRAuxSubIndexResolutionError::EncodedAuxLinkSubjectsDoesNotExists)
            }
        }

        Err(e) => {
            error!(
                "Io error in Aux subjects indicator objects existence check. kind: {}",
                e.kind()
            );
            Err(ODRAuxSubIndexResolutionError::UnknownIoError(e))
        }
    }
}

// #[cfg(test)]
// #[cfg(feature = "test-utils")]
// mod tests {
//     use crate::service::resource_operator::common::resource_state::mock::mock_filled_repo_context;
//     use claims::*;
//     use manas_http::uri::invariant::SolidResourceUri;
//     use rstest::*;

//     use super::*;

//     #[rstest]
//     #[case("http://ex.org/mock/s4/C1.html._aux/meta")]
//     #[case("http://ex.org/mock/s4/C1.html._aux/_tc2/ac1.txt")]
//     #[case("http://ex.org/mock/s3/")]
//     #[case("http://ex.org/mock/s5/Cb.html")]
//     #[tokio::test]
//     async fn aux_subjects_index_resolves_for_non_remnant_slot_path_path(#[case] res_uri_str: &str) {
//         let repo_context = mock_filled_repo_context().await.clone();

//         let res_slot = Arc::new(
//             ODRResourceContext::try_new(
//                 SolidResourceUri::try_new_from(res_uri_str).unwrap(),
//                 repo_context,
//             )
//             .unwrap(),
//         );

//         assert_ok!(resolve_slot_path_aux_subjects_index(res_slot).await);
//     }

//     #[rstest]
//     #[case("http://ex.org/mock/s4/c2.txt._aux/meta")]
//     #[case("http://ex.org/mock/s4/c2.txt._aux/_tc2/bc1.txt")]
//     #[case("http://ex.org/mock/s6._aux/acl")]
//     #[case("http://ex.org/mock/s4/c1.html._aux/meta")]
//     #[case("http://ex.org/mock/s4/c1.html._aux/_tc2/ac1.txt")]
//     #[tokio::test]
//     async fn aux_subjects_index_errors_for_remnant_slot_path_path(#[case] res_uri_str: &str) {
//         let repo_context = mock_filled_repo_context().await.clone();

//         let res_slot = Arc::new(
//             ODRResourceContext::try_new(
//                 SolidResourceUri::try_new_from(res_uri_str).unwrap(),
//                 repo_context,
//             )
//             .unwrap(),
//         );

//         assert_err!(resolve_slot_path_aux_subjects_index(res_slot).await);
//     }
// }

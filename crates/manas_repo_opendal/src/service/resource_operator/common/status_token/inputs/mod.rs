//! I provide types to represent status token inputs for ODR.
//!

pub(crate) mod altfm;
mod aux_subjects_index;
pub(crate) mod container_index;
pub(crate) mod supplem_fat_data;

use aux_subjects_index::*;
use opendal::Metadata;
use tracing::{error, info};

use crate::{
    object_store::{
        object::invariant::ODRFileObjectExt, object_space::assoc::rel_type::sidecar::SidecarRelType,
    },
    resource_context::invariant::ODRClassifiedResourceContext,
    setup::ODRSetup,
    util::opendal::OpendalResultExt,
};

/// A struct to represent inputs for odr resource status tokens.
#[derive(Debug, Clone)]
pub struct ODRResourceStatusTokenInputs<Setup: ODRSetup> {
    /// Resource context.
    pub res_context: ODRClassifiedResourceContext<Setup>,

    /// Wether slot path is represented.
    pub slot_path_is_represented: bool,

    /// Metadata of associated base object.
    pub base_obj_metadata: Option<Metadata>,

    /// Metadata of associated alt-content object.
    pub altcontent_obj_metadata: Option<Metadata>,

    /// Content of associated alt-fat-metadata object.
    pub altfm_obj_content: Option<Vec<u8>>,
}

impl<Setup: ODRSetup> ODRResourceStatusTokenInputs<Setup> {
    /// Get a new [`ODRResourceStatusTokenInputs`] for non-existing resource.
    #[inline]
    pub fn new_non_existing(res_context: ODRClassifiedResourceContext<Setup>) -> Self {
        Self {
            res_context,
            slot_path_is_represented: false,
            base_obj_metadata: None,
            altcontent_obj_metadata: None,
            altfm_obj_content: None,
        }
    }

    /// Try to get current resource status token base.
    #[tracing::instrument(
        skip_all,
        name = "ODRResourceStatusTokenInputs::try_current",
        fields(req)
    )]
    pub async fn try_current(
        res_context: ODRClassifiedResourceContext<Setup>,
    ) -> Result<Self, opendal::Error> {
        let assoc_obj_map = res_context.as_inner().as_ref().assoc_odr_object_map();

        // Send queries.
        let query_results = futures::future::join4(
            // Base object metadata.
            async { assoc_obj_map.base_object().metadata().await.found() },
            // AltContent object metadata.
            query_altcontent_obj_metadata(res_context.clone()),
            // AltFatMeta object content.
            query_altfm_obj_content(res_context.clone()),
            // Slo path aux subjects index.
            resolve_slot_path_aux_subjects_index(res_context.clone().into_inner()),
        )
        .await;

        let slot_path_is_represented = match query_results.3 {
            //On io success, and aux chain is represented
            Ok(_) => true,
            Err(e) => {
                match e {
                    // On aux chain is not represented,
                    ODRAuxSubIndexResolutionError::EncodedAuxLinkSubjectsDoesNotExists => {
                        info!("slot path is not represented.");
                        false
                    }

                    // On any io error.
                    ODRAuxSubIndexResolutionError::UnknownIoError(ioe) => {
                        error!("Backend io error in resolving slot path represented status.");
                        return Err(ioe);
                    }
                }
            }
        };

        Ok(Self {
            res_context,
            base_obj_metadata: query_results.0.map_err(|e| {
                error!(
                    "Unknown io error, when querying base object metadata. Error:\n {}",
                    e
                );
                e
            })?,
            altcontent_obj_metadata: query_results.1.map_err(|e| {
                error!(
                    "Unknown io error, when querying altcontent object metadata. Error:\n {}",
                    e
                );
                e
            })?,
            altfm_obj_content: query_results.2.map_err(|e| {
                error!(
                    "Unknown io error, when querying altfm object content. Error:\n {}",
                    e
                );
                e
            })?,
            slot_path_is_represented,
        })
    }
}

/// Query the associated altcontent object metadata.
pub(crate) async fn query_altcontent_obj_metadata<Setup: ODRSetup>(
    res_context: ODRClassifiedResourceContext<Setup>,
) -> Result<Option<Metadata>, opendal::Error> {
    let is_container_res = res_context.is_left_classified();

    if !is_container_res {
        // If non-container, then no need to query.
        return Ok(None);
    }

    res_context
        .as_ref()
        .as_ref()
        .assoc_odr_object_map()
        .sidecar_object(SidecarRelType::AltContent)
        .metadata()
        .await
        .found()
}

/// Query the associated altfm object content.
pub(crate) async fn query_altfm_obj_content<Setup: ODRSetup>(
    res_context: ODRClassifiedResourceContext<Setup>,
) -> Result<Option<Vec<u8>>, opendal::Error> {
    // Get if backend is cty capable.
    let is_cty_capable_backend = res_context
        .repo_context()
        .object_store
        .is_cty_capable_backend();

    if is_cty_capable_backend {
        return Ok(None);
    }

    res_context
        .as_ref()
        .as_ref()
        .assoc_odr_object_map()
        .sidecar_object(SidecarRelType::AltFatMeta)
        .read_complete()
        .await
        .found()
}

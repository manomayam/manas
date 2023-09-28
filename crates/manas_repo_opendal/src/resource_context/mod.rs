//! I define types to represent context of a resource in ODR.

use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use manas_repo::context::RepoContext;
use manas_semslot::{
    process::step::SlotPathEncodeStep, scheme::SemanticSlotEncodingScheme, SemanticResourceSlot,
};
use manas_space::{
    resource::{
        kind::SolidResourceKind, slot::SolidResourceSlot, slot_id::SolidResourceSlotId,
        slot_link::AuxLink, uri::SolidResourceUri,
    },
    SolidStorageSpace, SpcKnownAuxRelType,
};
use tracing::error;

use crate::{
    context::ODRContext,
    object_store::{assoc_object_map::ODRAssocObjectMap, ODRObjectAssociationError},
    setup::{aux_rep_policy::ODRAuxResourcePolicy, ODRSemSlotES, ODRSetup, ODRStSpace},
};

pub mod invariant;
pub mod predicate;

/// A struct to represent context of a resource in opendal repo.
#[derive(Clone)]
pub struct ODRResourceContext<Setup>
where
    Setup: ODRSetup,
{
    /// Repo context.
    repo_context: Arc<ODRContext<Setup>>,

    /// Semantic encoded slot of the resource.
    semslot: SemanticResourceSlot<'static, Setup::StSpace, ODRSemSlotES<Setup>>,

    /// Map from association link types to associated odr
    /// objects for the resource.
    assoc_odr_object_map: ODRAssocObjectMap<Setup::ObjectStoreSetup>,
}

impl<Setup: ODRSetup> Display for ODRResourceContext<Setup> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ODRResourceContext")
            .field("resource_uri", &self.slot().id().uri.as_str())
            .field("space_root", &self.storage_space().root_res_uri().as_str())
            .finish()
    }
}

impl<Setup: ODRSetup> Debug for ODRResourceContext<Setup> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<Setup: ODRSetup> ODRResourceContext<Setup> {
    /// Get storage space of the resource.
    #[inline]
    pub fn storage_space(&self) -> &Arc<Setup::StSpace> {
        self.semslot.space()
    }

    /// Get slot of the resource.
    #[inline]
    pub fn slot(&self) -> &SolidResourceSlot<Setup::StSpace> {
        self.semslot.inner()
    }

    /// Get uri of the resource.
    #[inline]
    pub fn uri(&self) -> &SolidResourceUri {
        &self.slot().id().uri
    }

    /// Get kind of the resource.
    #[inline]
    pub fn kind(&self) -> SolidResourceKind {
        self.slot().res_kind()
    }

    /// Get semslot of the resource.
    #[inline]
    pub fn semslot(&self) -> &SemanticResourceSlot<'static, Setup::StSpace, ODRSemSlotES<Setup>> {
        &self.semslot
    }

    /// Try create new [`ODRResourceContext`] from given resource slot.
    fn try_new_from_semslot(
        semslot: SemanticResourceSlot<'static, Setup::StSpace, ODRSemSlotES<Setup>>,
        repo_context: Arc<ODRContext<Setup>>,
    ) -> Result<Self, ODRResourceContextResolutionError<Setup>> {
        let assoc_odr_object_map = repo_context
            .as_ref()
            .object_store
            .assoc_odr_object_map(semslot.res_uri())
            .map_err(|e| {
                error!(
                    "Error in resolving assoc odr objects map for resource. error: {:?}",
                    &e
                );
                e
            })?;

        Ok(ODRResourceContext {
            repo_context,
            semslot,
            assoc_odr_object_map,
        })
    }

    /// Try create new [`ODRResourceContext`] from res uri.
    #[tracing::instrument(skip_all, name = "ODRResourceContext::try_new", fields(res_uri))]
    // TODO caching.
    pub fn try_new(
        res_uri: SolidResourceUri,
        repo_context: Arc<ODRContext<Setup>>,
    ) -> Result<Self, ODRResourceContextResolutionError<Setup>> {
        // Get resource slot id.
        let res_slot_id = SolidResourceSlotId {
            space: repo_context.storage_space().clone(),
            uri: res_uri,
        };

        // Decode semslot.
        let semslot = SemanticResourceSlot::<_, ODRSemSlotES<Setup>>::try_new(res_slot_id)
            .map_err(|e| {
                error!(
                    "Resource slot id has invalid encoded slot path. error: {:?}",
                    &e
                );
                ODRResourceContextResolutionError::InvalidIdEncodedSemSlot(e)
            })?;

        Self::try_new_from_semslot(semslot, repo_context)
    }

    /// Try create new [`ODRResourceContext`] for mutex resource.
    pub fn try_new_mutex(
        res_uri: SolidResourceUri,
        repo_context: Arc<ODRContext<Setup>>,
    ) -> Option<Self> {
        // Get resource slot id.
        let res_slot_id = SolidResourceSlotId {
            space: repo_context.storage_space().clone(),
            uri: res_uri,
        };

        // Decode mutex semslot.
        let mutex_semslot =
            SemanticResourceSlot::<_, ODRSemSlotES<Setup>>::try_new_mutex(res_slot_id)?;

        Self::try_new_from_semslot(mutex_semslot, repo_context).ok()
    }

    /// Get associated backend object map of the resource
    #[inline]
    pub fn assoc_odr_object_map(&self) -> &ODRAssocObjectMap<Setup::ObjectStoreSetup> {
        &self.assoc_odr_object_map
    }

    /// Get repo context.
    #[inline]
    pub fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }

    /// Get mutex resource context.
    pub fn mutex_resource_context(&self) -> Option<ODRResourceContext<Setup>> {
        self.semslot.mutex().and_then(|mutex_semslot| {
            ODRResourceContext::<Setup>::try_new_from_semslot(
                mutex_semslot,
                self.repo_context.clone(),
            )
            .ok()
        })
    }

    /// Get context of the host resource.
    pub fn host_resource_context(&self) -> Option<ODRResourceContext<Setup>> {
        self.semslot.host_slot().map(|host_semslot| {
            ODRResourceContext::<Setup>::try_new_from_semslot(
                host_semslot,
                self.repo_context.clone(),
            )
            .expect("Must be valid, for slot is that of host resource.")
        })
    }

    /// Get link to the resource linked through given aux rel
    /// type.
    /// NOTE: Private method, as it doesn't checks if aux rel
    /// type is supported by storage space or not.
    fn aux_res_link(
        &self,
        kn_aux_rel_type: SpcKnownAuxRelType<ODRStSpace<Setup>>,
    ) -> Option<AuxLink<Setup::StSpace>> {
        // Get semslot for aux res.
        let aux_res_semslot = self
            .semslot
            .linked(SlotPathEncodeStep::Aux {
                rel_type: kn_aux_rel_type.clone(),
            })
            .ok()?;

        Some(AuxLink::new(
            aux_res_semslot.res_uri().clone(),
            kn_aux_rel_type,
        ))
    }

    /// Get context of the aux resource linked through given
    /// aux rel type.
    /// NOTE: Method doesn't checks if aux rel type is
    /// supported by storage space or not.
    /// Callers must first ensure aux rel is supported by
    /// repo.
    pub(crate) fn aux_resource_context(
        &self,
        kn_aux_rel_type: SpcKnownAuxRelType<ODRStSpace<Setup>>,
    ) -> Option<ODRResourceContext<Setup>> {
        // Get semslot for aux res.
        let aux_res_semslot = self
            .semslot
            .linked(SlotPathEncodeStep::Aux {
                rel_type: kn_aux_rel_type,
            })
            .ok()?;

        ODRResourceContext::try_new_from_semslot(aux_res_semslot, self.repo_context().clone()).ok()
    }

    /// Get iterator of links to supported aux resources.
    pub fn supported_aux_links(&self) -> impl Iterator<Item = AuxLink<Setup::StSpace>> + '_ {
        <Setup::AuxResourcePolicy as ODRAuxResourcePolicy>::supported_aux_rel_types()
            .iter()
            .cloned()
            .filter_map(|kn_aux_rel_type| self.aux_res_link(kn_aux_rel_type))
    }

    /// Get iterator over contexts of supported aux resources
    /// of the resource.
    pub fn supported_aux_resource_contexts(
        &self,
    ) -> impl Iterator<
        Item = (
            SpcKnownAuxRelType<Setup::StSpace>,
            ODRResourceContext<Setup>,
        ),
    > + '_ {
        <Setup::AuxResourcePolicy as ODRAuxResourcePolicy>::supported_aux_rel_types()
            .iter()
            .filter_map(|kn_aux_rel_type| {
                self.aux_resource_context(kn_aux_rel_type.clone())
                    .map(|rc| (kn_aux_rel_type.clone(), rc))
            })
    }
}

/// An error type for error in resolving odr resource slot.
#[derive(Debug, thiserror::Error)]
pub enum ODRResourceContextResolutionError<Setup: ODRSetup> {
    /// Invalid encoded resource slot.
    #[error("Invalid encoded resource slot.")]
    InvalidIdEncodedSemSlot(<ODRSemSlotES<Setup> as SemanticSlotEncodingScheme>::DecodeError),

    /// Error in computing associated odr object for resource.
    #[error("Error in computing associated odr object for resource.")]
    ObjectAssociationError(#[from] ODRObjectAssociationError<Setup::ObjectStoreSetup>),
}

// /// I define few utils to mock with [`ODRResourceContext`].
// #[cfg(feature = "test-utils")]
// pub mod mock {
//     use crate::setup::mock::MockODRSetup;

//     use super::*;

//     /// A type alias for [`ODRResourceContext`] in repo with mock setup.
//     pub type MockODRResourceContext<const MAX_AUX_LINKS: usize> =
//         ODRResourceContext<MockODRSetup<MAX_AUX_LINKS>>;
// }

//! I define [`ODRAssocObjectMap`].
//!

use std::{collections::HashMap, ops::Index};

use super::{
    object::{
        invariant::{ODRClassifiedObject, ODRFileObject, ODRNamespaceObject},
        ODRObject,
    },
    object_space::assoc::rel_type::{sidecar::SidecarRelType, AssocRelType},
    ODRObjectStoreSetup,
};

/// A struct for representing index of associated odr objects
/// for a resource.
#[derive(Debug, Clone)]
pub struct ODRAssocObjectMap<OstSetup>
where
    OstSetup: ODRObjectStoreSetup,
{
    /// Associated base odr object.
    pub(super) base_object: ODRClassifiedObject<'static, OstSetup>,

    /// Associated aux namespace odr object.
    pub(super) aux_ns_object: ODRNamespaceObject<'static, OstSetup>,

    /// Associated sidecar odr objects.
    pub(super) sidecars: HashMap<SidecarRelType, ODRFileObject<'static, OstSetup>>,
}

impl<OstSetup> ODRAssocObjectMap<OstSetup>
where
    OstSetup: ODRObjectStoreSetup,
{
    /// Get associated base object.
    #[inline]
    pub fn base_object(&self) -> &ODRClassifiedObject<'static, OstSetup> {
        &self.base_object
    }

    /// Get associated aux namespace object.
    #[inline]
    pub fn aux_ns_object(&self) -> &ODRNamespaceObject<'static, OstSetup> {
        &self.aux_ns_object
    }

    /// Get associated sidecar object.
    #[inline]
    pub fn sidecar_object(
        &self,
        sidecar_rel_type: SidecarRelType,
    ) -> &ODRFileObject<'static, OstSetup> {
        &self.sidecars[&sidecar_rel_type]
    }
}

impl<OstSetup> Index<AssocRelType> for ODRAssocObjectMap<OstSetup>
where
    OstSetup: ODRObjectStoreSetup,
{
    type Output = ODRObject<'static, OstSetup>;

    fn index(&self, index: AssocRelType) -> &Self::Output {
        match index {
            AssocRelType::Base => self.base_object.as_inner(),
            AssocRelType::AuxNS => self.aux_ns_object.as_ref(),
            AssocRelType::Sidecar(sidecar_rel_tye) => self.sidecar_object(sidecar_rel_tye).as_ref(),
        }
    }
}

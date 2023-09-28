use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use manas_http::uri::invariant::NormalAbsoluteHttpUri;
use webid::WebId;

use crate::{
    policy::aux::{impl_::DefaultAuxPolicy, AuxPolicy},
    SolidStorageSpace,
};

/// A basic implementation of [`SolidStorageSpace`].
pub struct BasicSolidStorageSpace<AuxPol> {
    /// Uri of the storage space root resource.
    pub root_res_uri: NormalAbsoluteHttpUri,

    /// Uri of the storage space description resource.
    pub description_res_uri: NormalAbsoluteHttpUri,

    /// Webid of the storage owner.
    pub owner_id: WebId,

    _phantom: PhantomData<fn() -> AuxPol>,
}

impl<AuxPol> PartialEq for BasicSolidStorageSpace<AuxPol> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.root_res_uri == other.root_res_uri
            && self.description_res_uri == other.description_res_uri
            && self.owner_id == other.owner_id
    }
}

impl<AuxPol> Eq for BasicSolidStorageSpace<AuxPol> {}

impl<AuxPol> Clone for BasicSolidStorageSpace<AuxPol> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            root_res_uri: self.root_res_uri.clone(),
            description_res_uri: self.description_res_uri.clone(),
            owner_id: self.owner_id.clone(),
            _phantom: self._phantom,
        }
    }
}

impl<AuxPol> Display for BasicSolidStorageSpace<AuxPol> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BasicSolidStorageSpace({})", self.root_res_uri.as_str())
    }
}

impl<AuxPol> Debug for BasicSolidStorageSpace<AuxPol> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<AuxPol> BasicSolidStorageSpace<AuxPol> {
    /// Create a new [`BasicSolidStorageSpace`] with given params.
    #[inline]
    pub fn new(
        root_res_uri: NormalAbsoluteHttpUri,
        description_res_uri: NormalAbsoluteHttpUri,
        owner_id: WebId,
    ) -> Self {
        Self {
            root_res_uri,
            description_res_uri,
            owner_id,
            _phantom: PhantomData,
        }
    }
}

impl<AuxPol> SolidStorageSpace for BasicSolidStorageSpace<AuxPol>
where
    AuxPol: AuxPolicy,
{
    type AuxPolicy = AuxPol;

    #[inline]
    fn root_res_uri(&self) -> &NormalAbsoluteHttpUri {
        &self.root_res_uri
    }

    #[inline]
    fn description_res_uri(&self) -> &NormalAbsoluteHttpUri {
        &self.description_res_uri
    }

    #[inline]
    fn owner_id(&self) -> &WebId {
        &self.owner_id
    }
}

/// Alias for default storage space.
pub type DefaultSolidStorageSpace = BasicSolidStorageSpace<DefaultAuxPolicy>;

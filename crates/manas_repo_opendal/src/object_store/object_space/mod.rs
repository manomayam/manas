//! I define [`ODRObjectSpace`].
//!

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use if_chain::if_chain;
use manas_http::uri::component::segment::safe_token::{ConflictFreeToken, TSegmentSafeToken};
use manas_semslot::scheme::{
    impl_::hierarchical::{
        aux::AuxLinkEncodingScheme,
        decoder::{
            decode_from_hierarchical_uri_path, InvalidHierarchicalEncodedResourceSlot, SLASH_CHAR,
        },
        encoder::encode_to_hierarchical_relative_uri_path,
    },
    SemanticSlotEncodingScheme,
};
use manas_space::{
    resource::{slot_id::SolidResourceSlotId, uri::SolidResourceUri},
    SolidStorageSpace,
};
use tracing::debug;

use self::assoc::{
    link::AssocLink,
    mapping_scheme::{ODRObjectSpaceAssocMappingScheme, ODRObjectSpaceSidecarAssocMappingScheme},
    rel_type::{sidecar::SidecarRelType, AssocRelType},
    rev_link::AssocRevLink,
};
use super::object_id::ODRObjectId;
use crate::object_store::object_id::normal_rootless_uri_path::NormalRootlessUriPath;

pub mod assoc;
pub mod impl_;

/// A trait for representing concrete setup for generic [`ODRObjectSpace`].
pub trait ODRObjectSpaceSetup: Debug + Clone + Send + Sync + 'static + Unpin {
    /// Type of associated storage space, to which this object space is associated with.
    type AssocStSpace: SolidStorageSpace;

    /// Type of semantic slot encoding scheme used for
    /// encoding semantic slots for associated storage
    /// space.
    type AssocStSemSlotES: SemanticSlotEncodingScheme<Space = Self::AssocStSpace>;

    /// Type of association mapping scheme that governs
    /// mapping from associated storage space to object space.
    type AssocMappingScheme: ODRObjectSpaceAssocMappingScheme<AssocStSpace = Self::AssocStSpace>;
}

/// An odr object space is a space of objects
/// that is associated with a given solid storage space.
///
/// For each resource in the assoc storage space, object space
/// associates multiple objects as per it's mapping rules.
///
#[derive(Debug, Clone)]
pub struct ODRObjectSpace<OSSetup: ODRObjectSpaceSetup> {
    /// Storage space, to which this object space is associated with.
    pub assoc_storage_space: Arc<OSSetup::AssocStSpace>,
}

impl<OSSetup: ODRObjectSpaceSetup> PartialEq for ODRObjectSpace<OSSetup> {
    fn eq(&self, other: &Self) -> bool {
        self.assoc_storage_space == other.assoc_storage_space
    }
}

impl<OSSetup: ODRObjectSpaceSetup> Eq for ODRObjectSpace<OSSetup> {}

impl<ObjSpaceSetup: ODRObjectSpaceSetup> ODRObjectSpace<ObjSpaceSetup> {
    /// Get sidecar link encode delim.
    #[inline]
    fn sidecar_link_encode_delim() -> &'static str {
        <<<ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::SidecarAssocMS as ODRObjectSpaceSidecarAssocMappingScheme>::SidecarLinkDelim as TSegmentSafeToken>::token().as_ref()
    }

    /// Get base obj path aux link encode delim.
    #[inline]
    fn base_obj_path_aux_link_encode_delim() -> &'static str {
        <<<ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::BaseObjPathAuxLinkES as AuxLinkEncodingScheme>::AuxLinkDelim as TSegmentSafeToken>::token().as_ref()
    }

    /// Encode object id of base object associated with given resource uri.
    pub fn encode_assoc_base_obj_id(
        &self,
        res_uri: &SolidResourceUri,
    ) -> Result<ODRObjectId<'static, ObjSpaceSetup>, ODRAssocMappingError> {
        // Get resource slot id..
        let res_slot_id = SolidResourceSlotId {
            space: self.assoc_storage_space.clone(),
            uri: res_uri.clone(),
        };

        // Decode resource's slot path encode process.
        let slot_path_encode_process = ObjSpaceSetup::AssocStSemSlotES::decode(&res_slot_id)
            .map_err(|_| ODRAssocMappingError::InvalidIdEncodedResourceSlot)?;

        let base_obj_root_relative_path = encode_to_hierarchical_relative_uri_path::<
            ObjSpaceSetup::AssocStSpace,
            <ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::BaseObjPathAuxLinkES,
        >(&slot_path_encode_process)
        .map_err(|_| ODRAssocMappingError::ResourceSlotIdHasExtraEncodingSemantics)?;

        debug!(
            "res_uri: {}, assoc_base_obj_path: {}",
            res_slot_id.uri.as_str(),
            &base_obj_root_relative_path
        );

        // Ensure supplem aux link encoding delim is not present in path.
        if base_obj_root_relative_path.contains(Self::sidecar_link_encode_delim()) {
            return Err(ODRAssocMappingError::ResourceSlotIdHasExtraSupplemLinkEncodingSemantics);
        }

        Ok(ODRObjectId {
            space: self.clone(),

            // SAFETY: Encoded path is guaranteed to be normal and root less.
            // As, encode process allows only normal and clean slugs, tokens.
            root_relative_path: unsafe {
                NormalRootlessUriPath::new_unchecked(base_obj_root_relative_path.into())
            },
        })
    }

    /// Encode id of associated odr object for given resource and association type.
    pub fn encode_assoc_obj_id(
        &self,
        res_uri: &SolidResourceUri,
        assoc_rel_type: AssocRelType,
    ) -> Result<ODRObjectId<'static, ObjSpaceSetup>, ODRAssocMappingError> {
        let base_obj_id = self.encode_assoc_base_obj_id(res_uri)?;

        Ok(match assoc_rel_type {
            AssocRelType::Base => base_obj_id,
            AssocRelType::AuxNS => Self::encode_assoc_aux_space_dir_obj_id(&base_obj_id),
            AssocRelType::Sidecar(sidecar_rel_type) => {
                Self::encode_sidecar_obj_id(&base_obj_id, sidecar_rel_type)
            }
        })
    }

    /// Encode aux space dir object id.
    pub(crate) fn encode_assoc_aux_space_dir_obj_id(
        base_obj_id: &ODRObjectId<'_, ObjSpaceSetup>,
    ) -> ODRObjectId<'static, ObjSpaceSetup> {
        ODRObjectId {
            space: base_obj_id.space.clone(),
            // SAFETY: safe, as all tokens are guaranteed to be normal, and clean.
            root_relative_path: unsafe {
                NormalRootlessUriPath::new_unchecked(
                    format!(
                        "{}{}/",
                        // Rep obj path.
                        base_obj_id.root_relative_path.as_ref(),
                        // Aux link delim.
                        Self::base_obj_path_aux_link_encode_delim(),
                    )
                    .into(),
                )
            },
        }
    }

    /// Encode sidecar object id.
    pub(crate) fn encode_sidecar_obj_id(
        base_obj_id: &ODRObjectId<'_, ObjSpaceSetup>,
        sidecar_rel_type: SidecarRelType,
    ) -> ODRObjectId<'static, ObjSpaceSetup> {
        ODRObjectId {
            space: base_obj_id.space.clone(),
            // SAFETY: safe, as all tokens are guaranteed to be normal, and clean.
            root_relative_path: unsafe {
                NormalRootlessUriPath::new_unchecked(
                    format!(
                        "{}{}{}",
                        // Rep obj path.
                        base_obj_id.root_relative_path.as_ref(),
                        // Supplem link delim.
                        Self::sidecar_link_encode_delim(),
                        // Supplem rel_type encode token.
                        <<ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::SidecarAssocMS as ODRObjectSpaceSidecarAssocMappingScheme>::sidecar_rel_type_encoding_token(sidecar_rel_type).as_str()
                    )
                    .into(),
                )
            },
        }
    }

    /// Get association links for resource with given uri, indexed by link's rel type.
    pub fn assoc_links_for_res(
        &self,
        res_uri: &SolidResourceUri,
    ) -> Result<HashMap<AssocRelType, AssocLink<'static, ObjSpaceSetup>>, ODRAssocMappingError>
    {
        let base_obj_id = self.encode_assoc_base_obj_id(res_uri)?;

        let mut links_index = HashMap::new();

        // Insert assoc link to aux dir object.
        links_index.insert(
            AssocRelType::AuxNS,
            AssocLink {
                target: Self::encode_assoc_aux_space_dir_obj_id(&base_obj_id),
                rel_type: AssocRelType::AuxNS,
            },
        );

        // Insert assoc links for sidecar objects.
        for sidecar_rel_type in SidecarRelType::ALL {
            let assoc_rel_type = AssocRelType::Sidecar(*sidecar_rel_type);

            links_index.insert(
                assoc_rel_type,
                AssocLink {
                    target: Self::encode_sidecar_obj_id(&base_obj_id, *sidecar_rel_type),
                    rel_type: assoc_rel_type,
                },
            );
        }

        // Insert rep object id.
        links_index.insert(
            AssocRelType::Base,
            AssocLink {
                target: base_obj_id,
                rel_type: AssocRelType::Base,
            },
        );

        Ok(links_index)
    }

    /// Get unique association reverse link for odr object with given root relative path.
    pub fn assoc_rev_link_for_odr_obj(
        &self,
        odr_obj_root_relative_path: &NormalRootlessUriPath,
    ) -> Result<AssocRevLink<ObjSpaceSetup>, ODRRevAssocMappingError> {
        // Start with assumption, that obj id is rep object id.
        let (mut obj_path_base_part, mut assoc_rev_rel_type) =
            (odr_obj_root_relative_path.as_ref(), AssocRelType::Base);

        // Update if it is aux dir id.
        if_chain!(
            // Strip trailing slash
            if let Some(obj_path_slash_stripped) = odr_obj_root_relative_path.strip_suffix(SLASH_CHAR);
            // strip aux link delim token.
            if let Some(obj_path_aux_delim_stripped) = obj_path_slash_stripped.strip_suffix(Self::base_obj_path_aux_link_encode_delim());
            then {
                obj_path_base_part = obj_path_aux_delim_stripped;
                assoc_rev_rel_type = AssocRelType::AuxNS;
            }
        );

        // Update if it is supplem object id.
        if_chain!(
            // Split by sidecar link delim, if path contains one.
            if let Some((base_path_part, rel_type_encode_token_str)) = odr_obj_root_relative_path.split_once(Self::sidecar_link_encode_delim());

            // Ensure valid delim safe rel type encode token.
            if let Ok(rel_type_encode_token) = ConflictFreeToken::try_from(rel_type_encode_token_str);

            // Ensure, delim token corresponds to a sidecar rel_type.
            if let Some(sidecar_rel_type) = <<ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::SidecarAssocMS as ODRObjectSpaceSidecarAssocMappingScheme>::encoded_sidecar_rel_type(&rel_type_encode_token);

            then {
                obj_path_base_part = base_path_part;
                assoc_rev_rel_type = AssocRelType::Sidecar(sidecar_rel_type);
            }
        );

        // Ensure, no further assoc link encoding delims exists in base path.
        if obj_path_base_part.contains(Self::sidecar_link_encode_delim()) {
            return Err(ODRRevAssocMappingError::ObjIdHasExtraSupplemLinkEncodingSemantics);
        }

        let slot_path_encode_process = decode_from_hierarchical_uri_path::<
            _,
            <ObjSpaceSetup::AssocMappingScheme as ODRObjectSpaceAssocMappingScheme>::BaseObjPathAuxLinkES,
        >(
            self.assoc_storage_space.clone(), obj_path_base_part
        )?;

        let res_id = ObjSpaceSetup::AssocStSemSlotES::encode(&slot_path_encode_process)
            .map_err(|_| ODRRevAssocMappingError::ObjIdHasExtraEncodingSemantics)?;

        Ok(AssocRevLink {
            target: res_id,
            rev_rel_type: assoc_rev_rel_type,
        })
    }

    /// Get a new [`ODRObjectSpace`] for given associated storage space.
    #[inline]
    pub fn new(assoc_storage_space: Arc<ObjSpaceSetup::AssocStSpace>) -> Self {
        Self {
            assoc_storage_space,
        }
    }

    /// Get associated storage space of this odr object space.
    #[inline]
    pub fn assoc_storage_space(&self) -> &Arc<ObjSpaceSetup::AssocStSpace> {
        &self.assoc_storage_space
    }
}
/// An error type for errors in computing associated odr object
/// for a resource.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ODRAssocMappingError {
    /// Invalid encoded resource slot.
    #[error("Invalid encoded resource slot.")]
    InvalidIdEncodedResourceSlot,

    /// Given resource slot id has extra encoding semantics.
    #[error("Given resource slot id has extra encoding semantics.")]
    ResourceSlotIdHasExtraEncodingSemantics,

    /// Given resource slot id has extra supplem link encoding
    /// semantics.
    #[error("Given resource slot id has extra supplem link encoding semantics.")]
    ResourceSlotIdHasExtraSupplemLinkEncodingSemantics,
}

/// An error type for errors in computing reverse association
/// for an odr object.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ODRRevAssocMappingError {
    /// Obj id has extra supplem link encoding semantics.
    #[error("Obj id has extra supplem link encoding semantics.")]
    ObjIdHasExtraSupplemLinkEncodingSemantics,

    /// Invalid encoded resource slot.
    #[error("Invalid encoded resource slot.")]
    InvalidObjIdEncodedProvPath(#[from] InvalidHierarchicalEncodedResourceSlot),

    /// Obj id has extra encoding semantics.
    #[error("Obj id has extra encoding semantics.")]
    ObjIdHasExtraEncodingSemantics,
}

/// I define few utils for mocking with [`ODRObjectSpace`].
#[cfg(feature = "test-utils")]
pub mod mock {
    use manas_semslot::scheme::mock::MockSemanticSlotEncodingScheme;
    use manas_space::mock::MockSolidStorageSpace;

    use super::{assoc::mapping_scheme::mock::MockAssocMappingScheme, *};

    /// A mock implementation of [`ODRObjectSpace`Setup].
    #[derive(Debug, Clone)]
    pub struct MockODRObjectSpaceSetup<const MAX_AUX_LINKS: usize = 0> {}

    impl<const MAX_AUX_LINKS: usize> ODRObjectSpaceSetup for MockODRObjectSpaceSetup<MAX_AUX_LINKS> {
        type AssocStSpace = MockSolidStorageSpace<MAX_AUX_LINKS>;

        type AssocStSemSlotES = MockSemanticSlotEncodingScheme<MAX_AUX_LINKS>;

        type AssocMappingScheme = MockAssocMappingScheme<MAX_AUX_LINKS>;
    }

    /// A type alias for [`ODRObjectSpace`] with mock setup.
    pub type MockODRObjectSpace<const MAX_AUX_LINKS: usize = 0> =
        ODRObjectSpace<MockODRObjectSpaceSetup<MAX_AUX_LINKS>>;

    impl<const MAX_AUX_LINKS: usize> MockODRObjectSpace<MAX_AUX_LINKS> {
        /// Get a new mock object space with given associated storage space root.
        pub fn new_mock(assoc_storage_space_root_uri_str: &str) -> Self {
            Self::new(Arc::new(
                MockSolidStorageSpace::new_from_valid_root_uri_str(
                    assoc_storage_space_root_uri_str,
                ),
            ))
        }
    }
}

#[cfg(test)]
#[cfg(feature = "test-utils")]
mod tests {
    use std::collections::HashSet;

    use rstest::*;

    use super::{mock::*, *};

    #[rstest]
    #[case::root(
        "http://ex.org/",
        "http://ex.org/",
        Ok(vec![
            (AssocRelType::Base, ""),
            (AssocRelType::AuxNS, "$aux/"),
            (AssocRelType::ALT_FAT_META, ".__altfm"),
            (AssocRelType::ALT_CONTENT, ".__altcontent")
        ])
    )]
    #[case::contained(
        "http://ex.org/",
        "http://ex.org/a/b.png",
        Ok(vec![
            (AssocRelType::Base, "a/b.png"),
            (AssocRelType::AuxNS, "a/b.png$aux/"),
            (AssocRelType::ALT_FAT_META, "a/b.png.__altfm"),
            (AssocRelType::ALT_CONTENT, "a/b.png.__altcontent")
        ])
    )]
    #[case::container(
        "http://ex.org/",
        "http://ex.org/a/b/",
        Ok(vec![
            (AssocRelType::Base, "a/b/"),
            (AssocRelType::AuxNS, "a/b/$aux/"),
            (AssocRelType::ALT_CONTENT, "a/b/.__altcontent"),
            (AssocRelType::ALT_FAT_META, "a/b/.__altfm")
        ])
    )]
    #[case::aux(
        "http://ex.org/",
        "http://ex.org/a._aux/acl",
        Ok(vec![
            (AssocRelType::Base, "a$aux/acl"),
            (AssocRelType::AuxNS, "a$aux/acl$aux/"),
            (AssocRelType::ALT_FAT_META, "a$aux/acl.__altfm"),
            (AssocRelType::ALT_CONTENT, "a$aux/acl.__altcontent")
        ])
    )]
    #[case::aux_chain(
        "http://ex.org/",
        "http://ex.org/r1._aux/acl._aux/meta",
        Ok(vec![
            (AssocRelType::Base, "r1$aux/acl$aux/meta"),
            (AssocRelType::AuxNS, "r1$aux/acl$aux/meta$aux/"),
            (AssocRelType::ALT_FAT_META, "r1$aux/acl$aux/meta.__altfm"),
            (AssocRelType::ALT_CONTENT, "r1$aux/acl$aux/meta.__altcontent")
        ])
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/b._aux/",
        Err(ODRAssocMappingError::InvalidIdEncodedResourceSlot)
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/b._aux/containerindex",
        Err(ODRAssocMappingError::InvalidIdEncodedResourceSlot)
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/b$auxcd/",
        Err(ODRAssocMappingError::ResourceSlotIdHasExtraEncodingSemantics)
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/b.__def/",
        Err(ODRAssocMappingError::ResourceSlotIdHasExtraSupplemLinkEncodingSemantics)
    )]
    fn assoc_links_for_res_works_correctly(
        #[case] assoc_storage_root_uri_str: &str,
        #[case] res_uri_str: &str,
        #[case] expectation: Result<Vec<(AssocRelType, &str)>, ODRAssocMappingError>,
    ) {
        let obj_space = mock::MockODRObjectSpace::<0>::new_mock(assoc_storage_root_uri_str);

        let links_result_flattened = obj_space
            .assoc_links_for_res(
                &SolidResourceUri::try_new_from(res_uri_str).expect("Claimed valid"),
            )
            .map(|links_index| {
                links_index
                    .values()
                    .map(|l| (l.rel_type, l.target.root_relative_path.to_string()))
                    .collect::<HashSet<_>>()
            });

        let expected_result_flattened = expectation.map(|e| {
            e.iter()
                .map(|l| (l.0, l.1.to_owned()))
                .collect::<HashSet<_>>()
        });

        assert_eq!(
            links_result_flattened, expected_result_flattened,
            "Assoc links expectation failed."
        );
    }

    #[rstest]
    #[case(
        "",
        "http://ex.org/",
        Ok((AssocRelType::Base, "http://ex.org/"))
    )]
    #[case(
        "$aux/",
        "http://ex.org/",
        Ok((AssocRelType::AuxNS, "http://ex.org/"))
    )]
    #[case(
        "a/c.png",
        "http://ex.org/",
        Ok((AssocRelType::Base, "http://ex.org/a/c.png"))
    )]
    #[case(
        "a/c.png$aux/",
        "http://ex.org/",
        Ok((AssocRelType::AuxNS, "http://ex.org/a/c.png"))
    )]
    #[case(
        "a/c.png.__altfm",
        "http://ex.org/",
        Ok((AssocRelType::ALT_FAT_META, "http://ex.org/a/c.png"))
    )]
    #[case(
        "a/b$aux/_tc1/$aux/containerindex",
        "http://ex.org/",
        Ok((AssocRelType::Base, "http://ex.org/a/b._aux/_tc1/._aux/containerindex"))
    )]
    #[case(
        "a$aux/acl$aux/",
        "http://ex.org/",
        Ok((AssocRelType::AuxNS, "http://ex.org/a._aux/acl"))
    )]
    #[case(
        "a/b/.__altfm",
        "http://ex.org/",
        Ok((AssocRelType::ALT_FAT_META, "http://ex.org/a/b/"))
    )]
    #[case(
        "a/b.__c/d",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::ObjIdHasExtraSupplemLinkEncodingSemantics)
    )]
    #[case(
        "a/b._aux",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::ObjIdHasExtraEncodingSemantics)
    )]
    #[case(
        "a/b$aux",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::InvalidObjIdEncodedProvPath(
            InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim
        ))
    )]
    #[case(
        "a/b$aux/containerindex",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::InvalidObjIdEncodedProvPath(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkSubjectConstraintsViolation
        ))
    )]
    #[case(
        "a/b$aux/acl/",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::InvalidObjIdEncodedProvPath(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation
        ))
    )]
    #[case(
        "a/b$aux/png",
        "http://ex.org/",
        Err(ODRRevAssocMappingError::InvalidObjIdEncodedProvPath(
            InvalidHierarchicalEncodedResourceSlot::HasUnknownAuxRelTypeEncodeToken
        ))
    )]
    fn assoc_rev_link_for_obj_works_correctly(
        #[case] odr_obj_root_relative_path: &str,
        #[case] assoc_storage_root_uri_str: &str,
        #[case] expectation: Result<(AssocRelType, &str), ODRRevAssocMappingError>,
    ) {
        let obj_space = MockODRObjectSpace::<0>::new_mock(assoc_storage_root_uri_str);

        let obj_root_relative_path =
            unsafe { NormalRootlessUriPath::new_unchecked(odr_obj_root_relative_path.into()) };

        let rev_link_result = obj_space.assoc_rev_link_for_odr_obj(&obj_root_relative_path);

        assert_eq!(
            rev_link_result.map(|l| (l.rev_rel_type, l.target.uri.as_str().to_owned())),
            expectation.map(|e| (e.0, e.1.to_owned())),
            "Assoc rev link expectation not met."
        );
    }
}

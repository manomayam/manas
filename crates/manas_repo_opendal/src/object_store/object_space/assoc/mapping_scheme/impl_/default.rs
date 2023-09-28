//! I define a default implementation of [`ODRObjectSpaceAssocMappingScheme`].

use std::marker::PhantomData;

use manas_http::uri::component::segment::{
    invariant::NonEmptyCleanSegmentStr,
    safe_token::{ConflictFreeToken, TSegmentSafeToken},
};
use manas_semslot::scheme::impl_::hierarchical::aux::{
    impl_::default::DefaultAuxLinkEncodingScheme, AuxLinkEncodingScheme,
};
use manas_space::{SolidStorageSpace, SpcKnownAuxRelType};
use once_cell::sync::Lazy;

use crate::object_store::object_space::assoc::{
    mapping_scheme::{
        ODRObjectSpaceAssocMappingScheme, ODRObjectSpaceSidecarAssocMappingScheme,
        SidecarRelTypeEncodingTokenBimap,
    },
    rel_type::sidecar::SidecarRelType,
};

/// An implementation of [`TSegmentSafeToken`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultSidecarLinkDelim;

static SIDECAR_LINK_DELIM_TOKEN: Lazy<NonEmptyCleanSegmentStr> =
    Lazy::new(|| NonEmptyCleanSegmentStr::try_new_from(".__").expect("Must be valid."));

impl TSegmentSafeToken for DefaultSidecarLinkDelim {
    #[inline]
    fn token() -> &'static NonEmptyCleanSegmentStr {
        Lazy::force(&SIDECAR_LINK_DELIM_TOKEN)
    }
}

/// Default implementation of [`ODRObjectSpaceSidecarAssocMappingScheme`].
#[derive(Debug, Clone)]
pub struct DefaultSidecarAssocMappingScheme;

static SCHEME_BIMAP: Lazy<SidecarRelTypeEncodingTokenBimap<DefaultSidecarLinkDelim>> =
    Lazy::new(|| {
        SidecarRelTypeEncodingTokenBimap::try_from_raw_items(&[
            (SidecarRelType::AltFatMeta, "altfm"),
            (SidecarRelType::AltContent, "altcontent"),
        ])
        .expect("Must be valid.")
    });

impl ODRObjectSpaceSidecarAssocMappingScheme for DefaultSidecarAssocMappingScheme {
    type SidecarLinkDelim = DefaultSidecarLinkDelim;

    fn sidecar_rel_type_encoding_token(
        supplem_rel_type: SidecarRelType,
    ) -> &'static ConflictFreeToken<DefaultSidecarLinkDelim> {
        SCHEME_BIMAP
            .get_by_left(&supplem_rel_type)
            .expect("Must be Some, as all possible items have correspondence.")
    }

    fn encoded_sidecar_rel_type(
        token: &ConflictFreeToken<DefaultSidecarLinkDelim>,
    ) -> Option<SidecarRelType> {
        SCHEME_BIMAP.get_by_right(token).copied()
    }
}

/// Default implementation of [`ODRObjectSpaceAssocMappingScheme`].
#[derive(Debug)]
pub struct DefaultAssocMappingScheme<ASSpace, RepObjPathAuxLinkES = DefaultAuxLinkEncodingScheme>
where
    ASSpace: SolidStorageSpace,
    RepObjPathAuxLinkES: AuxLinkEncodingScheme<KnownAuxRelType = SpcKnownAuxRelType<ASSpace>>,
{
    _phantom: PhantomData<ASSpace>,
    _phantom_es: PhantomData<RepObjPathAuxLinkES>,
}

impl<ASSpace, RepObjPathAuxLinkES> ODRObjectSpaceAssocMappingScheme
    for DefaultAssocMappingScheme<ASSpace, RepObjPathAuxLinkES>
where
    ASSpace: SolidStorageSpace,
    RepObjPathAuxLinkES: AuxLinkEncodingScheme<KnownAuxRelType = SpcKnownAuxRelType<ASSpace>>,
{
    type AssocStSpace = ASSpace;

    type BaseObjPathAuxLinkES = RepObjPathAuxLinkES;

    type SidecarAssocMS = DefaultSidecarAssocMappingScheme;
}

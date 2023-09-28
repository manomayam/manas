//! I define a default implementation of [`AuxLinkEncodingScheme`].

use manas_http::{
    header::link::RelationType,
    uri::component::segment::{
        invariant::NonEmptyCleanSegmentStr,
        safe_token::{ConflictFreeToken, TSegmentSafeToken},
    },
};
use manas_space::resource::slot_rel_type::aux_rel_type::{
    known::impl_::default::DefaultKnownAuxRelType, ACL_REL_TYPE, DESCRIBED_BY_REL_TYPE,
};
use once_cell::sync::Lazy;

use crate::scheme::impl_::hierarchical::aux::{AuxLinkEncodingScheme, RelTypeEncodingTokenBimap};

/// Default aux link delimiter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultAuxLinkDelim;

static AUX_LINK_DELIM_TOKEN: Lazy<NonEmptyCleanSegmentStr> =
    Lazy::new(|| NonEmptyCleanSegmentStr::try_new_from("._aux").expect("Must be valid."));

impl TSegmentSafeToken for DefaultAuxLinkDelim {
    #[inline]
    fn token() -> &'static NonEmptyCleanSegmentStr {
        Lazy::force(&AUX_LINK_DELIM_TOKEN)
    }
}

/// Default implementation of [`AuxLinkEncodingScheme`].
#[derive(Debug, Clone)]
pub struct DefaultAuxLinkEncodingScheme;

static SCHEME_BIMAP_ITEMS: &[(&Lazy<RelationType>, &str)] =
    &[(&ACL_REL_TYPE, "acl"), (&DESCRIBED_BY_REL_TYPE, "meta")];

static SCHEME_BIMAP: Lazy<RelTypeEncodingTokenBimap<DefaultKnownAuxRelType, DefaultAuxLinkDelim>> =
    Lazy::new(|| {
        RelTypeEncodingTokenBimap::try_from_raw_items(SCHEME_BIMAP_ITEMS).expect("Must be valid")
    });

impl AuxLinkEncodingScheme for DefaultAuxLinkEncodingScheme {
    type KnownAuxRelType = DefaultKnownAuxRelType;

    type AuxLinkDelim = DefaultAuxLinkDelim;

    #[inline]
    fn rel_type_encoding_token(
        kn_aux_rel_type: &Self::KnownAuxRelType,
    ) -> &'static ConflictFreeToken<Self::AuxLinkDelim> {
        SCHEME_BIMAP
            .get_by_left(kn_aux_rel_type)
            .expect("Must be Some, as all possible items have correspondence.")
    }

    #[inline]
    fn encoded_rel_type(
        token: &ConflictFreeToken<Self::AuxLinkDelim>,
    ) -> Option<&Self::KnownAuxRelType> {
        SCHEME_BIMAP.get_by_right(token)
    }
}

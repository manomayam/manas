//! I define [`AuxLinkEncodingScheme`].

use std::{fmt::Debug, ops::Deref};

use bimap::BiMap;
use manas_http::{
    header::link::RelationType,
    uri::component::segment::safe_token::{ConflictFreeToken, TSegmentSafeToken},
};
use manas_space::{resource::slot_rel_type::aux_rel_type::known::KnownAuxRelType, BoxError};
use once_cell::sync::Lazy;

pub mod impl_;

/// A trait for scheme of encoding aux links in hierarchical uri.
///
pub trait AuxLinkEncodingScheme: Debug + Clone + 'static + Send + Sync {
    /// Type of known aux rel type this scheme is defined
    /// against.
    type KnownAuxRelType: KnownAuxRelType;

    /// Type of aux link delim.
    type AuxLinkDelim: TSegmentSafeToken;

    /// Get corresponding encoding token for a given known rel
    /// type.
    ///
    /// Implementations must ensure that token is distinct for
    /// different rel types.
    fn rel_type_encoding_token(
        kn_aux_rel_type: &Self::KnownAuxRelType,
    ) -> &'static ConflictFreeToken<Self::AuxLinkDelim>;

    /// Get known rel type encoded in given token.
    /// If token doesn't match with any, return [`None`].
    fn encoded_rel_type(
        token: &ConflictFreeToken<Self::AuxLinkDelim>,
    ) -> Option<&Self::KnownAuxRelType>;
}

/// A helper bijection b/w aux rel type, and encoding token.
pub struct RelTypeEncodingTokenBimap<KnRelType, D>(pub BiMap<KnRelType, ConflictFreeToken<D>>)
where
    KnRelType: KnownAuxRelType,
    D: TSegmentSafeToken;

impl<KnRelType, D> RelTypeEncodingTokenBimap<KnRelType, D>
where
    KnRelType: KnownAuxRelType,
    D: TSegmentSafeToken,
{
    /// Try to create new rel type encoding token bimap from raw items.
    pub fn try_from_raw_items(
        items: &[(&'static Lazy<RelationType>, &'static str)],
    ) -> Result<Self, BoxError> {
        let mut map = BiMap::new();

        for (rel_type, token_str) in items {
            map.insert(
                (*rel_type).deref().clone().try_into()?,
                (*token_str).try_into()?,
            );
        }

        Ok(Self(map))
    }
}

impl<KnRelType, D> Deref for RelTypeEncodingTokenBimap<KnRelType, D>
where
    KnRelType: KnownAuxRelType,
    D: TSegmentSafeToken,
{
    type Target = BiMap<KnRelType, ConflictFreeToken<D>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "test-utils")]
///This module provides utils for mocking with [`AuxLinkEncodingScheme`].
pub mod mock {
    use manas_http::uri::component::segment::invariant::NonEmptyCleanSegmentStr;
    use manas_space::resource::slot_rel_type::aux_rel_type::{
        known::mock::MockKnownAuxRelType, mock::*,
    };

    use super::*;

    /// An implementation of [`TSegmentSafeToken`] for mocking.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MockAuxLinkDelim;

    static AUX_LINK_DELIM_TOKEN: Lazy<NonEmptyCleanSegmentStr> =
        Lazy::new(|| NonEmptyCleanSegmentStr::try_new_from("._aux").expect("Must be valid."));

    impl TSegmentSafeToken for MockAuxLinkDelim {
        #[inline]
        fn token() -> &'static NonEmptyCleanSegmentStr {
            Lazy::force(&AUX_LINK_DELIM_TOKEN)
        }
    }

    /// An implementation of [`TSegmentSafeToken`] for mocking.
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MockAuxLinkDelim2;

    static AUX_LINK_DELIM_TOKEN2: Lazy<NonEmptyCleanSegmentStr> =
        Lazy::new(|| NonEmptyCleanSegmentStr::try_new_from("$aux").expect("Must be valid."));

    impl TSegmentSafeToken for MockAuxLinkDelim2 {
        #[inline]
        fn token() -> &'static NonEmptyCleanSegmentStr {
            Lazy::force(&AUX_LINK_DELIM_TOKEN2)
        }
    }

    /// A mock implementation of [`AuxLinkEncodingScheme`].
    #[derive(Debug, Clone)]
    pub struct MockAuxLinkEncodingScheme;

    static SCHEME_BIMAP_ITEMS: &[(&Lazy<RelationType>, &str)] = &[
        (&ACL_REL_TYPE, "acl"),
        (&DESCRIBED_BY_REL_TYPE, "meta"),
        (&CONTAINER_INDEX_REL_TYPE, "containerindex"),
        (&TA1_REL_TYPE, "_ta1"),
        (&TA2_REL_TYPE, "_ta2"),
        (&TC1_REL_TYPE, "_tc1"),
        (&TC2_REL_TYPE, "_tc2"),
    ];

    static SCHEME_BIMAP: Lazy<RelTypeEncodingTokenBimap<MockKnownAuxRelType, MockAuxLinkDelim>> =
        Lazy::new(|| {
            RelTypeEncodingTokenBimap::try_from_raw_items(SCHEME_BIMAP_ITEMS)
                .expect("Must be valid")
        });

    impl AuxLinkEncodingScheme for MockAuxLinkEncodingScheme {
        type KnownAuxRelType = MockKnownAuxRelType;

        type AuxLinkDelim = MockAuxLinkDelim;

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

    static SCHEME2_BIMAP: Lazy<RelTypeEncodingTokenBimap<MockKnownAuxRelType, MockAuxLinkDelim2>> =
        Lazy::new(|| {
            RelTypeEncodingTokenBimap::try_from_raw_items(SCHEME_BIMAP_ITEMS)
                .expect("Must be valid")
        });

    /// A mock implementation of [`AuxLinkEncodingScheme`].
    #[derive(Debug, Clone)]
    pub struct MockAuxLinkEncodingScheme2;
    impl AuxLinkEncodingScheme for MockAuxLinkEncodingScheme2 {
        type KnownAuxRelType = MockKnownAuxRelType;

        type AuxLinkDelim = MockAuxLinkDelim2;

        #[inline]
        fn rel_type_encoding_token(
            kn_aux_rel_type: &Self::KnownAuxRelType,
        ) -> &'static ConflictFreeToken<Self::AuxLinkDelim> {
            SCHEME2_BIMAP
                .get_by_left(kn_aux_rel_type)
                .expect("Must be Some, as all possible items have correspondence.")
        }

        #[inline]
        fn encoded_rel_type(
            token: &ConflictFreeToken<Self::AuxLinkDelim>,
        ) -> Option<&Self::KnownAuxRelType> {
            SCHEME2_BIMAP.get_by_right(token)
        }
    }
}

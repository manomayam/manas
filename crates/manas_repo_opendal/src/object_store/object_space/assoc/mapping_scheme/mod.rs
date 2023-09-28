//! I define a mapping scheme to encode odr object
//! paths from resource slot id, and assoc relation.
//!

use std::{fmt::Debug, ops::Deref};

use bimap::BiMap;
use manas_http::uri::component::segment::safe_token::{ConflictFreeToken, TSegmentSafeToken};
use manas_semslot::scheme::impl_::hierarchical::aux::AuxLinkEncodingScheme;
use manas_space::{SolidStorageSpace, SpcKnownAuxRelType};
use tower::BoxError;

use super::rel_type::sidecar::SidecarRelType;

pub mod impl_;

/// A trait for defining assoc mapping from resource id to associated ors object.
pub trait ODRObjectSpaceAssocMappingScheme: Debug + Send + Sync + 'static {
    /// Type of associated storage space.
    type AssocStSpace: SolidStorageSpace;

    /// Encode scheme, that specifies how to encode aux rev link of  a resource
    /// in odr object path of it's base object.
    type BaseObjPathAuxLinkES: AuxLinkEncodingScheme<
        KnownAuxRelType = SpcKnownAuxRelType<Self::AssocStSpace>,
    >;

    /// Type of sidecar assoc mapping scheme.
    type SidecarAssocMS: ODRObjectSpaceSidecarAssocMappingScheme;
}

/// A trait for defining assoc mapping from resource slot id to
/// associated sidecar odr object.
pub trait ODRObjectSpaceSidecarAssocMappingScheme: Debug {
    /// Type supplying sidecar link encoding delim.
    type SidecarLinkDelim: TSegmentSafeToken;

    /// Get corresponding encoding token for a given supplem rel
    /// type.
    fn sidecar_rel_type_encoding_token(
        sidecar_rel_type: SidecarRelType,
    ) -> &'static ConflictFreeToken<Self::SidecarLinkDelim>;

    /// Get sidecar rel type encoded in given token.
    fn encoded_sidecar_rel_type(
        suffix_token: &ConflictFreeToken<Self::SidecarLinkDelim>,
    ) -> Option<SidecarRelType>;
}

/// A helper bijection b/w sidecar rel type, and encoding token.
pub struct SidecarRelTypeEncodingTokenBimap<D>(pub BiMap<SidecarRelType, ConflictFreeToken<D>>)
where
    D: TSegmentSafeToken;

impl<D> SidecarRelTypeEncodingTokenBimap<D>
where
    D: TSegmentSafeToken,
{
    /// Try to create new rel type encoding token bimap from raw
    /// items.
    pub fn try_from_raw_items(items: &[(SidecarRelType, &'static str)]) -> Result<Self, BoxError> {
        let mut map = BiMap::new();

        for (rel_type, token_str) in items {
            map.insert(*rel_type, (*token_str).try_into()?);
        }

        Ok(Self(map))
    }
}

impl<D> Deref for SidecarRelTypeEncodingTokenBimap<D>
where
    D: TSegmentSafeToken,
{
    type Target = BiMap<SidecarRelType, ConflictFreeToken<D>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// I provide few utils for mocking with [`ODRObjectSpaceAssocMappingScheme`].
#[cfg(feature = "test-utils")]
pub mod mock {
    use manas_semslot::scheme::impl_::hierarchical::aux::mock::MockAuxLinkEncodingScheme2;
    use manas_space::mock::MockSolidStorageSpace;

    use super::impl_::default::DefaultAssocMappingScheme;

    /// A mock implementation of [`ODRObjectSpaceAssocMappingScheme`](super::ODRObjectSpaceAssocMappingScheme).
    pub type MockAssocMappingScheme<const MAX_AUX_LINKS: usize = 0> =
        DefaultAssocMappingScheme<MockSolidStorageSpace<MAX_AUX_LINKS>, MockAuxLinkEncodingScheme2>;
}

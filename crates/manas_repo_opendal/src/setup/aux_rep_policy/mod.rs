//! I define [`ODRAuxResourcePolicy`] trait for specifying configuration for an ODR.

use std::{collections::HashSet, ops::Deref};

use manas_http::header::common::media_type::MediaType;
use manas_space::{
    resource::slot_rel_type::aux_rel_type::ACL_REL_TYPE, SolidStorageSpace, SpcKnownAuxRelType,
};

pub mod impl_;

/// A trait for odr aux resource policy.
pub trait ODRAuxResourcePolicy: Send + Sync + 'static {
    /// Type of storage space.
    type StSpace: SolidStorageSpace;

    /// Wether should allow custom aux rep media type.
    const ALLOW_CUSTOM_AUX_REP_MEDIA_TYPE: bool;

    /// Get the set of supported aux rel types.
    fn supported_aux_rel_types() -> &'static HashSet<SpcKnownAuxRelType<Self::StSpace>>;

    /// Get media type of representation associated with an aux
    /// resource.
    fn aux_rep_media_type(aux_rel_type: &SpcKnownAuxRelType<Self::StSpace>) -> &'static MediaType;
}

/// An extension trait for ODRAuxResourcePolicy
pub trait ODRAuxResourcePolicyExt: ODRAuxResourcePolicy {
    /// Get the known aux rel type for acl relation.
    fn known_acl_rel_type(
    ) -> Option<&'static SpcKnownAuxRelType<<Self as ODRAuxResourcePolicy>::StSpace>> {
        Self::supported_aux_rel_types()
            .iter()
            .find(|kn_aux_rel_type| (*kn_aux_rel_type).deref() == &*ACL_REL_TYPE)
    }
}

impl<P: ODRAuxResourcePolicy> ODRAuxResourcePolicyExt for P {}

/// I provide few utils for mocking with [`ODRAuxResourcePolicy`].
#[cfg(feature = "test-utils")]
pub mod mock {
    use std::collections::HashSet;

    use manas_http::header::common::media_type::MediaType;
    use manas_space::{
        mock::MockSolidStorageSpace,
        resource::slot_rel_type::aux_rel_type::known::mock::{
            MockKnownAuxRelType, ALL_KNOWN_AUX_REL_TYPES,
        },
        SpcKnownAuxRelType,
    };
    use once_cell::sync::Lazy;

    use super::ODRAuxResourcePolicy;

    /// Mock implementation of [`ODRAuxResourcePolicy`].
    #[derive(Debug, Clone, Default)]
    pub struct MockODRAuxResourcePolicy<const MAX_AUX_LINKS: usize = 0> {}

    static TURTLE_MEDIA_TYPE: Lazy<MediaType> =
        Lazy::new(|| "text/turtle".parse().expect("Must be valid"));

    impl<const MAX_AUX_LINKS: usize> ODRAuxResourcePolicy for MockODRAuxResourcePolicy<MAX_AUX_LINKS> {
        type StSpace = MockSolidStorageSpace<MAX_AUX_LINKS>;

        const ALLOW_CUSTOM_AUX_REP_MEDIA_TYPE: bool = false;

        fn supported_aux_rel_types() -> &'static HashSet<SpcKnownAuxRelType<Self::StSpace>> {
            &ALL_KNOWN_AUX_REL_TYPES
        }

        #[inline]
        fn aux_rep_media_type(_: &MockKnownAuxRelType) -> &'static MediaType {
            // For all aux resources, allow only turtle media type.
            Lazy::force(&TURTLE_MEDIA_TYPE)
        }
    }
}

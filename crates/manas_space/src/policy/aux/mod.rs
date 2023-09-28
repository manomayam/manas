//! I define types for defining aux policy of a solid storage space.

use std::{fmt::Debug, num::NonZeroUsize};

use crate::resource::slot_rel_type::aux_rel_type::known::KnownAuxRelType;

pub mod impl_;

/// A trait for defining auxiliary resource policy of a storage space.
pub trait AuxPolicy: Debug + Clone + PartialEq + Send + Sync + 'static {
    /// A type that can represent all known aux rel types.
    type KnownAuxRelType: KnownAuxRelType;

    /// Maximum number of aux links allowed in a slot path.
    /// [`None`] implies, no limit.
    const PROV_PATH_MAX_AUX_LINKS: Option<NonZeroUsize>;
}

#[cfg(feature = "test-utils")]
/// A module for easily mocking [`AuxPolicy`].
pub mod mock {
    use super::*;
    use crate::resource::slot_rel_type::aux_rel_type::known::mock::MockKnownAuxRelType;

    /// An implementation of [`AuxPolicy`] for testing purposes.
    ///
    /// It is generic over `MAX_AUX_LINKS` const type param.
    /// If it is zero, will be treated as no limit.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MockAuxPolicy<const MAX_AUX_LINKS: usize = 0>;

    impl<const MAX_AUX_LINKS: usize> AuxPolicy for MockAuxPolicy<MAX_AUX_LINKS> {
        type KnownAuxRelType = MockKnownAuxRelType;

        const PROV_PATH_MAX_AUX_LINKS: Option<NonZeroUsize> = NonZeroUsize::new(MAX_AUX_LINKS);
    }
}

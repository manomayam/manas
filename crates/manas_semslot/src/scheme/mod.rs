//! I define [`SemanticSlotEncodingScheme`] trait, and provide
//! an implementation of it with hierarchical encoding semantics.
//!

pub mod impl_;

use std::fmt::Debug;

use manas_space::{resource::slot_id::SolidResourceSlotId, BoxError, SolidStorageSpace};

use super::process::SlotPathEncodeProcess;

/// A semantic slot encoding scheme defines codec for
/// encoding and decoding a slot path in target resource's
/// slot id.
pub trait SemanticSlotEncodingScheme: Debug + Send + Sync + Clone + 'static {
    /// Type of storage space, this scheme supports.
    type Space: SolidStorageSpace;

    /// Type of encoding error.
    type EncodeError: Into<BoxError> + Debug + Send + Sync;

    /// Type of decoding error.
    type DecodeError: Into<BoxError> + Debug + Send + Sync;

    /// Encode given slot path encode process into resource slot id.
    /// Scheme must ensure that, if it succeeds encoding a
    /// given process, It must succeed it's host process.
    fn encode(
        process: &SlotPathEncodeProcess<Self::Space>,
    ) -> Result<SolidResourceSlotId<Self::Space>, Self::EncodeError>;

    /// Decode slot-path-encode-process from given resource
    /// slot id.
    fn decode(
        res_slot_id: &SolidResourceSlotId<Self::Space>,
    ) -> Result<SlotPathEncodeProcess<'static, Self::Space>, Self::DecodeError>;

    /// Decode the slot path encode process of mutex resource (if any) of the
    /// resource with given slot id.
    #[allow(clippy::type_complexity)]
    fn decode_mutex(
        res_slot_id: &SolidResourceSlotId<Self::Space>,
    ) -> Option<(
        SolidResourceSlotId<Self::Space>,
        SlotPathEncodeProcess<'static, Self::Space>,
    )>;
}

/// I define few utilities for mocking with [`SemanticSlotEncodingScheme`].
#[cfg(feature = "test-utils")]
pub mod mock {
    use manas_space::mock::MockSolidStorageSpace;

    use super::impl_::hierarchical::{
        aux::mock::MockAuxLinkEncodingScheme, HierarchicalSemanticSlotEncodingScheme,
    };

    /// Mock sem slot encoding scheme.
    pub type MockSemanticSlotEncodingScheme<const MAX_AUX_LINKS: usize = 0> =
        HierarchicalSemanticSlotEncodingScheme<
            MockSolidStorageSpace<MAX_AUX_LINKS>,
            MockAuxLinkEncodingScheme,
        >;
}

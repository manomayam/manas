//! I define [`AssocRelType].
//!

use self::sidecar::SidecarRelType;

pub mod sidecar;

/// An enum of association rel types.
/// It represents link rel type, for link between a resource to
/// an associated odr object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssocRelType {
    /// Base object.
    Base,

    /// Aux namespace object.
    AuxNS,

    /// A rel type indicating sidecar relation.
    Sidecar(SidecarRelType),
}

impl AssocRelType {
    /// Static slice of all assoc rel types.
    pub const ALL: &'static [Self] = &[
        Self::Base,
        Self::AuxNS,
        Self::ALT_CONTENT,
        Self::ALT_FAT_META,
    ];

    /// Associated constant for `Alt` sidecar rel type.
    pub const ALT_CONTENT: Self = Self::Sidecar(SidecarRelType::AltContent);

    /// Associated constant for `AltMeta` sidecar rel type.
    pub const ALT_FAT_META: Self = Self::Sidecar(SidecarRelType::AltFatMeta);
}

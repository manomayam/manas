//! I define [`SlotPathEncodeStep`].

use std::ops::Deref;

use manas_http::uri::component::segment::invariant::NonEmptyCleanSegmentStr;
use manas_space::{
    policy::aux::AuxPolicy,
    resource::{
        kind::SolidResourceKind,
        slot_rel_type::{aux_rel_type::known::KnownAuxRelType, SlotRelationType},
    },
    SolidStorageSpace,
};

/// An enum that represents a single step in encoding of
/// resource slot path.
///
/// Each encoding step specifies encoding of a single slot link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotPathEncodeStep<Space: SolidStorageSpace> {
    /// A Mero link encoding step
    Mero {
        /// Slug for this mero encoding step .
        slug: NonEmptyCleanSegmentStr,

        /// Kind of slotted resource.
        slotted_res_kind: SolidResourceKind,
    },

    /// Aux link encoding step.
    Aux {
        /// Aux rel type.
        rel_type: <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType,
    },
}

impl<Space> SlotPathEncodeStep<Space>
where
    Space: SolidStorageSpace,
{
    /// Get slot rel type corresponding to this step.
    pub fn encoded_slot_rel_type(
        &self,
    ) -> SlotRelationType<<Space::AuxPolicy as AuxPolicy>::KnownAuxRelType> {
        match self {
            Self::Mero { .. } => SlotRelationType::Contains,

            Self::Aux { rel_type } => SlotRelationType::Auxiliary(rel_type.clone()),
        }
    }

    /// Get encoding step, that is mutually exclusive to current
    /// one.
    /// Two mero steps differing only in `slotted_res_kind`
    /// attribute are mutually exclusive.
    pub fn mutex_step(&self) -> Option<Self> {
        match self {
            // If is mero encoding step, change slotted res kind.
            Self::Mero {
                slug,
                slotted_res_kind,
            } => Some(Self::Mero {
                slug: slug.clone(),
                slotted_res_kind: if slotted_res_kind == &SolidResourceKind::NonContainer {
                    SolidResourceKind::Container
                } else {
                    SolidResourceKind::NonContainer
                },
            }),

            // If aux encoding step, return [`None`].
            Self::Aux { .. } => None,
        }
    }

    /// Check if step is a mero link encoding step.
    #[inline]
    pub fn is_mero_link_encoding_step(&self) -> bool {
        match self {
            Self::Mero { .. } => true,
            Self::Aux { .. } => false,
        }
    }

    /// Check if step is an aux link encoding step.
    #[inline]
    pub fn is_aux_link_encoding_step(&self) -> bool {
        !self.is_mero_link_encoding_step()
    }

    /// Get kind of the slotted resource.
    #[inline]
    pub fn slotted_res_kind(&self) -> SolidResourceKind {
        match self {
            Self::Mero {
                slotted_res_kind, ..
            } => *slotted_res_kind,

            Self::Aux { rel_type } => rel_type.target_res_kind(),
        }
    }

    /// Translate to parallel step in other space.
    pub fn translate_parallel<OSpace: SolidStorageSpace>(
        &self,
    ) -> Result<SlotPathEncodeStep<OSpace>, SlotPathEncodeStepTranslationError> {
        match self {
            Self::Mero {
                slug,
                slotted_res_kind,
            } => Ok(SlotPathEncodeStep::Mero {
                slug: slug.clone(),
                slotted_res_kind: *slotted_res_kind,
            }),
            SlotPathEncodeStep::Aux { rel_type } => Ok(SlotPathEncodeStep::Aux {
                rel_type: rel_type.deref().clone().try_into().map_err(|_| {
                    SlotPathEncodeStepTranslationError::AuxRelTypeIsNotKnownInOtherSpace
                })?,
            }),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
/// An error type for representing error in translation of a
/// slot path encode step.
pub enum SlotPathEncodeStepTranslationError {
    /// Aux rel type is not known in other space.
    #[error("Aux rel type in given aux link encoding step is not known in other space.")]
    AuxRelTypeIsNotKnownInOtherSpace,
}

#[cfg(feature = "test-utils")]
/// Module providing few test utils.
pub mod mock {
    use manas_http::header::link::RelationType;
    use manas_space::BoxError;
    use once_cell::sync::Lazy;

    use super::*;

    /// A test helper enum for specifying  resource slot
    /// path encoding step hints quickly.
    #[derive(Debug, Clone)]
    pub enum SlotPathEncodeStepHint {
        /// Mero step hint.
        Mero(&'static str, SolidResourceKind),

        /// Aux step hint.
        Aux(&'static Lazy<RelationType>),
    }

    impl<Space: SolidStorageSpace> TryInto<SlotPathEncodeStep<Space>> for SlotPathEncodeStepHint {
        type Error = BoxError;

        fn try_into(self) -> Result<SlotPathEncodeStep<Space>, Self::Error> {
            Ok(match self {
                Self::Mero(slug_str, slotted_res_kind) => SlotPathEncodeStep::Mero {
                    slug: NonEmptyCleanSegmentStr::try_new_from(slug_str)?,
                    slotted_res_kind,
                },

                Self::Aux(rel_type) => SlotPathEncodeStep::Aux {
                    rel_type: Lazy::force(rel_type).clone().try_into()?,
                },
            })
        }
    }
}

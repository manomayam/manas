//! I define hierarchical scheme decoder functionality.
//!

use std::sync::Arc;

use manas_http::uri::component::segment::{
    invariant::NonEmptyCleanSegmentStr,
    safe_token::{TSegmentSafeToken, TokenIsNotConflictFree},
};
use manas_space::{
    policy::aux::AuxPolicy,
    resource::{kind::SolidResourceKind, slot_rel_type::aux_rel_type::known::KnownAuxRelType},
    SolidStorageSpace,
};

use super::aux::AuxLinkEncodingScheme;
use crate::process::{
    step::SlotPathEncodeStep, SlotPathEncodeProcess, SlotPathEncodeProcessExtensionError,
};

/// Constant for slash char.
pub const SLASH_CHAR: char = '/';

/// Constant for slash string.
pub const SLASH: &str = "/";

/// Decode encode-process from hierarchical uri path.
pub fn decode_from_hierarchical_uri_path<Space, AuxLinkES>(
    space: Arc<Space>,
    root_relative_uri_path: &str,
) -> Result<SlotPathEncodeProcess<'static, Space>, InvalidHierarchicalEncodedResourceSlot>
where
    Space: SolidStorageSpace,
    AuxLinkES:
        AuxLinkEncodingScheme<KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType>,
{
    let mut residual_uri_path_start: usize = 0;
    let mut cursor_in_aux_step = false;

    let mut process = SlotPathEncodeProcess::to_root(space);

    loop {
        // Get residual uri path.
        let residual_uri_path = &root_relative_uri_path[residual_uri_path_start..];

        // When no residual uri path is left,
        if residual_uri_path.is_empty() {
            // Ensure that, cursor is not in middle of decoding an aux step.
            if cursor_in_aux_step {
                return Err(InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim);
            }

            // Else break loop.
            break;
        }

        let aux_link_delim_token = <AuxLinkES::AuxLinkDelim as TSegmentSafeToken>::token();

        // Split by slash, to get this iter's uri path segment, and next residual uri path.
        let (mut iter_uri_segment, next_residual_uri_path) =
            if let Some(l_slash_index) = residual_uri_path.find(SLASH_CHAR) {
                (
                    &residual_uri_path[..l_slash_index],
                    Some(&residual_uri_path[l_slash_index + 1..]),
                )
            } else {
                (residual_uri_path, None)
            };

        // If iter uri segment equals to aux link delim token,
        if iter_uri_segment == aux_link_delim_token.as_ref().as_ref() {
            // Ensure, cursor is not already in middle of decoding an aux step.
            if cursor_in_aux_step {
                return Err(InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim);
            }

            // Ensure, residual path is some, so that aux rel can be decoded.
            if next_residual_uri_path.is_none() {
                return Err(InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim);
            }

            // Move cursor to next uri segment, and set flag for cursor context.
            residual_uri_path_start += iter_uri_segment.len() + 1;
            cursor_in_aux_step = true;

            continue;
        }

        // If cumulative uri path has trailing slash.
        let mut cum_uri_path_has_trailing_slash = next_residual_uri_path.is_some();

        // Now adjust cursor to consume only up to encoded mero
        // step bounds, and leave any aux_delim_token suffix for
        // next iteration.
        if let Some(iter_uri_segment_aux_delim_stripped) =
            iter_uri_segment.strip_suffix(aux_link_delim_token.as_ref().as_ref())
        {
            cum_uri_path_has_trailing_slash = false;
            iter_uri_segment = iter_uri_segment_aux_delim_stripped;
        }

        //Till now iter encoding bounds are established.
        // Now have to decode step from encoded tokens.

        // Decode slot link encode step
        let slot_link_encode_step = if cursor_in_aux_step {
            let aux_rel_encode_token = iter_uri_segment.try_into()?;

            // Get aux rel type corresponding to encoded token.
            let kn_aux_rel_type = AuxLinkES::encoded_rel_type(&aux_rel_encode_token)
                .ok_or(InvalidHierarchicalEncodedResourceSlot::HasUnknownAuxRelTypeEncodeToken)?;

            // Ensure aux rel target constraints are not violated.
            if kn_aux_rel_type.target_res_kind()
                != if cum_uri_path_has_trailing_slash {
                    SolidResourceKind::Container
                } else {
                    SolidResourceKind::NonContainer
                }
            {
                return Err(
                    InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation,
                );
            }

            SlotPathEncodeStep::Aux {
                rel_type: kn_aux_rel_type.clone(),
            }
        } else {
            // Else, encode step is a mero step.
            let target_slug = NonEmptyCleanSegmentStr::try_new_from(iter_uri_segment)
                .map_err(|_| InvalidHierarchicalEncodedResourceSlot::HasNonCleanStepEncoding)?;

            SlotPathEncodeStep::Mero {
                slug: target_slug,
                slotted_res_kind: if cum_uri_path_has_trailing_slash {
                    SolidResourceKind::Container
                } else {
                    SolidResourceKind::NonContainer
                },
            }
        };

        // Push link encode step to process.
        process.push_step(slot_link_encode_step)?;

        // Update cursor
        residual_uri_path_start = residual_uri_path_start
            + iter_uri_segment.len()
            + usize::from(cum_uri_path_has_trailing_slash);

        cursor_in_aux_step = false;
    }

    Ok(process)
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// An error type for errors of invalid hierarchical encode resource slot.
pub enum InvalidHierarchicalEncodedResourceSlot {
    /// Has invalid aux rel delim.
    #[error("Has invalid aux rel delim.")]
    HasInvalidAuxRelDelim,

    /// Has non clean step encoding.
    #[error("Has non clean step encoding.")]
    HasNonCleanStepEncoding,

    /// Has unknown aux rel type encode token.
    #[error("Has unknown aux rel type encode token.")]
    HasUnknownAuxRelTypeEncodeToken,

    /// Aux link target constraints violation.
    #[error("Aux link target constraints violation.")]
    AuxLinkTargetConstraintsViolation,

    /// Mero rel subject constraints violation.
    #[error("Mero rel subject constraints violation.")]
    MeroLinkSubjectConstraintsViolation,

    /// Aux rel subject constraints violation.
    #[error("Aux rel subject constraints violation.")]
    AuxLinkSubjectConstraintsViolation,

    /// Aux step count out of limit.
    #[error("Aux step count out of limit.")]
    AuxLinkCountOutOfLimit,
}

impl From<SlotPathEncodeProcessExtensionError> for InvalidHierarchicalEncodedResourceSlot {
    #[inline]
    fn from(e: SlotPathEncodeProcessExtensionError) -> Self {
        match e {
            SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation => {
                Self::MeroLinkSubjectConstraintsViolation
            }

            SlotPathEncodeProcessExtensionError::AuxLinkSubjectConstraintsViolation => {
                Self::AuxLinkSubjectConstraintsViolation
            }

            SlotPathEncodeProcessExtensionError::AuxStepCountOutOfLimit => {
                Self::AuxLinkCountOutOfLimit
            }
        }
    }
}

impl From<TokenIsNotConflictFree> for InvalidHierarchicalEncodedResourceSlot {
    #[inline]
    fn from(_: TokenIsNotConflictFree) -> Self {
        Self::HasNonCleanStepEncoding
    }
}

//! I define hierarchical scheme encoder functionality.
//!

use manas_http::uri::component::segment::safe_token::{ConflictFreeToken, TSegmentSafeToken};
use manas_space::{
    policy::aux::AuxPolicy,
    resource::{kind::SolidResourceKind, slot_rel_type::aux_rel_type::known::KnownAuxRelType},
    SolidStorageSpace,
};

use super::{aux::AuxLinkEncodingScheme, decoder::SLASH_CHAR};
use crate::process::{step::SlotPathEncodeStep, SlotPathEncodeProcess};

/// Materialize encode-process to hierarchical uri path.
pub fn encode_to_hierarchical_relative_uri_path<Space, AuxLinkES>(
    encode_process: &SlotPathEncodeProcess<'_, Space>,
) -> Result<String, InvalidHierarchicalEncodeProcess>
where
    Space: SolidStorageSpace,
    AuxLinkES:
        AuxLinkEncodingScheme<KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType>,
{
    let mut buffer = String::new();

    for link_encode_step in encode_process.steps() {
        match link_encode_step {
            SlotPathEncodeStep::Mero {
                slug,
                slotted_res_kind,
            } => {
                // Ensure slug is aux delim safe.
                let aux_delim_safe_target_slug =
                    ConflictFreeToken::<AuxLinkES::AuxLinkDelim>::try_from(slug.clone()).map_err(
                        |_| InvalidHierarchicalEncodeProcess::TargetSlugHasExtraEncodingSemantics,
                    )?;

                // Push slug.
                buffer.push_str(aux_delim_safe_target_slug.as_ref());

                // Push slash, if slotted resource is container.
                if slotted_res_kind == &SolidResourceKind::Container {
                    buffer.push(SLASH_CHAR);
                }
            }

            SlotPathEncodeStep::Aux { rel_type } => {
                let rel_type_encoding_token = AuxLinkES::rel_type_encoding_token(rel_type);

                let aux_link_delim_token = <AuxLinkES::AuxLinkDelim as TSegmentSafeToken>::token();

                // Push aux link delim token.
                buffer.push_str(aux_link_delim_token.as_ref());

                // Push slash
                buffer.push(SLASH_CHAR);

                // Push aux rel type encoding token.
                buffer.push_str(rel_type_encoding_token.as_ref());

                // Push slash if targets a container.
                if rel_type.target_res_kind() == SolidResourceKind::Container {
                    buffer.push(SLASH_CHAR);
                }
            }
        }
    }

    Ok(buffer)
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// An error type for errors of invalid hierarchical encode
/// process.
pub enum InvalidHierarchicalEncodeProcess {
    /// Target slug has extra encoding semantics.
    #[error("Target slug has extra encoding semantics.")]
    TargetSlugHasExtraEncodingSemantics,
}

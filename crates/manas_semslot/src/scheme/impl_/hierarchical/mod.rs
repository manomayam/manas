//! I define [`HierarchicalSemanticSlotEncodingScheme`].

use std::marker::PhantomData;

use manas_http::uri::invariant::NormalAbsoluteHttpUri;
use manas_space::{
    policy::aux::AuxPolicy,
    resource::{slot_id::SolidResourceSlotId, uri::SolidResourceUri},
    SolidStorageSpace,
};

use self::{
    aux::AuxLinkEncodingScheme,
    decoder::{
        decode_from_hierarchical_uri_path, InvalidHierarchicalEncodedResourceSlot, SLASH_CHAR,
    },
    encoder::{encode_to_hierarchical_relative_uri_path, InvalidHierarchicalEncodeProcess},
};
use crate::{process::SlotPathEncodeProcess, scheme::SemanticSlotEncodingScheme};

pub mod aux;

pub mod decoder;
pub mod encoder;

/// An implementation of [`SemanticSlotEncodingScheme`] that encodes
/// resource slot path in resource uri's hierarchical path.
#[derive(Debug, Clone)]
pub struct HierarchicalSemanticSlotEncodingScheme<Space, AuxLinkES>
where
    Space: SolidStorageSpace,
    AuxLinkES: AuxLinkEncodingScheme,
{
    _phantom: PhantomData<fn(Space, AuxLinkES)>,
}

/// Check if storage root uri is valid for hierarchical scheme.
#[inline]
pub fn has_valid_storage_root_uri(space: &impl SolidStorageSpace) -> bool {
    let root_res_uri = space.root_res_uri();

    // Ensure root res uri has no query, and has trailing slash.
    root_res_uri.query_str().is_none() && root_res_uri.as_str().ends_with(SLASH_CHAR)
}

impl<Space, AuxLinkES> SemanticSlotEncodingScheme
    for HierarchicalSemanticSlotEncodingScheme<Space, AuxLinkES>
where
    Space: SolidStorageSpace,
    AuxLinkES:
        AuxLinkEncodingScheme<KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType>,
{
    type Space = Space;

    type EncodeError = HierarchicalSemanticSlotEncodeError;

    type DecodeError = HierarchicalSemanticSlotDecodeError;

    fn encode(
        process: &SlotPathEncodeProcess<Self::Space>,
    ) -> Result<SolidResourceSlotId<Space>, Self::EncodeError> {
        // Ensure storage root uri is valid for hierarchical
        // scheme.
        if !has_valid_storage_root_uri(process.space().as_ref()) {
            return Err(HierarchicalSemanticSlotEncodeError::InvalidStorageRootUri);
        }

        // Compute storage root relative uri path.
        let root_relative_uri_path =
            encode_to_hierarchical_relative_uri_path::<Space, AuxLinkES>(process)?;

        // Prepend storage root uri to relative path, to get res uri.
        let mut res_uri_str = root_relative_uri_path;
        res_uri_str.insert_str(0, process.space().root_res_uri().as_str());

        Ok(SolidResourceSlotId {
            uri: SolidResourceUri::try_new_from(res_uri_str.as_str())
                .expect("Must be valid, as encoded path is normal"),
            space: process.space().clone(),
        })
    }

    fn decode(
        res_slot_id: &SolidResourceSlotId<Space>,
    ) -> Result<SlotPathEncodeProcess<'static, Self::Space>, Self::DecodeError> {
        // Ensure valid storage root uri.
        if !has_valid_storage_root_uri(res_slot_id.space.as_ref()) {
            return Err(HierarchicalSemanticSlotDecodeError::InvalidStorageRootUri);
        }

        // Ensure res uri is hierarchical.
        if res_slot_id.uri.query_str().is_some() {
            return Err(HierarchicalSemanticSlotDecodeError::NonHierarchicalResUri);
        }

        // Ensure res uri is in namespace of storage root uri.
        if let Some(root_relative_uri_path) = res_slot_id
            .uri
            .as_str()
            .strip_prefix(res_slot_id.space.root_res_uri().as_str())
        {
            Ok(decode_from_hierarchical_uri_path::<Space, AuxLinkES>(
                res_slot_id.space.clone(),
                root_relative_uri_path,
            )?)
        } else {
            Err(HierarchicalSemanticSlotDecodeError::ResUriNotInHierarchicalNamespaceOfStorage)
        }
    }

    fn decode_mutex(
        res_slot_id: &SolidResourceSlotId<Self::Space>,
    ) -> Option<(
        SolidResourceSlotId<Self::Space>,
        SlotPathEncodeProcess<'static, Self::Space>,
    )> {
        let res_uri_str = res_slot_id.uri.as_str();

        // Toggle the trailing slash in uri path, and then
        // perform other checks and decoding.
        let mutex_res_uri = res_slot_id
            .uri
            .query_str()
            .is_none()
            .then(|| {
                res_uri_str
                    .strip_suffix(SLASH_CHAR)
                    .and_then(|v| NormalAbsoluteHttpUri::try_new_from(v).ok())
                    .or_else(|| {
                        NormalAbsoluteHttpUri::try_new_from(format!("{}/", res_uri_str).as_str())
                            .ok()
                    })
            })
            .flatten()?;

        let mutex_slot_id = SolidResourceSlotId {
            space: res_slot_id.space.clone(),
            uri: mutex_res_uri,
        };

        Self::decode(&mutex_slot_id)
            .ok()
            .map(|process| (mutex_slot_id, process))
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// An error type for errors in encoding a semantic slot using
/// hierarchical scheme.
pub enum HierarchicalSemanticSlotEncodeError {
    /// Invalid storage root uri.c
    #[error("Storage root uri is not valid for hierarchical scheme.")]
    InvalidStorageRootUri,

    /// Invalid hierarchical encode process.
    #[error("Invalid hierarchical encode process.")]
    InvalidHierarchicalEncodeProcess(#[from] InvalidHierarchicalEncodeProcess),
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// An error type for errors in decoding a semantic slot using
/// hierarchical scheme.
pub enum HierarchicalSemanticSlotDecodeError {
    /// Invalid storage root uri.
    #[error("Storage root uri is not valid for hierarchical scheme.")]
    InvalidStorageRootUri,

    /// Resource uri is non hierarchical.
    #[error("Res uri is not hierarchical.")]
    NonHierarchicalResUri,

    /// Resource uri is not in hierarchical namespace of storage.
    #[error("Resource uri is not in hierarchical namespace of storage.")]
    ResUriNotInHierarchicalNamespaceOfStorage,

    /// Invalid hierarchical encoded resource slot.
    #[error("Invalid hierarchical encoded resource slot.")]
    InvalidHierarchicalEncodedSlot(#[from] InvalidHierarchicalEncodedResourceSlot),
}

/// Tests for encoding and decoding.
#[cfg(feature = "test-utils")]
#[cfg(test)]
mod tests_codec {
    use std::sync::Arc;

    use claims::*;
    use manas_space::{
        mock::*,
        resource::{kind::SolidResourceKind, slot_rel_type::aux_rel_type::mock::*},
    };
    use rstest::*;

    use super::{aux::mock::MockAuxLinkEncodingScheme, *};
    use crate::process::{
        step::mock::SlotPathEncodeStepHint,
        tests_helper::assert_valid_slot_path_encode_process_steps,
    };

    fn assert_decoding_invalid_slot_id_will_error<Space, AuxLinkES>(
        space: Arc<Space>,
        res_uri_str: &'static str,
        expected_error: HierarchicalSemanticSlotDecodeError,
    ) where
        Space: SolidStorageSpace,
        AuxLinkES: AuxLinkEncodingScheme<
            KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType,
        >,
    {
        assert_eq!(
            assert_err!(
                HierarchicalSemanticSlotEncodingScheme::<_, AuxLinkES>::decode(
                    &SolidResourceSlotId {
                        uri: SolidResourceUri::try_new_from(res_uri_str)
                            .expect("Claimed valid uri"),
                        space
                    }
                ),
                "Invalid encoded res uri decoded successfully"
            ),
            expected_error,
            "Unexpected error in decoding invalid encoded slot path"
        );
    }

    fn assert_encoding_invalid_process_will_error<Space, AuxLinkES>(
        space: Arc<Space>,
        encode_process_step_hints: &[SlotPathEncodeStepHint],
        expected_error: HierarchicalSemanticSlotEncodeError,
    ) where
        Space: SolidStorageSpace,
        AuxLinkES: AuxLinkEncodingScheme<
            KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType,
        >,
    {
        let process = assert_valid_slot_path_encode_process_steps(space, encode_process_step_hints);

        assert_eq!(
            assert_err!(
                HierarchicalSemanticSlotEncodingScheme::<_, AuxLinkES>::encode(&process),
                "Invalid slot path encode process encoded successfully"
            ),
            expected_error,
            "Unexpected error in encoding invalid slot path encode process."
        );
    }

    fn assert_valid_roundtrip_expectation<Space, AuxLinkES>(
        space: Arc<Space>,
        res_uri_str: &'static str,
        encode_process_step_hints: &[SlotPathEncodeStepHint],
    ) where
        Space: SolidStorageSpace,
        AuxLinkES: AuxLinkEncodingScheme<
            KnownAuxRelType = <Space::AuxPolicy as AuxPolicy>::KnownAuxRelType,
        >,
    {
        // Get expected process.
        let expected_process =
            assert_valid_slot_path_encode_process_steps(space.clone(), encode_process_step_hints);

        let decoded_process = assert_ok!(
            HierarchicalSemanticSlotEncodingScheme::<_, AuxLinkES>::decode(&SolidResourceSlotId {
                uri: NormalAbsoluteHttpUri::try_new_from(res_uri_str)
                    .expect("Claimed valid res uri str"),
                space
            }),
            "Error in decoding valid claimed res uri."
        );

        // Assert decode works correctly.
        assert_eq!(
            decoded_process, expected_process,
            "Decode expectation not satisfied."
        );

        let round_tripped_res_slot_id = assert_ok!(
            HierarchicalSemanticSlotEncodingScheme::<_, AuxLinkES>::encode(&expected_process),
            "Error in valid round tripping res slot id"
        );

        assert_eq!(
            round_tripped_res_slot_id.uri.as_str(),
            res_uri_str,
            "res uri roundtrip expectation not satisfied."
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/b._aux/containerindex",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkSubjectConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/b._aux/_tc1",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/b._aux/acl/",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/b._aux/_ta1/c/d._aux/acl",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/b._aux/_tc1/._aux/_tc2",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkTargetConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/.._aux/acl",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasNonCleanStepEncoding
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/def/..._aux/_tc1/",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasNonCleanStepEncoding
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/r._aux/abc",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasUnknownAuxRelTypeEncodeToken
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/r._aux/",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/r._aux/._aux/acl",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::HasInvalidAuxRelDelim
        )
    )]
    #[case(
        "http://ex.org/a",
        "http://ex.org/a/r",
        HierarchicalSemanticSlotDecodeError::InvalidStorageRootUri
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/r?abc",
        HierarchicalSemanticSlotDecodeError::NonHierarchicalResUri
    )]
    #[case(
        "http://ex.org/",
        "http://ex2.org/r",
        HierarchicalSemanticSlotDecodeError::ResUriNotInHierarchicalNamespaceOfStorage
    )]
    #[case(
        "http://ex.org/a/b/",
        "http://ex.org/a/c/d",
        HierarchicalSemanticSlotDecodeError::ResUriNotInHierarchicalNamespaceOfStorage
    )]
    fn decoding_invalid_encoded_slot_id_will_error(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] expected_error: HierarchicalSemanticSlotDecodeError,
    ) {
        let space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            space_root_uri_str,
        ));

        assert_decoding_invalid_slot_id_will_error::<_, MockAuxLinkEncodingScheme>(
            space,
            res_uri_str,
            expected_error,
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/def/pqr._aux/_tc1/i1/._aux/_ta1._aux/_tc2/bcd.png._aux/acl",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkCountOutOfLimit
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/image.png._aux/acl._aux/_tc1/",
        HierarchicalSemanticSlotDecodeError::InvalidHierarchicalEncodedSlot(
            InvalidHierarchicalEncodedResourceSlot::AuxLinkCountOutOfLimit
        )
    )]
    fn decoding_invalid_encoded_slot_id_will_error_with_aux_constrained_space(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] expected_error: HierarchicalSemanticSlotDecodeError,
    ) {
        let space = Arc::new(MockSolidStorageSpace::<1>::new_from_valid_root_uri_str(
            space_root_uri_str,
        ));

        assert_decoding_invalid_slot_id_will_error::<_, MockAuxLinkEncodingScheme>(
            space,
            res_uri_str,
            expected_error,
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/a",
        &[],
        HierarchicalSemanticSlotEncodeError::InvalidStorageRootUri
    )]
    #[case(
        "http://ex.org/",
        &[SlotPathEncodeStepHint::Mero("abc._aux", SolidResourceKind::Container)],
        HierarchicalSemanticSlotEncodeError::InvalidHierarchicalEncodeProcess(InvalidHierarchicalEncodeProcess::TargetSlugHasExtraEncodingSemantics)
    )]
    #[case(
        "http://ex.org/",
        &[SlotPathEncodeStepHint::Mero("abc._aux", SolidResourceKind::NonContainer)],
        HierarchicalSemanticSlotEncodeError::InvalidHierarchicalEncodeProcess(InvalidHierarchicalEncodeProcess::TargetSlugHasExtraEncodingSemantics)
    )]
    fn encoding_invalid_process_will_error(
        #[case] space_root_uri_str: &'static str,
        #[case] encode_process_step_hints: &'static [SlotPathEncodeStepHint],
        #[case] expected_error: HierarchicalSemanticSlotEncodeError,
    ) {
        let space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            space_root_uri_str,
        ));

        assert_encoding_invalid_process_will_error::<_, MockAuxLinkEncodingScheme>(
            space,
            encode_process_step_hints,
            expected_error,
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        &[]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a",
        &[SlotPathEncodeStepHint::Mero("a", SolidResourceKind::NonContainer)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/",
        &[SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/acl",
        &[SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        &[SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE)]
    )]
    #[case::storage_root(
        "http://ex.org/",
        "http://ex.org/",
        &[]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a",
        &[SlotPathEncodeStepHint::Mero("a", SolidResourceKind::NonContainer)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a.png",
        &[SlotPathEncodeStepHint::Mero("a.png", SolidResourceKind::NonContainer)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/.thumb",
        &[SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero(".thumb", SolidResourceKind::NonContainer)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/._aux/containerindex",
        &[SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container), SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE)]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/image.png._aux/acl._aux/_tc1/",
        &[
            SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Mero("image.png", SolidResourceKind::NonContainer),
            SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE),
            SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE),
        ]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/io.acl.png/._aux/_ta1",
        &[
            SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Mero("io.acl.png", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Aux(&TA1_REL_TYPE),
        ]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/a/.thumb",
        &[
            SlotPathEncodeStepHint::Mero("a", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Mero(".thumb", SolidResourceKind::NonContainer),
        ]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/def/pqr._aux/_tc1/i1/._aux/_ta1._aux/_tc2/bcd.png._aux/acl",
        &[
            SlotPathEncodeStepHint::Mero("def", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Mero("pqr", SolidResourceKind::NonContainer),
            SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE),
            SlotPathEncodeStepHint::Mero("i1", SolidResourceKind::Container),
            SlotPathEncodeStepHint::Aux(&TA1_REL_TYPE),
            SlotPathEncodeStepHint::Aux(&TC2_REL_TYPE),
            SlotPathEncodeStepHint::Mero("bcd.png", SolidResourceKind::NonContainer),
            SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE),
        ]
    )]
    fn valid_slot_id_round_trips_correctly(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] expected_encode_process_step_hints: &[SlotPathEncodeStepHint],
    ) {
        let space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            space_root_uri_str,
        ));

        assert_valid_roundtrip_expectation::<_, MockAuxLinkEncodingScheme>(
            space,
            res_uri_str,
            expected_encode_process_step_hints,
        );
    }
}

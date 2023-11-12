//! It is a bad idea to encode extra semantics into  opaque
//! resource ds as part of an http engine logic. But it is ok
//! to do so behind a linked architecture abstraction, as an
//! implementation detail for the sake of efficiency to avoid
//! a remote lookup. For such cases, this crate provides a
//! type driven codec for encoding and decoding a solid
//! resource slot path into/from it's slot id.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{borrow::Cow, marker::PhantomData, sync::Arc};

use manas_space::{
    resource::{
        slot::SolidResourceSlot, slot_id::SolidResourceSlotId, slot_path::SolidResourceSlotPath,
        slot_rev_link::SlotRevLink, uri::SolidResourceUri,
    },
    SolidStorageSpace,
};
use smallvec::SmallVec;

use self::{
    process::{
        step::SlotPathEncodeStep, SlotPathEncodeProcess, SlotPathEncodeProcessExtensionError,
    },
    scheme::SemanticSlotEncodingScheme,
};

pub mod process;
pub mod scheme;

/// A type to represent semantic resource slot.
/// It is generic over an encoding scheme
///
/// See module documentation for rational.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticResourceSlot<'p, Space, ES>
where
    Space: SolidStorageSpace,
    ES: SemanticSlotEncodingScheme<Space = Space>,
{
    /// Inner slot.
    inner: SolidResourceSlot<Space>,

    /// slot path encode process.
    slot_path_encode_process: SlotPathEncodeProcess<'p, Space>,

    _phantom_es: PhantomData<ES>,
}

impl<'p, Space, ES> TryFrom<SlotPathEncodeProcess<'p, Space>>
    for SemanticResourceSlot<'p, Space, ES>
where
    Space: SolidStorageSpace,
    ES: SemanticSlotEncodingScheme<Space = Space>,
{
    type Error = ES::EncodeError;

    fn try_from(
        slot_path_encode_process: SlotPathEncodeProcess<'p, Space>,
    ) -> Result<Self, Self::Error> {
        let slot_id = ES::encode(&slot_path_encode_process)?;

        Ok(Self {
            inner: Self::decode_slot(slot_id, &slot_path_encode_process),
            slot_path_encode_process,
            _phantom_es: PhantomData,
        })
    }
}

impl<'p, Space, ProvPathES> From<SemanticResourceSlot<'p, Space, ProvPathES>>
    for SolidResourceSlotPath<'static, Space>
where
    Space: SolidStorageSpace,
    ProvPathES: SemanticSlotEncodingScheme<Space = Space>,
{
    fn from(semslot: SemanticResourceSlot<'p, Space, ProvPathES>) -> Self {
        let mut slots = semslot.path_rev_iter().collect::<Vec<_>>();
        slots.reverse();

        // Safety: codec satisfies required constraints.
        unsafe { SolidResourceSlotPath::new_unchecked(Cow::Owned(slots)) }
    }
}

impl<'p, Space, ES> SemanticResourceSlot<'p, Space, ES>
where
    Space: SolidStorageSpace,
    ES: SemanticSlotEncodingScheme<Space = Space>,
{
    /// Decode resource slot.
    /// Callers must ensure param correspondence.
    fn decode_slot(
        slot_id: SolidResourceSlotId<Space>,
        slot_path_encode_process: &SlotPathEncodeProcess<'_, Space>,
    ) -> SolidResourceSlot<Space> {
        let res_kind = slot_path_encode_process.encoded_target_res_kind();

        let steps_count = slot_path_encode_process.steps().len();

        let slot_rev_link = (steps_count > 0).then(|| {
            let (host_path_encode_process, last_steps) =
                slot_path_encode_process.split_at(steps_count - 1);

            SlotRevLink {
                target: ES::encode(&host_path_encode_process)
                    .expect("Must be valid, as super process is valid")
                    .uri,
                rev_rel_type: last_steps[0].encoded_slot_rel_type(),
            }
        });

        SolidResourceSlot::try_new(slot_id, res_kind, slot_rev_link).expect("Must be valid.")
    }

    /// Try to create new [`SemanticResourceSlot`] with given
    /// resource slot id.
    #[inline]
    pub fn try_new(res_slot_id: SolidResourceSlotId<Space>) -> Result<Self, ES::DecodeError> {
        let slot_path_encode_process = ES::decode(&res_slot_id)?;
        Ok(Self {
            inner: Self::decode_slot(res_slot_id, &slot_path_encode_process),
            slot_path_encode_process,
            _phantom_es: PhantomData,
        })
    }

    /// Try to create new [`SemanticResourceSlot`] for mutex
    /// resource of specified resource.
    #[inline]
    pub fn try_new_mutex(res_slot_id: SolidResourceSlotId<Space>) -> Option<Self> {
        let (res_slot_id, slot_path_encode_process) = ES::decode_mutex(&res_slot_id)?;
        Some(Self {
            inner: Self::decode_slot(res_slot_id, &slot_path_encode_process),
            slot_path_encode_process,
            _phantom_es: PhantomData,
        })
    }

    /// Get the storage space.
    #[inline]
    pub fn space(&self) -> &Arc<Space> {
        self.inner.space()
    }

    /// Get the inner slot.
    #[inline]
    pub fn inner(&self) -> &SolidResourceSlot<Space> {
        &self.inner
    }

    /// Get the uri of the slot resource.
    #[inline]
    pub fn res_uri(&self) -> &SolidResourceUri {
        &self.inner.id().uri
    }

    /// Get aux link subject ids in the slot path.
    pub fn path_aux_link_subject_ids(&self) -> SmallVec<[SolidResourceSlotId<Space>; 1]> {
        self.slot_path_encode_process
            .steps()
            .iter()
            .enumerate()
            .filter_map(|(step_index, step)| {
                if step.is_aux_link_encoding_step() {
                    let (sub_process, _) = self.slot_path_encode_process.split_at(step_index);
                    Some(
                        ES::encode(&sub_process).expect("Must be valid, as super process is valid"),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the semantic slot of the resource's host.
    pub fn host_slot(&self) -> Option<SemanticResourceSlot<'static, Space, ES>> {
        // None, if current slot is root slot.
        if self.inner.is_root_slot() {
            return None;
        }

        let host_slot_path_encode_process = self
            .slot_path_encode_process
            .split_at(self.slot_path_encode_process.steps().len() - 1)
            .0;

        let host_slot_id = ES::encode(&host_slot_path_encode_process)
            .expect("Encoding must succeed for base process");

        Some(SemanticResourceSlot {
            inner: Self::decode_slot(host_slot_id, &host_slot_path_encode_process),
            slot_path_encode_process: host_slot_path_encode_process.into_owned(),
            _phantom_es: PhantomData,
        })
    }

    /// Get the semantic slot of a linked resource.
    pub fn linked(
        &self,
        linked_slot_encode_step: SlotPathEncodeStep<Space>,
    ) -> Result<SemanticResourceSlot<'static, Space, ES>, LinkedResourceSlotEncodeError<ES>> {
        let mut linked_res_slot_path_encode_process =
            self.slot_path_encode_process.clone().into_owned();
        linked_res_slot_path_encode_process.push_step(linked_slot_encode_step)?;

        let linked_res_slot_id = ES::encode(&linked_res_slot_path_encode_process)
            .map_err(|e| LinkedResourceSlotEncodeError::EncodeError(e))?;

        Ok(SemanticResourceSlot {
            inner: Self::decode_slot(linked_res_slot_id, &linked_res_slot_path_encode_process),
            slot_path_encode_process: linked_res_slot_path_encode_process,
            _phantom_es: PhantomData,
        })
    }

    /// Get reverse iterator over slot path.
    pub fn path_rev_iter(&self) -> impl Iterator<Item = SolidResourceSlot<Space>> + '_ {
        let mut next_iter_res_slot_id = Some(self.inner.id().clone());

        (0..=self.slot_path_encode_process.steps().len())
            .rev()
            .map(move |iter_process_len| {
                let (iter_process, _) = self.slot_path_encode_process.split_at(iter_process_len);

                let iter_res_slot = Self::decode_slot(
                    next_iter_res_slot_id
                        .take()
                        .expect("Must be some, as cursor guarantees"),
                    &iter_process,
                );

                next_iter_res_slot_id = iter_res_slot.host_slot_id();

                iter_res_slot
            })
    }

    /// Get semantic slot of the mutex resource.
    pub fn mutex(&self) -> Option<SemanticResourceSlot<'static, Space, ES>> {
        self.slot_path_encode_process
            .mutex()
            .and_then(|mutex_process| SemanticResourceSlot::try_from(mutex_process).ok())
    }

    /// Get the semantic slot that encodes mutex normal
    /// encode process.
    pub fn mutex_normal(&self) -> Option<SemanticResourceSlot<'static, Space, ES>> {
        self.slot_path_encode_process.mutex_normal().try_into().ok()
    }

    /// Get a new identical struct, with lifetime tied to
    /// current one.
    #[inline]
    pub fn to_borrowed(&self) -> SemanticResourceSlot<'_, Space, ES> {
        SemanticResourceSlot {
            inner: self.inner.clone(),
            slot_path_encode_process: self.slot_path_encode_process.to_borrowed(),
            _phantom_es: PhantomData,
        }
    }

    /// Get a new identical struct, with static lifetime.
    #[inline]
    pub fn into_owned(self) -> SemanticResourceSlot<'static, Space, ES> {
        SemanticResourceSlot {
            inner: self.inner,
            slot_path_encode_process: self.slot_path_encode_process.into_owned(),
            _phantom_es: PhantomData,
        }
    }

    /// Get the inner slot, and encode process.
    #[inline]
    pub fn into_parts(self) -> (SolidResourceSlot<Space>, SlotPathEncodeProcess<'p, Space>) {
        (self.inner, self.slot_path_encode_process)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
/// An error type for errors in encoding a linked resource slot.
pub enum LinkedResourceSlotEncodeError<ES: SemanticSlotEncodingScheme> {
    /// Encode process extension error.
    #[error("Error in extending encode process")]
    ProcessExtensionError(#[from] SlotPathEncodeProcessExtensionError),

    /// Encode error.
    #[error("Error in encoding process into resource slot.")]
    EncodeError(ES::EncodeError),
}

impl<ProvPathES> PartialEq for LinkedResourceSlotEncodeError<ProvPathES>
where
    ProvPathES: SemanticSlotEncodingScheme,
    ProvPathES::EncodeError: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ProcessExtensionError(l0), Self::ProcessExtensionError(r0)) => l0 == r0,
            (Self::EncodeError(l0), Self::EncodeError(r0)) => l0 == r0,
            (_, _) => false,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "test-utils")]
mod tests_link_extended {
    use claims::*;
    use manas_space::{
        mock::*,
        resource::{
            kind::SolidResourceKind, slot_id::mock::MockSpaceResourceSlotId,
            slot_rel_type::aux_rel_type::mock::*,
        },
    };
    use rstest::*;

    use super::{
        process::step::mock::SlotPathEncodeStepHint, scheme::mock::MockSemanticSlotEncodingScheme,
        *,
    };
    use crate::scheme::impl_::hierarchical::{
        aux::mock::MockAuxLinkEncodingScheme, encoder::InvalidHierarchicalEncodeProcess,
        HierarchicalSemanticSlotEncodeError, HierarchicalSemanticSlotEncodingScheme,
    };

    fn assert_invalid_linked_slot_encode_step<Space, ProvPathES>(
        semslot: SemanticResourceSlot<'_, Space, ProvPathES>,
        linked_slot_encode_step_hint: SlotPathEncodeStepHint,
        expected_error: LinkedResourceSlotEncodeError<ProvPathES>,
    ) where
        Space: SolidStorageSpace,
        ProvPathES: SemanticSlotEncodingScheme<Space = Space>,
        ProvPathES::EncodeError: PartialEq,
    {
        assert_eq!(
            assert_err!(
                semslot.linked(
                    linked_slot_encode_step_hint
                        .try_into()
                        .expect("Claimed valid step hint")
                ),
                "Invalid claimed linked slot encoding succeeded."
            ),
            expected_error,
            "Unexpected error in linked slot path encoding with invalid claimed link step."
        );
    }

    fn assert_valid_linked_slot_encode_step<Space, ProvPathES>(
        semslot: SemanticResourceSlot<'_, Space, ProvPathES>,
        linked_slot_encode_step_hint: SlotPathEncodeStepHint,
        expected_linked_res_uri_str: &'static str,
    ) where
        Space: SolidStorageSpace,
        ProvPathES: SemanticSlotEncodingScheme<Space = Space>,
        ProvPathES::EncodeError: PartialEq,
    {
        let link_encode_step: SlotPathEncodeStep<Space> = linked_slot_encode_step_hint
            .try_into()
            .expect("Claimed valid step hint");

        let linked_res_semslot = assert_ok!(
            semslot.linked(link_encode_step.clone()),
            "Error in extending encoded slot path with valid claimed link."
        );

        assert_eq!(
            semslot.space().root_res_uri(),
            linked_res_semslot.space().root_res_uri(),
            "slot path extension incorrectly changed space root."
        );

        assert_eq!(
            expected_linked_res_uri_str,
            linked_res_semslot.res_uri().as_str(),
            "Linked res uri expectation not satisfied"
        );

        // Assert round tripped slot link expectations.
        let last_rev_link = linked_res_semslot
            .inner()
            .slot_rev_link()
            .expect("Must be some, as extended with link now.");

        assert_eq!(
            &last_rev_link.target,
            semslot.res_uri(),
            "slot rev link expectation not satisfied."
        );

        assert_eq!(
            last_rev_link.rev_rel_type,
            link_encode_step.encoded_slot_rel_type(),
            "slot rev link expectation not satisfied."
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc",
        SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer),
        LinkedResourceSlotEncodeError::ProcessExtensionError(
            SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/acl",
        SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer),
        LinkedResourceSlotEncodeError::ProcessExtensionError(
            SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc",
        SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE),
        LinkedResourceSlotEncodeError::ProcessExtensionError(
            SlotPathEncodeProcessExtensionError::AuxLinkSubjectConstraintsViolation
        )
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/",
        SlotPathEncodeStepHint::Mero("c._aux", SolidResourceKind::NonContainer),
        LinkedResourceSlotEncodeError::EncodeError(
            HierarchicalSemanticSlotEncodeError::InvalidHierarchicalEncodeProcess(
                InvalidHierarchicalEncodeProcess::TargetSlugHasExtraEncodingSemantics
            )
        )
    )]
    fn invalid_linked_slot_encode_step_will_raise_error(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] linked_slot_encode_step_hint: SlotPathEncodeStepHint,
        #[case] expected_error: LinkedResourceSlotEncodeError<
            HierarchicalSemanticSlotEncodingScheme<
                MockSolidStorageSpace<0>,
                MockAuxLinkEncodingScheme,
            >,
        >,
    ) {
        let semslot = assert_ok!(
            SemanticResourceSlot::try_new(MockSpaceResourceSlotId::<0>::new_from_valid_parts(
                space_root_uri_str,
                res_uri_str
            )),
            "Claimed valid id encoded slot path"
        );

        assert_invalid_linked_slot_encode_step(
            semslot,
            linked_slot_encode_step_hint,
            expected_error,
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/._aux/acl",
        SlotPathEncodeStepHint::Aux(&DESCRIBED_BY_REL_TYPE),
        LinkedResourceSlotEncodeError::ProcessExtensionError(SlotPathEncodeProcessExtensionError::AuxStepCountOutOfLimit)
    )]
    fn invalid_linked_slot_encode_step_will_raise_error_with_constrained_aux_policy(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] linked_slot_encode_step_hint: SlotPathEncodeStepHint,
        #[case] expected_error: LinkedResourceSlotEncodeError<MockSemanticSlotEncodingScheme<1>>,
    ) {
        let semslot = assert_ok!(
            SemanticResourceSlot::try_new(MockSpaceResourceSlotId::<1>::new_from_valid_parts(
                space_root_uri_str,
                res_uri_str
            )),
            "Claimed valid id encoded slot path"
        );

        assert_invalid_linked_slot_encode_step(
            semslot,
            linked_slot_encode_step_hint,
            expected_error,
        );
    }

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer),
        "http://ex.org/abc"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container),
        "http://ex.org/abc/"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        SlotPathEncodeStepHint::Aux(&DESCRIBED_BY_REL_TYPE),
        "http://ex.org/._aux/meta"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE),
        "http://ex.org/._aux/_tc1/"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE),
        "http://ex.org/._aux/containerindex"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/def",
        SlotPathEncodeStepHint::Aux(&DESCRIBED_BY_REL_TYPE),
        "http://ex.org/abc/def._aux/meta"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/def",
        SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE),
        "http://ex.org/abc/def._aux/_tc1/"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/acl",
        SlotPathEncodeStepHint::Aux(&DESCRIBED_BY_REL_TYPE),
        "http://ex.org/._aux/acl._aux/meta"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE),
        "http://ex.org/._aux/_tc1/._aux/acl"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE),
        "http://ex.org/._aux/_tc1/._aux/containerindex"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer),
        "http://ex.org/._aux/_tc1/abc"
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container),
        "http://ex.org/._aux/_tc1/abc/"
    )]
    fn valid_linked_slot_encode_step_works_correctly(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] linked_slot_encode_step_hint: SlotPathEncodeStepHint,
        #[case] expected_linked_res_uri_str: &'static str,
    ) {
        let semslot = assert_ok!(
            SemanticResourceSlot::try_new(MockSpaceResourceSlotId::<0>::new_from_valid_parts(
                space_root_uri_str,
                res_uri_str
            )),
            "Claimed valid semslot"
        );

        assert_valid_linked_slot_encode_step::<_, MockSemanticSlotEncodingScheme<0>>(
            semslot,
            linked_slot_encode_step_hint,
            expected_linked_res_uri_str,
        );
    }
}

#[cfg(feature = "test-utils")]
#[cfg(test)]
mod tests_aux_subjects {
    use claims::*;
    use manas_space::resource::slot_id::mock::MockSpaceResourceSlotId;
    use rstest::*;

    use super::*;
    use crate::scheme::mock::MockSemanticSlotEncodingScheme;

    #[rstest]
    #[case(
        "http://ex.org/",
        "http://ex.org/",
        &[]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc",
        &[]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/",
        &[]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/meta",
        &["http://ex.org/"]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/",
        &["http://ex.org/"]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/containerindex",
        &["http://ex.org/"]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/def._aux/meta",
        &["http://ex.org/abc/def"]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/abc/def._aux/_tc1/cd/pq._aux/acl._aux/meta",
        &[
            "http://ex.org/abc/def",
            "http://ex.org/abc/def._aux/_tc1/cd/pq",
            "http://ex.org/abc/def._aux/_tc1/cd/pq._aux/acl"
        ]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/acl._aux/meta",
        &[
            "http://ex.org/",
            "http://ex.org/._aux/acl"
        ]
    )]
    #[case(
        "http://ex.org/",
        "http://ex.org/._aux/_tc1/._aux/acl",
        &[
            "http://ex.org/",
            "http://ex.org/._aux/_tc1/"
        ]
    )]
    fn path_aux_links_subjects_works_correctly(
        #[case] space_root_uri_str: &'static str,
        #[case] res_uri_str: &'static str,
        #[case] expected_aux_subject_uri_strs: &[&str],
    ) {
        let semslot = assert_ok!(
            SemanticResourceSlot::<_, MockSemanticSlotEncodingScheme<0>>::try_new(
                MockSpaceResourceSlotId::<0>::new_from_valid_parts(space_root_uri_str, res_uri_str)
            ),
            "Claimed valid semslot"
        );

        let aux_subject_ids = semslot.path_aux_link_subject_ids();

        assert_eq!(
            aux_subject_ids.len(),
            expected_aux_subject_uri_strs.len(),
            "Un expected number of aux links."
        );

        assert!(
            aux_subject_ids
                .iter()
                .zip(expected_aux_subject_uri_strs.iter())
                .all(|(res_id, expected_uri_str)| res_id.uri.as_str() == *expected_uri_str),
            "Aux subject uris expectation not satisfied"
        );
    }
}
// TODO path_rev_iter tests.

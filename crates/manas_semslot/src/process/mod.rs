//! I define types for slot path enciding process.
//!

use std::{borrow::Cow, sync::Arc};

use if_chain::if_chain;
use manas_space::{
    policy::aux::AuxPolicy,
    resource::{kind::SolidResourceKind, slot_rel_type::aux_rel_type::known::KnownAuxRelType},
    SolidStorageSpace,
};

use self::step::{SlotPathEncodeStep, SlotPathEncodeStepTranslationError};

pub mod step;

/// A struct that represents encoding-scheme-agnostic process of
/// encoding a resource slot path.
///
/// It represents process in units of [`SlotPathEncodeStep`].
///
/// It is up to an encoding scheme, how to materialize this
/// process.
#[derive(Debug, Clone)]
pub struct SlotPathEncodeProcess<'p, Space>
where
    Space: SolidStorageSpace,
{
    space: Arc<Space>,
    steps: Cow<'p, [SlotPathEncodeStep<Space>]>,
}

impl<'p, Space> SlotPathEncodeProcess<'p, Space>
where
    Space: SolidStorageSpace,
{
    /// Get process to encode root slot.
    #[inline]
    pub fn to_root(space: Arc<Space>) -> Self {
        Self {
            space,
            steps: Vec::new().into(),
        }
    }

    /// Get storage space, this process is defined in.
    #[inline]
    pub fn space(&self) -> &Arc<Space> {
        &self.space
    }

    /// Get encode steps.
    #[inline]
    pub fn steps(&self) -> &[SlotPathEncodeStep<Space>] {
        &self.steps
    }

    /// Split process at given index.
    ///
    /// Returns sub process up to step at `mid` index exclusive,
    /// and slice of remaining steps.
    ///
    /// ## Panics:
    ///
    /// Panics, if `mid` is greater than steps len.
    pub fn split_at(
        &self,
        mid: usize,
    ) -> (
        SlotPathEncodeProcess<'_, Space>,
        &[SlotPathEncodeStep<Space>],
    ) {
        let (sub_process_steps, remaining) = self.steps().split_at(mid);

        (
            SlotPathEncodeProcess {
                space: self.space.clone(),
                steps: Cow::Borrowed(sub_process_steps),
            },
            remaining,
        )
    }

    /// Push step to the process.
    pub fn push_step(
        &mut self,
        step: SlotPathEncodeStep<Space>,
    ) -> Result<(), SlotPathEncodeProcessExtensionError> {
        let is_container_slot = self
            .steps
            .last()
            .map(|last_step| last_step.slotted_res_kind() == SolidResourceKind::Container)
            // Root node is a container.
            .unwrap_or(true);

        match &step {
            SlotPathEncodeStep::Mero { .. } => {
                // Ensure link subject is a container.
                if !is_container_slot {
                    return Err(
                        SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation,
                    );
                }
            }

            SlotPathEncodeStep::Aux { rel_type } => {
                // Ensure aux step count will be with in limit.
                let aux_step_count = self
                    .steps
                    .iter()
                    .filter(|step| step.is_aux_link_encoding_step())
                    .count();

                if_chain! {
                    if let Some(max_aux_links) =  <Space::AuxPolicy as AuxPolicy>::PROV_PATH_MAX_AUX_LINKS;
                    if aux_step_count >= max_aux_links.into();

                    then {
                        return Err(SlotPathEncodeProcessExtensionError::AuxStepCountOutOfLimit);
                    }
                }

                // Ensure subject res kind is honoured.
                if !rel_type
                    .allowed_subject_res_kinds()
                    .contains(&if is_container_slot {
                        SolidResourceKind::Container
                    } else {
                        SolidResourceKind::NonContainer
                    })
                {
                    return Err(
                        SlotPathEncodeProcessExtensionError::AuxLinkSubjectConstraintsViolation,
                    );
                }
            }
        }

        // Push step.
        match &mut self.steps {
            // If backed by slice, make it a vec.
            Cow::Borrowed(steps_slice) => {
                let mut steps = Vec::from(*steps_slice);
                steps.push(step);
                self.steps = Cow::Owned(steps);
            }

            Cow::Owned(steps) => steps.push(step),
        }

        Ok(())
    }

    /// Two slot path encode processes, that differ only in
    /// their last step's slotted res kind are mutually
    /// exclusive.
    pub fn mutex(&self) -> Option<SlotPathEncodeProcess<'static, Space>> {
        if_chain!(
            if let Some(last_step) = self.steps.last();
            if let Some(mutex_step) = last_step.mutex_step();

            then {
                let mut mutex_process = self.split_at(self.steps.len() - 1).0.into_owned();

                mutex_process.push_step(mutex_step).expect("Must be valid");
                Some(mutex_process)
            }
            else {
                None
            }
        )
    }

    /// Get mutex normal slot path encode process.
    /// It always ensures last step slots a container if it is
    /// mero step.
    pub fn mutex_normal(&self) -> SlotPathEncodeProcess<'static, Space> {
        if_chain!(
            if let Some(last_step) = self.steps.last();
            if last_step.is_mero_link_encoding_step() && last_step.slotted_res_kind() != SolidResourceKind::Container;

            then {
                self.mutex().expect("Must be some.")
            } else {
                self.clone().into_owned()
            }
        )
    }

    /// Translate process to encode parallel slot path in other
    /// space.
    pub fn translate_parallel<OSpace: SolidStorageSpace>(
        &self,
        other_space: Arc<OSpace>,
    ) -> Result<SlotPathEncodeProcess<'static, OSpace>, SlotPathEncodeProcessTranslationError> {
        let mut parallel_process = SlotPathEncodeProcess::to_root(other_space);

        for (i, step) in self.steps().iter().enumerate() {
            let parallel_step = step.translate_parallel().map_err(|e| {
                SlotPathEncodeProcessTranslationError::StepIsNotTranslatableToOtherSpace(e, i)
            })?;

            parallel_process.push_step(parallel_step).map_err(|e| {
                SlotPathEncodeProcessTranslationError::StepIsNotValidInOtherSpace(e, i)
            })?;
        }

        Ok(parallel_process)
    }

    /// Get owned version of process.
    #[inline]
    pub fn into_owned(self) -> SlotPathEncodeProcess<'static, Space> {
        SlotPathEncodeProcess {
            space: self.space,
            steps: Cow::Owned(self.steps.into_owned()),
        }
    }

    /// Get borrowed process.
    #[inline]
    pub fn to_borrowed(&self) -> SlotPathEncodeProcess<'_, Space> {
        SlotPathEncodeProcess {
            space: self.space.clone(),
            steps: Cow::Borrowed(self.steps()),
        }
    }

    /// Get encoded kind of the slotted resource.
    #[inline]
    pub fn encoded_target_res_kind(&self) -> SolidResourceKind {
        self.steps()
            .last()
            .map(|last_step| last_step.slotted_res_kind())
            // Storage root is always a container.
            .unwrap_or(SolidResourceKind::Container)
    }
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
/// An error type to represent errors in extending slot path
/// encode process.
pub enum SlotPathEncodeProcessExtensionError {
    /// Mero link subject constraints violation.
    #[error("Mero rel subject constraints violation.")]
    MeroLinkSubjectConstraintsViolation,

    /// Aux link subject constraints violation.
    #[error("Aux rel subject constraints violation.")]
    AuxLinkSubjectConstraintsViolation,

    /// Aux steps count out of limit.
    #[error("Aux step count out of limit.")]
    AuxStepCountOutOfLimit,
}

#[derive(Debug, Clone, thiserror::Error)]
/// An error type to represent errors in translating slot path
/// encode process.
pub enum SlotPathEncodeProcessTranslationError {
    /// Step is not translatable to other process.
    #[error("Step at index {1} is not translatable to other space.")]
    StepIsNotTranslatableToOtherSpace(SlotPathEncodeStepTranslationError, usize),

    /// Step is invalid in other space.
    #[error("Step at index {1} is not valid in other space.")]
    StepIsNotValidInOtherSpace(SlotPathEncodeProcessExtensionError, usize),
}

impl<'p, Space> PartialEq for SlotPathEncodeProcess<'p, Space>
where
    Space: SolidStorageSpace,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.space.root_res_uri() == other.space.root_res_uri() && self.steps == other.steps
    }
}

#[cfg(feature = "test-utils")]
/// A module of test utils.
pub mod tests_helper {
    use std::sync::Arc;

    use claims::*;

    use super::{
        step::{mock::SlotPathEncodeStepHint, SlotPathEncodeStep},
        SlotPathEncodeProcess, SlotPathEncodeProcessExtensionError,
    };
    use crate::SolidStorageSpace;

    /// Assert slot path encode process steps corresponding to given step_hints are invalid.
    pub fn assert_invalid_slot_path_encode_process_steps<Space: SolidStorageSpace>(
        space: Arc<Space>,
        step_hints: &[SlotPathEncodeStepHint],
        invalid_step_index: usize,
        expected_error: SlotPathEncodeProcessExtensionError,
    ) {
        let mut process = SlotPathEncodeProcess::to_root(space);

        for (step_index, step_hint) in step_hints.iter().enumerate() {
            let step: SlotPathEncodeStep<Space> =
                assert_ok!(step_hint.clone().try_into(), "Claimed valid step hint");

            let push_result = process.push_step(step);

            if step_index == invalid_step_index {
                assert_eq!(
                    assert_err!(
                        push_result,
                        "Invalid claimed slot path encode step pushed successfully."
                    ),
                    expected_error,
                    "Unexpected error in pushing invalid slot path encode step."
                );

                break;
            } else {
                assert_ok!(
                    push_result,
                    "Error in pushing valid claimed slot path encode step at index {}.",
                    step_index
                );
            }
        }
    }

    /// Assert slot path encode process steps corresponding to
    /// given step_hints is valid.
    pub fn assert_valid_slot_path_encode_process_steps<Space: SolidStorageSpace>(
        space: Arc<Space>,
        step_hints: &[SlotPathEncodeStepHint],
    ) -> SlotPathEncodeProcess<Space> {
        let mut process = SlotPathEncodeProcess::to_root(space);

        for (step_index, step_hint) in step_hints.iter().enumerate() {
            let step: SlotPathEncodeStep<Space> =
                assert_ok!(step_hint.clone().try_into(), "Claimed valid step hint");

            assert_ok!(
                process.push_step(step),
                "Error in pushing valid claimed slot path encode step at index {}.",
                step_index
            );
        }

        process
    }

    /// Assert encode process steps expectation satisfies.
    pub fn assert_matches_slot_path_encode_steps<Space: SolidStorageSpace>(
        process: &SlotPathEncodeProcess<'_, Space>,
        expected_step_hints: &[SlotPathEncodeStepHint],
    ) {
        let expected_steps = expected_step_hints
            .iter()
            .map(|step_hint| {
                assert_ok!(
                    TryInto::<SlotPathEncodeStep<Space>>::try_into(step_hint.clone()),
                    "Claimed valid encode step hint"
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            process.steps(),
            &expected_steps,
            "slot encode steps didn't matched expectation."
        );
    }
}

#[cfg(feature = "test-utils")]
#[cfg(test)]
mod tests_push {
    use manas_space::{mock::*, resource::slot_rel_type::aux_rel_type::mock::*};
    use rstest::*;

    use super::{step::mock::SlotPathEncodeStepHint, tests_helper::*, *};

    fn assert_valid_push_step_expectation<Space: SolidStorageSpace>(
        space: Arc<Space>,
        step_hints: &[SlotPathEncodeStepHint],
        expected_error_context: Option<(usize, SlotPathEncodeProcessExtensionError)>,
    ) {
        if let Some((invalid_step_index, expected_error)) = expected_error_context {
            assert_invalid_slot_path_encode_process_steps(
                space,
                step_hints,
                invalid_step_index,
                expected_error,
            );
        } else {
            assert_valid_slot_path_encode_process_steps(space, step_hints);
        }
    }

    #[rstest]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer)],
        Some((1, SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation)),
    )]
    #[case(
        &[SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer)],
        Some((1, SlotPathEncodeProcessExtensionError::MeroLinkSubjectConstraintsViolation))
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer), SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE)],
        Some((1, SlotPathEncodeProcessExtensionError::AuxLinkSubjectConstraintsViolation)),
    )]
    #[case(
        &[SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer)],
        None,
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer)],
        None,
    )]
    #[case(
        &[SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE), SlotPathEncodeStepHint::Aux(&CONTAINER_INDEX_REL_TYPE)],
        None,
    )]
    fn push_step_works_correctly(
        #[case] step_hints: &[SlotPathEncodeStepHint],
        #[case] expected_error_context: Option<(usize, SlotPathEncodeProcessExtensionError)>,
    ) {
        let mock_space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            "http://ex.org/",
        ));

        assert_valid_push_step_expectation(mock_space, step_hints, expected_error_context);
    }

    #[rstest]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)],
        None,
    )]
    #[case(
        &[SlotPathEncodeStepHint::Aux(&TC1_REL_TYPE), SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)],
        Some((1, SlotPathEncodeProcessExtensionError::AuxStepCountOutOfLimit)),
    )]
    fn push_step_works_correctly_with_constrained_aux_policy(
        #[case] step_hints: &[SlotPathEncodeStepHint],
        #[case] expected_error_context: Option<(usize, SlotPathEncodeProcessExtensionError)>,
    ) {
        let mock_space = Arc::new(MockSolidStorageSpace::<1>::new_from_valid_root_uri_str(
            "http://ex.org/",
        ));

        assert_valid_push_step_expectation(mock_space, step_hints, expected_error_context);
    }
}

#[cfg(feature = "test-utils")]
#[cfg(test)]
mod tests_mutex_process {
    use manas_space::{
        mock::MockSolidStorageSpace, resource::slot_rel_type::aux_rel_type::ACL_REL_TYPE,
    };
    use rstest::*;

    use super::{step::mock::SlotPathEncodeStepHint, tests_helper::*, *};

    #[rstest]
    #[case(
        &[],
        None,
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer)],
        Some(vec![SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container)]),
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::Container)],
        Some(vec![SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::NonContainer)]),
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)],
        None,
    )]
    fn mutex_prov_path_encode_process_works_correctly(
        #[case] step_hints: &[SlotPathEncodeStepHint],
        #[case] expected_mutex_step_hints: Option<Vec<SlotPathEncodeStepHint>>,
    ) {
        let mock_space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            "http://ex.org/",
        ));

        let process = assert_valid_slot_path_encode_process_steps(mock_space, step_hints);

        let mutex_process = process.mutex();

        match (mutex_process, expected_mutex_step_hints) {
            (None, None) => (),
            (Some(p), Some(e)) => assert_matches_slot_path_encode_steps(&p, &e),
            (l, r) => panic!(
                "mutex process expectation not satisfied. Got {:?}, expected {:?}",
                l, r
            ),
        }
    }

    #[rstest]
    #[case(
        &[],
        &[],
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::NonContainer)],
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container)],
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::Container)],
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Mero("def", SolidResourceKind::Container)],
    )]
    #[case(
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)],
        &[SlotPathEncodeStepHint::Mero("abc", SolidResourceKind::Container), SlotPathEncodeStepHint::Aux(&ACL_REL_TYPE)],
    )]
    fn mutex_normal_slot_path_encode_process_works_correctly(
        #[case] step_hints: &[SlotPathEncodeStepHint],
        #[case] expected_mutex_normal_step_hints: &[SlotPathEncodeStepHint],
    ) {
        let mock_space = Arc::new(MockSolidStorageSpace::<0>::new_from_valid_root_uri_str(
            "http://ex.org/",
        ));

        let process = assert_valid_slot_path_encode_process_steps(mock_space, step_hints);

        let mutex_normal_process = process.mutex_normal();

        assert_matches_slot_path_encode_steps(
            &mutex_normal_process,
            expected_mutex_normal_step_hints,
        );
    }
}

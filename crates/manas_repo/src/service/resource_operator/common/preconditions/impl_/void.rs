//! I define an implementation of [`Preconditions`] that is no-op.
//!

use manas_http::representation::metadata::RepresentationMetadata;

use crate::service::resource_operator::common::preconditions::{
    Preconditions, PreconditionsEvalResult,
};

impl Preconditions for () {
    #[inline]
    fn are_trivial(&self) -> bool {
        true
    }

    #[inline]
    fn evaluate(
        &self,
        _base_rep_validators: Option<&RepresentationMetadata>,
    ) -> Box<dyn PreconditionsEvalResult> {
        Box::new(())
    }
}

impl PreconditionsEvalResult for () {
    #[inline]
    fn are_satisfied(&self) -> bool {
        true
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

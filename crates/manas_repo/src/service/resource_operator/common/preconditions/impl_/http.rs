//! I define an implementation of [`Preconditions`] to use with in http method services.
//!

use std::ops::Deref;

use headers::HeaderMap;
use http::{
    header::{IF_MATCH, IF_MODIFIED_SINCE, IF_NONE_MATCH, IF_RANGE, IF_UNMODIFIED_SINCE},
    Method,
};
use manas_http::{
    conditional_req::{PreconditionsEvaluator, PreconditionsResolvedAction},
    representation::metadata::{KDerivedETag, KLastModified, RepresentationMetadata},
};
use typed_record::TypedRecord;

use crate::service::resource_operator::common::preconditions::{
    Preconditions, PreconditionsEvalResult,
};

/// an implementation of [`Preconditions`] to use with in http method services.
/// It evaluates http precondition headers against resource state for given method.
#[derive(Debug, Clone)]
pub struct HttpPreconditions {
    /// Http method, for which preconditions needs to be evaluated.
    pub method: Method,

    /// Http preconditions.
    pub preconditions: HeaderMap,
}

/// An implementation of [`PreconditionsEvalResult`] for [`HttpPreconditions`].
#[derive(Debug, Clone)]
pub struct HttpPreconditionsEvalResult(pub PreconditionsResolvedAction);

impl Deref for HttpPreconditionsEvalResult {
    type Target = PreconditionsResolvedAction;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PreconditionsResolvedAction> for HttpPreconditionsEvalResult {
    #[inline]
    fn from(value: PreconditionsResolvedAction) -> Self {
        Self(value)
    }
}

impl Preconditions for HttpPreconditions {
    #[inline]
    fn are_trivial(&self) -> bool {
        !(self.preconditions.contains_key(&IF_MATCH)
            || self.preconditions.contains_key(&IF_NONE_MATCH)
            || self.preconditions.contains_key(&IF_MODIFIED_SINCE)
            || self.preconditions.contains_key(&IF_UNMODIFIED_SINCE)
            || self.preconditions.contains_key(&IF_RANGE))
    }

    #[inline]
    fn evaluate(
        &self,
        base_rep_validators: Option<&RepresentationMetadata>,
    ) -> Box<dyn PreconditionsEvalResult> {
        Box::new(self.evaluate_raw(base_rep_validators))
    }
}

impl HttpPreconditions {
    /// Evaluate preconditions against given base rep validators.
    pub fn evaluate_raw(
        &self,
        base_rep_validators: Option<&RepresentationMetadata>,
    ) -> HttpPreconditionsEvalResult {
        let evaluator = PreconditionsEvaluator {
            req_method: self.method.clone(),
            req_headers: &self.preconditions,
            res_is_represented: base_rep_validators.is_some(),
            res_last_modified: base_rep_validators.and_then(|m| m.get_rv::<KLastModified>()),
            selected_rep_etag: base_rep_validators
                .and_then(|m| m.get_rv::<KDerivedETag>())
                .map(|detag| detag.as_ref()),
        };

        HttpPreconditionsEvalResult(evaluator.evaluate())
    }
}

impl PreconditionsEvalResult for HttpPreconditionsEvalResult {
    #[inline]
    fn are_satisfied(&self) -> bool {
        self.0 == PreconditionsResolvedAction::ApplyMethod
    }

    #[inline]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

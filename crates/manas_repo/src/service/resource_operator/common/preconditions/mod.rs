//! I define traits for representing resource operation preconditions.
//!

use std::{any::Any, fmt::Debug};

use dyn_clone::{clone_trait_object, DynClone};
use manas_http::representation::metadata::RepresentationMetadata;
use typed_record::TypedRecordKey;

pub mod impl_;

/// A trait for representing resource operation preconditions.
pub trait Preconditions: Debug + Send + Sync + 'static + DynClone {
    /// Get if preconditions are trivial.
    /// Trivial pre conditions doesn't necessitates resource
    /// state for evaluation.
    fn are_trivial(&self) -> bool;

    /// Evaluate preconditions against given base representation validators.
    // NOTE: Used boxed trait object to simplify
    // generics in implementations.
    fn evaluate(
        &self,
        base_rep_validators: Option<&RepresentationMetadata>,
    ) -> Box<dyn PreconditionsEvalResult>;
}

clone_trait_object!(Preconditions);

/// A trait for representing preconditions evaluation result.
pub trait PreconditionsEvalResult: Debug + Send + Sync + 'static + DynClone {
    /// Get if preconditions are satisfied.
    fn are_satisfied(&self) -> bool;

    /// Cast self to `dyn Any`.
    fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(PreconditionsEvalResult);

/// A typed record key for pre conditions eval result.
#[derive(Debug, Clone)]
pub struct KPreconditionsEvalResult {}

impl TypedRecordKey for KPreconditionsEvalResult {
    type Value = Box<dyn PreconditionsEvalResult>;
}

/// A typed record key for rep validators against which
/// preconditions are validated..
#[derive(Debug, Clone)]
pub struct KEvaluatedRepValidators {}

impl TypedRecordKey for KEvaluatedRepValidators {
    type Value = Option<RepresentationMetadata>;
}

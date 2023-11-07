//! I define predicates to check if rdf syntax is a
//! dataset-encoding-syntax.
//!

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::syntax::{RdfSyntax, JSON_LD, N_QUADS, TRIG};

/// A type representing a predicate over an rdf-syntax, stating that syntax encodes rdf-datasets.
#[derive(Debug, Clone)]
pub struct IsDatasetEncoding {}

impl IsDatasetEncoding {
    /// Slice of all dataset encoding syntaxes.
    const ALL_RAW: &'static [RdfSyntax] = &[N_QUADS, TRIG, JSON_LD];
}

impl Predicate<RdfSyntax> for IsDatasetEncoding {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsDatasetEncoding".into()
    }
}

impl PurePredicate<RdfSyntax> for IsDatasetEncoding {}

impl SyncEvaluablePredicate<RdfSyntax> for IsDatasetEncoding {
    type EvalError = NotADatasetEncodingSyntax;

    #[inline]
    fn evaluate_for(sub: &RdfSyntax) -> Result<(), Self::EvalError> {
        if Self::ALL_RAW.contains(sub) {
            Ok(())
        } else {
            Err(NotADatasetEncodingSyntax(*sub))
        }
    }
}

/// An error type for non dataset-encoding-syntaxes.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Syntax is not a dataset encoding syntax.")]
pub struct NotADatasetEncodingSyntax(RdfSyntax);

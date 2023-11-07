//! I define predicates to check if rdf syntax is a dynsyn-serializable-syntax.
//!

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::syntax::{RdfSyntax, JSON_LD, N_QUADS, N_TRIPLES, RDF_XML, TRIG, TURTLE};

/// A type representing a predicate over an rdf-syntax, stating that syntax is dynsyn serializable.
#[derive(Debug, Clone)]
pub struct IsDynSynSerializable {}

impl IsDynSynSerializable {
    /// Slice of all dynsyn serializable syntaxes.
    const ALL_RAW: &'static [RdfSyntax] = &[
        N_QUADS,
        TRIG,
        N_TRIPLES,
        TURTLE,
        #[cfg(feature = "rdf-xml")]
        RDF_XML,
        #[cfg(feature = "jsonld")]
        JSON_LD,
    ];
}

impl Predicate<RdfSyntax> for IsDynSynSerializable {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsDynSynSerializable".into()
    }
}

impl PurePredicate<RdfSyntax> for IsDynSynSerializable {}

impl SyncEvaluablePredicate<RdfSyntax> for IsDynSynSerializable {
    type EvalError = NotADynsynSerializableSyntax;

    #[inline]
    fn evaluate_for(sub: &RdfSyntax) -> Result<(), Self::EvalError> {
        if Self::ALL_RAW.contains(sub) {
            Ok(())
        } else {
            Err(NotADynsynSerializableSyntax(*sub))
        }
    }
}

/// An error type for non dynsyn-serializable-syntaxes.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Syntax is not a dynsyn serializable syntax.")]
pub struct NotADynsynSerializableSyntax(RdfSyntax);

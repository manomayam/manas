//! I define predicates to check if rdf syntax is a dynsyn-parsable-syntax.
//!

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::syntax::{RdfSyntax, JSON_LD, N_QUADS, N_TRIPLES, RDF_XML, TRIG, TURTLE};

/// A type representing a predicate over an rdf-syntax, stating that syntax is dynsyn parsable.
#[derive(Debug, Clone)]
pub struct IsDynSynParsable {}

impl IsDynSynParsable {
    /// Slice of all dynsyn parsable syntaxes.
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

impl Predicate<RdfSyntax> for IsDynSynParsable {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsDynSynParsable".into()
    }
}

impl PurePredicate<RdfSyntax> for IsDynSynParsable {}

impl SyncEvaluablePredicate<RdfSyntax> for IsDynSynParsable {
    type EvalError = NotADynsynParsableSyntax;

    #[inline]
    fn evaluate_for(sub: &RdfSyntax) -> Result<(), Self::EvalError> {
        if Self::ALL_RAW.contains(sub) {
            Ok(())
        } else {
            Err(NotADynsynParsableSyntax(*sub))
        }
    }
}

/// An error type for non dynsyn-parsable-syntaxes.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Syntax is not a dynsyn parsable syntax.")]
pub struct NotADynsynParsableSyntax(RdfSyntax);

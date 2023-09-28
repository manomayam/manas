//! I define predicates to check if rdf syntax is a graph-encoding-syntax.
//!

use gdp_rs::predicate::{Predicate, PurePredicate, SyncEvaluablePredicate};

use crate::syntax::{RdfSyntax, N_TRIPLES, RDF_XML, TURTLE};

/// A type representing a predicate over an rdf-syntax, stating that syntax encodes rdf-graphs.
#[derive(Debug, Clone)]
pub struct IsGraphEncoding;

impl IsGraphEncoding {
    /// Slice of all graph encoding syntaxes.
    const ALL_RAW: &'static [RdfSyntax] = &[
        N_TRIPLES,
        TURTLE,
        #[cfg(feature = "rdf_xml")]
        RDF_XML,
    ];
}

impl Predicate<RdfSyntax> for IsGraphEncoding {
    fn label() -> std::borrow::Cow<'static, str> {
        "IsGraphEncoding".into()
    }
}

impl PurePredicate<RdfSyntax> for IsGraphEncoding {}

impl SyncEvaluablePredicate<RdfSyntax> for IsGraphEncoding {
    type EvalError = NotAGraphEncodingSyntax;

    #[inline]
    fn evaluate_for(sub: &RdfSyntax) -> Result<(), Self::EvalError> {
        if Self::ALL_RAW.contains(sub) {
            Ok(())
        } else {
            Err(NotAGraphEncodingSyntax(*sub))
        }
    }
}

/// An error type for non graph-encoding-syntaxes.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Syntax is not a graph encoding syntax.")]
pub struct NotAGraphEncodingSyntax(RdfSyntax);

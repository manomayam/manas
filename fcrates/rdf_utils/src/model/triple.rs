//! I define models for rdf triples.
//!

use super::term::ArcTerm;

/// A triple type with [`ArcTerm`]as term.
pub type ArcTriple = [ArcTerm; 3];

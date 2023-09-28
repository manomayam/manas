//! I define models for rdf quads.
//!

use sophia_api::quad::Spog;

use super::term::ArcTerm;

/// A quad type with [`ArcTerm`]as term.
pub type ArcQuad = Spog<ArcTerm>;

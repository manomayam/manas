//! I define types and statics for dynsyn quads-parsable rdf syntaxes.
//!

use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use crate::syntax::{
    predicate::{IsDatasetEncoding, IsDynSynParsable},
    RdfSyntax, N_QUADS, TRIG,
};

/// Type alias for rdf syntax that can encode dataset, and can be parsable by dynsyn.
pub type QuadsParsableSyntax =
    Proven<RdfSyntax, AllOf<RdfSyntax, HList![IsDatasetEncoding, IsDynSynParsable]>>;

/// n-quads quads parsable syntax.
pub static QP_N_QUADS: QuadsParsableSyntax = unsafe { Proven::new_unchecked(N_QUADS) };

/// trig quads parsable syntax.
pub static QP_TRIG: QuadsParsableSyntax = unsafe { Proven::new_unchecked(TRIG) };

/// List of all quads parsable syntaxes.
pub static QP_ALL: &[QuadsParsableSyntax] = &[QP_N_QUADS, QP_TRIG];

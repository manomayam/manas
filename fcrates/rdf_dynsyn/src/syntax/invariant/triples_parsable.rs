//! I define types and statics for dynsyn triples parsable rdf syntaxes.
//!

use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use crate::syntax::{
    predicate::{IsDynSynParsable, IsGraphEncoding},
    RdfSyntax, N_TRIPLES, RDF_XML, TURTLE,
};

/// Type alias for rdf syntax that can encode a graph, and can be parsable by dynsyn.
pub type TriplesParsableSyntax =
    Proven<RdfSyntax, AllOf<RdfSyntax, HList![IsGraphEncoding, IsDynSynParsable]>>;

/// n-triples triples parsable syntax.
pub static TP_N_TRIPLES: TriplesParsableSyntax = unsafe { Proven::new_unchecked(N_TRIPLES) };

/// turtle triples parsable syntax.
pub static TP_TURTLE: TriplesParsableSyntax = unsafe { Proven::new_unchecked(TURTLE) };

/// rdf/xml triples parsable syntax.
#[cfg(feature = "rdf_xml")]
pub static TP_RDF_XML: TriplesParsableSyntax = unsafe { Proven::new_unchecked(RDF_XML) };

/// List of all triples parsable syntaxes.
pub static TP_ALL: &[TriplesParsableSyntax] = &[
    TP_N_TRIPLES,
    TP_TURTLE,
    #[cfg(feature = "rdf_xml")]
    TP_RDF_XML,
];

//! I define types and statics for dynsyn triples serializable rdf syntaxes.
//!
use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use crate::syntax::{
    predicate::{IsDynSynSerializable, IsGraphEncoding},
    RdfSyntax, N_TRIPLES, RDF_XML, TURTLE,
};

/// Type alias for rdf syntax that can encode a graph, and can be serializable by dynsyn.
pub type TriplesSerializableSyntax =
    Proven<RdfSyntax, AllOf<RdfSyntax, HList![IsGraphEncoding, IsDynSynSerializable]>>;

/// n-triples triples serializable syntax.
pub static TS_N_TRIPLES: TriplesSerializableSyntax = unsafe { Proven::new_unchecked(N_TRIPLES) };

/// turtle triples serializable syntax.
pub static TS_TURTLE: TriplesSerializableSyntax = unsafe { Proven::new_unchecked(TURTLE) };

/// rdf/xml triples serializable syntax.
#[cfg_attr(doc_cfg, doc(cfg(feature = "rdf-xml")))]
#[cfg(feature = "rdf-xml")]
pub static TS_RDF_XML: TriplesSerializableSyntax = unsafe { Proven::new_unchecked(RDF_XML) };

/// List of all triples serializable syntaxes.
pub static TS_ALL: &[TriplesSerializableSyntax] = &[
    TS_N_TRIPLES,
    TS_TURTLE,
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rdf-xml")))]
    #[cfg(feature = "rdf-xml")]
    TS_RDF_XML,
];

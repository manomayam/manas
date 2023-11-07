//! I define types and statics for dynsyn quads-serializable rdf syntaxes.
//!

use frunk_core::HList;
use gdp_rs::{predicate::impl_::all_of::AllOf, Proven};

use crate::syntax::{
    predicate::{IsDatasetEncoding, IsDynSynSerializable},
    RdfSyntax, JSON_LD, N_QUADS, TRIG,
};

/// Type alias for rdf syntax that can encode dataset, and can be serializable by dynsyn.
pub type QuadsSerializableSyntax =
    Proven<RdfSyntax, AllOf<RdfSyntax, HList![IsDatasetEncoding, IsDynSynSerializable]>>;

/// n-quads quads serializable syntax.
pub static QS_N_QUADS: QuadsSerializableSyntax = unsafe { Proven::new_unchecked(N_QUADS) };

/// trig quads serializable syntax.
pub static QS_TRIG: QuadsSerializableSyntax = unsafe { Proven::new_unchecked(TRIG) };

#[cfg_attr(doc_cfg, doc(cfg(feature = "jsonld")))]
#[cfg(feature = "jsonld")]
/// json-ld quads serializable syntax.
pub static QS_JSON_LD: QuadsSerializableSyntax = unsafe { Proven::new_unchecked(JSON_LD) };

/// List of all quads serializable syntaxes.
pub static QS_ALL: &[QuadsSerializableSyntax] = &[
    QS_N_QUADS,
    QS_TRIG,
    #[cfg_attr(doc_cfg, doc(cfg(feature = "jsonld")))]
    #[cfg(feature = "jsonld")]
    QS_JSON_LD,
];

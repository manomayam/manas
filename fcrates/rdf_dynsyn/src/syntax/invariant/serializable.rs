//! I define types and statics for dynsyn serializable rdf syntaxes.
//!

use gdp_rs::Proven;
use mime::Mime;

use crate::{
    correspondence::Correspondent,
    syntax::{
        predicate::IsDynSynSerializable, RdfSyntax, N_QUADS, N_TRIPLES, RDF_XML, TRIG, TURTLE,
    },
};

/// Type alias for dynsyn serializable syntax.
pub type DynSynSerializableSyntax = Proven<RdfSyntax, IsDynSynSerializable>;

impl TryFrom<&Mime> for Correspondent<DynSynSerializableSyntax> {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(content_type: &Mime) -> Result<Self, Self::Error> {
        // Resolve corresponding syntax.
        let correspondent_syntax = Correspondent::<RdfSyntax>::try_from(content_type)?;

        // Ensure syntax is serializable.
        let serializable_syntax = DynSynSerializableSyntax::try_new(correspondent_syntax.value)?;

        Ok(Correspondent {
            value: serializable_syntax,
            // Set proper correspondence level.
            is_total: correspondent_syntax.is_total,
        })
    }
}

/// n-triples DynSyn serializable syntax.
pub static S_N_TRIPLES: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(N_TRIPLES) };

/// turtle DynSyn serializable syntax.
pub static S_TURTLE: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(TURTLE) };

/// rdf/xml DynSyn serializable syntax.
#[cfg(feature = "rdf_xml")]
pub static S_RDF_XML: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(RDF_XML) };

/// n-quads DynSyn serializable syntax.
pub static S_N_QUADS: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(N_QUADS) };

/// trig DynSyn serializable syntax.
pub static S_TRIG: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(TRIG) };

// /// json-ld DynSyn serializable syntax.
// pub static S_JSON_LD: DynSynSerializableSyntax = unsafe { Proven::new_unchecked(JSON_LD) };

/// List of all DynSyn serializable syntaxes.
pub static S_ALL: &[DynSynSerializableSyntax] = &[
    S_N_TRIPLES,
    S_TURTLE,
    #[cfg(feature = "rdf_xml")]
    S_RDF_XML,
    S_N_QUADS,
    S_TRIG,
    // S_JSON_LD,
];

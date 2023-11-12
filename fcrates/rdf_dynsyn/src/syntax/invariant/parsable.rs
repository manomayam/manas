//! I define types and statics for dynsyn parsable rdf syntaxes.
//!

use gdp_rs::Proven;
use mime::Mime;

use crate::{
    correspondence::Correspondent,
    syntax::{predicate::IsDynSynParsable, RdfSyntax, N_QUADS, N_TRIPLES, RDF_XML, TRIG, TURTLE},
};

/// Type alias for dynsyn parsable syntax.
pub type DynSynParsableSyntax = Proven<RdfSyntax, IsDynSynParsable>;

impl TryFrom<&Mime> for Correspondent<DynSynParsableSyntax> {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(content_type: &Mime) -> Result<Self, Self::Error> {
        // Resolve corresponding syntax.
        let correspondent_syntax = Correspondent::<RdfSyntax>::try_from(content_type)?;

        // Ensure syntax is parsable.
        let parsable_syntax = DynSynParsableSyntax::try_new(correspondent_syntax.value)?;

        Ok(Correspondent {
            value: parsable_syntax,
            // Set proper correspondence level.
            is_total: correspondent_syntax.is_total,
        })
    }
}

/// n-triples DynSyn parsable syntax.
pub static P_N_TRIPLES: DynSynParsableSyntax = unsafe { Proven::new_unchecked(N_TRIPLES) };

/// turtle DynSyn parsable syntax.
pub static P_TURTLE: DynSynParsableSyntax = unsafe { Proven::new_unchecked(TURTLE) };

/// rdf/xml DynSyn parsable syntax.
#[cfg(feature = "rdf-xml")]
pub static P_RDF_XML: DynSynParsableSyntax = unsafe { Proven::new_unchecked(RDF_XML) };

/// n-quads DynSyn parsable syntax.
pub static P_N_QUADS: DynSynParsableSyntax = unsafe { Proven::new_unchecked(N_QUADS) };

/// trig DynSyn parsable syntax.
pub static P_TRIG: DynSynParsableSyntax = unsafe { Proven::new_unchecked(TRIG) };

/// json-ld DynSyn parsable syntax.
#[cfg(feature = "jsonld")]
pub static P_JSON_LD: DynSynParsableSyntax = unsafe {
    use crate::syntax::JSON_LD;
    Proven::new_unchecked(JSON_LD)
};

/// List of all DynSyn parsable syntaxes.
pub static P_ALL: &[DynSynParsableSyntax] = &[
    P_N_TRIPLES,
    P_TURTLE,
    #[cfg(feature = "rdf-xml")]
    P_RDF_XML,
    P_N_QUADS,
    P_TRIG,
    #[cfg(feature = "jsonld")]
    P_JSON_LD,
];

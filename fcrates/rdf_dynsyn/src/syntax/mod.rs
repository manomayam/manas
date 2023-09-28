//! I define struct for rdf concrete syntax. It also exports few syntax constants.
//!

use std::fmt::Display;

pub mod invariant;
pub mod predicate;

// pub mod parsable;
// pub mod serializable;

/// A concrete rdf syntax is a syntax in which we can serialize rdf graphs or datasets unambiguously. see [https://www.w3.org/TR/rdf11-concepts/#rdf-documents](https://www.w3.org/TR/rdf11-concepts/#rdf-documents)
///
/// [`syntax`](self) module exports pre-defined constants for most of common rdf syntaxes.
///
/// At runtime We can resolve known syntaxes from corresponding [`Mime`](mime::Mime), or [`FileExtension`](crate::file_extension::FileExtension`) values using [`Correspondent::<RdfSyntax>::try_from`](crate::correspondence::Correspondent) method.
///
/// Example:
///
///  ```
/// use rdf_dynsyn::syntax::{self, RdfSyntax};
/// use rdf_dynsyn::media_type;
/// use rdf_dynsyn::file_extension::FileExtension;
/// use rdf_dynsyn::correspondence::Correspondent;
///
/// # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
/// // You can create a `mime::Mime` in what ever way.
/// let mt = &*media_type::APPLICATION_N_QUADS;
/// let correspondent_syntax = Correspondent::<RdfSyntax>::try_from(mt)?;
/// assert_eq!(correspondent_syntax.value, syntax::N_QUADS);
///
/// let extn = FileExtension(String::from("ttl").into());
/// let correspondent_syntax = Correspondent::<RdfSyntax>::try_from(&extn)?;
/// assert_eq!(correspondent_syntax.value, syntax::TURTLE);
/// # Ok(())
/// # }
/// # fn main() {try_main().unwrap();}
/// ```
///

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RdfSyntax(pub &'static str);

impl Display for RdfSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

///RDF 1.1 Turtle: Terse RDF Triple Language
///
/// Spec: [http://www.w3.org/TR/turtle/](http://www.w3.org/TR/turtle/)
pub const TURTLE: RdfSyntax = RdfSyntax("http://www.w3.org/TR/turtle/");

///RDF 1.1 XML Syntax
///
/// Spec [https://www.w3.org/TR/rdf-syntax-grammar/](https://www.w3.org/TR/rdf-syntax-grammar/)
pub const RDF_XML: RdfSyntax = RdfSyntax("http://www.w3.org/TR/rdf-syntax-grammar/");

///Notation3 (N3): A readable RDF syntax
///
/// Spec [https://www.w3.org/TeamSubmission/n3/](https://www.w3.org/TeamSubmission/n3/)
pub const N3: RdfSyntax = RdfSyntax("http://www.w3.org/TeamSubmission/n3/");

/// RDF 1.1 N-Triples: A line-based syntax for an RDF graph
///
/// Spec: [https://www.w3.org/TR/n-triples/](https://www.w3.org/TR/n-triples/)
pub const N_TRIPLES: RdfSyntax = RdfSyntax("http://www.w3.org/TR/n-triples/");

/// RDF 1.1 N-Quads: A line-based syntax for RDF datasets
///
/// Spec: [https://www.w3.org/TR/n-quads/](https://www.w3.org/TR/n-quads/)
pub const N_QUADS: RdfSyntax = RdfSyntax("http://www.w3.org/TR/n-quads/");

/// OWL 2 Web Ontology Language XML Serialization (Second Edition)
///
/// Spec: [http://www.w3.org/TR/owl2-xml-serialization/](http://www.w3.org/TR/owl2-xml-serialization/)
pub const OWL2_XML: RdfSyntax = RdfSyntax("https://www.w3.org/TR/owl2-xml-serialization/");

/// OWL 2 Web Ontology Language Manchester Syntax (Second Edition)
///
/// Spec: [http://www.w3.org/TR/owl2-manchester-syntax/](http://www.w3.org/TR/owl2-manchester-syntax/)
pub const OWL2_MANCHESTER: RdfSyntax = RdfSyntax("https://www.w3.org/TR/owl2-manchester-syntax/");

/// RDF 1.1 TriG: RDF Dataset Language
///
/// Spec: [https://www.w3.org/TR/trig/](https://www.w3.org/TR/trig/)
pub const TRIG: RdfSyntax = RdfSyntax("https://www.w3.org/TR/trig/");

/// JSON-LD 1.1: A JSON-based Serialization for Linked Data
///
/// Spec: [https://www.w3.org/TR/json-ld/](https://www.w3.org/TR/json-ld/)
pub const JSON_LD: RdfSyntax = RdfSyntax("https://www.w3.org/TR/json-ld/");

/// XHTML+RDFa 1.1 - Third Edition: Support for RDFa via XHTML Modularization
///
/// Spec: [https://www.w3.org/TR/xhtml-rdfa/](https://www.w3.org/TR/xhtml-rdfa/)
pub const XHTML_RDFA: RdfSyntax = RdfSyntax("https://www.w3.org/TR/xhtml-rdfa/");

///  HTML+RDFa 1.1 - Second Edition: Support for RDFa in HTML4 and HTML5
///
///  Spec: [https://www.w3.org/TR/html-rdfa/](https://www.w3.org/TR/html-rdfa/)
pub const HTML_RDFA: RdfSyntax = RdfSyntax("https://www.w3.org/TR/html-rdfa/");

/// An error indicating, given syntax is not known/supported in given context
#[derive(Debug, thiserror::Error)]
#[error("Un supported syntax: {0}")]
pub struct UnKnownSyntaxError(pub RdfSyntax);

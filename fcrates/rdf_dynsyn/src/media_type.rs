//! I define statics for media types corresponding different rdf concrete syntaxes.
//!

use mime::Mime;
use once_cell::sync::Lazy;

/// application/ld+json
pub static APPLICATION_JSON_LD: Lazy<Mime> = Lazy::new(|| "application/ld+json".parse().unwrap());

/// application/n-quads
pub static APPLICATION_N_QUADS: Lazy<Mime> = Lazy::new(|| "application/n-quads".parse().unwrap());

/// application/n-triples
pub static APPLICATION_N_TRIPLES: Lazy<Mime> =
    Lazy::new(|| "application/n-triples".parse().unwrap());

/// application/owl+xml
pub static APPLICATION_OWL_XML: Lazy<Mime> = Lazy::new(|| "application/owl+xml".parse().unwrap());

/// application/rdf+xml
pub static APPLICATION_RDF_XML: Lazy<Mime> = Lazy::new(|| "application/rdf+xml".parse().unwrap());

/// application/trig
pub static APPLICATION_TRIG: Lazy<Mime> = Lazy::new(|| "application/trig".parse().unwrap());

/// application/xhtml+xml
pub static APPLICATION_XHTML_XML: Lazy<Mime> =
    Lazy::new(|| "application/xhtml+xml".parse().unwrap());

/// text/html
pub static TEXT_HTML: Lazy<Mime> = Lazy::new(|| mime::TEXT_HTML);

/// text/n3
pub static TEXT_N3: Lazy<Mime> = Lazy::new(|| "text/n3".parse().unwrap());

/// text/owl-manchester
pub static TEXT_OWL_MANCHESTER: Lazy<Mime> = Lazy::new(|| "text/owl-manchester".parse().unwrap());

/// text/turtle
pub static TEXT_TURTLE: Lazy<Mime> = Lazy::new(|| "text/turtle".parse().unwrap());

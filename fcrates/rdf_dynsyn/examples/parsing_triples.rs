use std::collections::HashSet;

use rdf_dynsyn::{parser::triples::*, syntax::invariant::triples_parsable::TP_TURTLE};
use sophia_api::{
    graph::Graph,
    parser::TripleParser,
    prelude::Iri,
    source::TripleSource,
    term::{matcher::Any, SimpleTerm},
};

pub fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let parser_factory = DynSynTripleParserFactory::default();

    let turtle_doc = r#"
     @prefix : <http://example.org/ns/> .
     <#me> :knows [ a :Person ; :name "Alice" ].
 "#;
    let doc_base_iri = Iri::new_unchecked("http://localhost/ex".to_owned());

    // A `DynSynQuadParser<BoxTerm>` instance, configured for trig syntax.
    let parser = parser_factory.new_parser(TP_TURTLE, Some(doc_base_iri));
    let mut graph = HashSet::<[SimpleTerm; 3]>::new();
    let c = parser.parse_str(turtle_doc).add_to_graph(&mut graph)?;

    assert_eq!(c, 3);
    assert!(graph
        .triples_matching(
            Some(Iri::new_unchecked("http://localhost/ex#me")),
            Some(Iri::new_unchecked("http://example.org/ns/knows")),
            Any,
        )
        .next()
        .is_some());

    Ok(())
}

fn main() {
    try_main().unwrap();
}

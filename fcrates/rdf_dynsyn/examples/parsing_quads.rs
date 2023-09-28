use std::collections::HashSet;

use rdf_dynsyn::{parser::quads::*, syntax::invariant::quads_parsable::QP_TRIG};
use sophia_api::{
    dataset::Dataset,
    parser::QuadParser,
    prelude::Iri,
    quad::Spog,
    source::QuadSource,
    term::{matcher::Any, SimpleTerm},
};

pub fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let parser_factory = DynSynQuadParserFactory::default();

    let trig_doc = r#"
     @prefix : <http://example.org/ns/> .
     <#g1> {
         <#me> :knows _:alice.
     }
     <#g2> {
         _:alice a :Person ; :name "Alice".
     }
 "#;
    let doc_base_iri = Iri::new_unchecked("http://localhost/ex".to_owned());

    // A `DynSynQuadParser instance, configured for trig syntax.
    let parser = parser_factory.new_parser(QP_TRIG, Some(doc_base_iri));
    let mut dataset = HashSet::<Spog<SimpleTerm>>::new();
    let c = parser.parse_str(trig_doc).add_to_dataset(&mut dataset)?;

    assert_eq!(c, 3);
    assert!(dataset
        .quads_matching(
            Some(Iri::new_unchecked("http://localhost/ex#me")),
            Some(Iri::new_unchecked("http://example.org/ns/knows")),
            Any,
            Some(Some(Iri::new_unchecked("http://localhost/ex#g1"))),
        )
        .next()
        .is_some());
    Ok(())
}

fn main() {
    try_main().unwrap();
}

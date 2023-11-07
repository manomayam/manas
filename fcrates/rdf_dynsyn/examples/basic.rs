use std::{collections::HashSet, str::FromStr};

use gdp_rs::proven::TryProven;
use mime::Mime;
use rdf_dynsyn::{
    correspondence::Correspondent, parser::triples::*, serializer::triples::*, syntax::RdfSyntax,
};
use sophia_api::{
    graph::MutableGraph,
    ns::Namespace,
    parser::TripleParser,
    serializer::{Stringifier, TripleSerializer},
    source::TripleSource,
    term::SimpleTerm,
};

pub fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    //  let's say following are input params we got dynamically.
    // source media_type, content of source doc, and target media_type
    let src_doc_media_type = "text/turtle";
    let tgt_doc_media_type = "application/rdf+xml";
    let src_doc_content = r#"
        @prefix : <http://example.org/>.
        @prefix foaf: <http://xmlns.com/foaf/0.1/>.

        :alice foaf:name "Alice";
            foaf:mbox <mailto:alice@work.example> .

        :bob foaf:name "Bob".
    "#;

    // resolve syntaxes for media_types. Or one can use static constants exported by `syntax` module,
    let src_doc_syntax =
        Correspondent::<RdfSyntax>::try_from(&Mime::from_str(src_doc_media_type)?)?.value;
    let tgt_doc_syntax =
        Correspondent::<RdfSyntax>::try_from(&Mime::from_str(tgt_doc_media_type)?)?.value;

    // get parser for source syntax
    let parser_factory = DynSynTripleParserFactory::default();
    let parser = parser_factory.new_parser(src_doc_syntax.try_proven()?, None);

    // parse to a graph
    let mut graph: HashSet<[SimpleTerm; 3]> =
        parser.parse_str(src_doc_content).collect_triples()?;

    let ex = Namespace::new("http://example.org/")?;
    let foaf = Namespace::new("http://xmlns.com/foaf/0.1/")?;

    // mutate graph
    graph.insert_triple([&ex.get("bob")?, &foaf.get("knows")?, &ex.get("alice")?])?;

    // get serializer for target syntax
    let serializer_factory = DynSynTripleSerializerFactory::new(Default::default()); // Here we can pass optional formatting options. see documentation.

    let mut serializer = serializer_factory.new_stringifier(tgt_doc_syntax.try_proven()?);
    let serialized_doc = serializer.serialize_graph(&graph)?.as_str();

    println!("The resulting graph\n{}", serialized_doc);

    Ok(())
}
fn main() {
    try_main().unwrap();
}

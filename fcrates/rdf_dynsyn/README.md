# rdf_dynsyn

This crate provides sophia-compatible and sophia-based rdf 
parsers/serializers, that can be instantiated against any of 
supported syntaxes dynamically at run time.

## Why?
Although sophia provides specialized parsers/serializers for 
each syntax, we have to know document syntax at code-time to 
practically use them. In many cases of web, we may know syntax 
of a doc only at runtime, like from content-type, file-extn, 
etc. As each specialized parser parses to corresponding stream 
types, etc.. it will be difficult to work with them in such 
dynamic cases. For Handling such cases this crate provides 
well-tested abstractions, that integrates into sophia eco-system.

## Getting Started

Following is a short example on how to get syntax from media-types/
file-extensions, and instantiate parser for detected syntax, 
parse content,mutate it  and serialize back into desired syntax. 
Also see examples for more.

```rust
use std::{collections::HashSet, str::FromStr};

use mime::Mime;
use sophia_api::{
    graph::MutableGraph,
    ns::Namespace,
    parser::TripleParser,
    serializer::{Stringifier, TripleSerializer},
    source::TripleSource,
    term::SimpleTerm,
};

use gdp_rs::proven::TryProven;
use rdf_dynsyn::{
    correspondence::Correspondent, parser::triples::*, serializer::triples::*, syntax::RdfSyntax,
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
    let serializer_factory = DynSynTripleSerializerFactory::new(None); // Here we can pass optional formatting options. see documentation.

    let mut serializer = serializer_factory.new_stringifier(tgt_doc_syntax.try_proven()?);
    let serialized_doc = serializer.serialize_graph(&graph)?.as_str();

    println!("The resulting graph\n{}", serialized_doc);

    Ok(())
}
fn main() {
    try_main().unwrap();
}
``````

License: MIT OR Apache-2.0

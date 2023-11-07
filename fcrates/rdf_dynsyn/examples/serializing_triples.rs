use rdf_dynsyn::{
    serializer::{config::DynSynSerializerConfig, triples::DynSynTripleSerializerFactory},
    syntax::invariant::triples_serializable::{TS_N_TRIPLES, TS_TURTLE},
};
use sophia_api::{
    ns::{rdf, Namespace},
    serializer::{Stringifier, TripleSerializer},
    term::{SimpleTerm, Term},
};
use sophia_turtle::serializer::turtle::TurtleConfig;

pub fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    // A config that holds configurations for different serialization syntaxes.
    let serializer_config =
        DynSynSerializerConfig::default().with_turtle_config(TurtleConfig::new().with_pretty(true));

    let serializer_factory = DynSynTripleSerializerFactory::new(serializer_config);

    let schema_org = Namespace::new("http://schema.org/")?;
    let example_org = Namespace::new("http://example.org/")?;

    // create a graph to serialize.
    let graph: Vec<[SimpleTerm; 3]> = vec![
        [
            example_org.get("me")?.into_term(),
            rdf::type_.into_term(),
            schema_org.get("Person")?.into_term(),
        ],
        [
            example_org.get("me")?.into_term(),
            schema_org.get("name")?.into_term(),
            "My-name".into_term(),
        ],
    ];

    let mut turtle_serializer = serializer_factory.new_stringifier(TS_TURTLE);
    turtle_serializer.serialize_graph(&graph)?;
    // get to string
    let _turtle_doc = turtle_serializer.as_str();

    let mut nt_serializer = serializer_factory.new_stringifier(TS_N_TRIPLES);
    nt_serializer.serialize_graph(&graph)?;
    let _nt_doc = nt_serializer.as_str();

    Ok(())
}
fn main() {
    try_main().unwrap();
}

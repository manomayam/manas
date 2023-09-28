use rdf_dynsyn::{serializer::quads::*, syntax::invariant::quads_serializable::*, ConfigMap};
use sophia_api::{
    ns::{rdf, Namespace},
    quad::Spog,
    serializer::{QuadSerializer, Stringifier},
    term::{SimpleTerm, Term},
};
use sophia_turtle::serializer::trig::TrigConfig;

pub fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    // A configmap that holds *optional* configurations for different serialization syntaxes.
    let mut serializer_config_map = ConfigMap::new();
    // add optional configurations to config_map
    serializer_config_map.insert::<TrigConfig>(TrigConfig::new().with_pretty(true));

    let serializer_factory = DynSynQuadSerializerFactory::new(Some(serializer_config_map));

    let schema_org = Namespace::new("http://schema.org/")?;
    let example_org = Namespace::new("http://example.org/")?;

    // create a dataset to serialize.
    let dataset: Vec<Spog<SimpleTerm>> = vec![
        (
            [
                example_org.get("me")?.into_term(),
                rdf::type_.into_term(),
                schema_org.get("Person")?.into_term(),
            ],
            None,
        ),
        (
            [
                example_org.get("me")?.into_term(),
                schema_org.get("name")?.into_term(),
                "My-name".into_term(),
            ],
            Some(example_org.get("")?.into_term()),
        ),
    ];

    let mut trig_serializer = serializer_factory.new_stringifier(QS_TRIG);
    trig_serializer.serialize_dataset(&dataset)?;
    // get to string
    let _trig_doc = trig_serializer.as_str();

    let mut nq_serializer = serializer_factory.new_stringifier(QS_N_QUADS);
    nq_serializer.serialize_dataset(&dataset)?;
    let _nq_doc = nq_serializer.as_str();

    Ok(())
}
fn main() {
    try_main().unwrap();
}

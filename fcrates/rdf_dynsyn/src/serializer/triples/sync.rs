use std::{fmt::Debug, io};

use sophia_api::{
    serializer::{Stringifier, TripleSerializer},
    source::{StreamResult, TripleSource},
};
use sophia_turtle::serializer::{
    nt::{NtConfig, NtSerializer},
    turtle::{TurtleConfig, TurtleSerializer},
};
#[cfg(feature = "rdf_xml")]
use sophia_xml::serializer::{RdfXmlConfig, RdfXmlSerializer};

use super::factory::DynSynTripleSerializerFactory;
use crate::syntax::{self, invariant::triples_serializable::TriplesSerializableSyntax};

/// This is a sum-type that wraps around different
/// triple-serializers from sophia.
pub(crate) enum InnerTripleSerializer<W: io::Write> {
    NTriples(NtSerializer<W>),
    Turtle(TurtleSerializer<W>),
    #[cfg(feature = "rdf_xml")]
    RdfXml(RdfXmlSerializer<W>),
}

impl<W: io::Write> Debug for InnerTripleSerializer<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NTriples(_) => f.debug_tuple("NTriples").finish(),
            Self::Turtle(_) => f.debug_tuple("Turtle").finish(),
            #[cfg(feature = "rdf_xml")]
            Self::RdfXml(_) => f.debug_tuple("RdfXml").finish(),
        }
    }
}

/// A [`TripleSerializer`], that can be instantiated at run time
/// against any of supported rdf-syntaxes. We can get it's tuned instance from
/// [`DynSynTripleSerializerFactory::new_serializer`] factory method.
///
/// It can currently serialize triple-sources/datasets into
/// documents in any of concrete_syntaxes: [`n-triples`](crate::syntax::invariant::triples_serializable::TS_N_TRIPLES),
/// [`turtle`](crate::syntax::invariant::triples_serializable::TS_TURTLE), [`rdf-xml`](crate::syntax::invariant::triples_serializable::TS_RDF_XML). Other syntaxes that
/// cannot represent triples are not supported
///
/// For each supported serialization syntax, it also supports
/// corresponding formatting options that sophia supports.
///
/// Example:
///
pub struct DynSynTripleSerializer<W: io::Write>(InnerTripleSerializer<W>);

impl<W: io::Write> DynSynTripleSerializer<W> {
    #[inline]
    pub(crate) fn new(inner_serializer: InnerTripleSerializer<W>) -> Self {
        Self(inner_serializer)
    }
}

impl<W: io::Write> TripleSerializer for DynSynTripleSerializer<W> {
    type Error = io::Error;

    fn serialize_triples<TS>(
        &mut self,
        source: TS,
    ) -> StreamResult<&mut Self, TS::Error, Self::Error>
    where
        TS: TripleSource,
        Self: Sized,
    {
        match &mut self.0 {
            InnerTripleSerializer::NTriples(s) => match s.serialize_triples(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            },
            InnerTripleSerializer::Turtle(s) => match s.serialize_triples(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            },
            #[cfg(feature = "rdf_xml")]
            InnerTripleSerializer::RdfXml(s) => match s.serialize_triples(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            },
        }
    }
}

impl Stringifier for DynSynTripleSerializer<Vec<u8>> {
    fn as_utf8(&self) -> &[u8] {
        match &self.0 {
            InnerTripleSerializer::NTriples(s) => s.as_utf8(),
            InnerTripleSerializer::Turtle(s) => s.as_utf8(),
            #[cfg(feature = "rdf_xml")]
            InnerTripleSerializer::RdfXml(s) => s.as_utf8(),
        }
    }
}

impl DynSynTripleSerializerFactory {
    /// Create new [`DynSynTripleSerializer`] instance, for given `syntax_`, `write`,
    pub fn new_serializer<W: io::Write>(
        &self,
        syntax_: TriplesSerializableSyntax,
        write: W,
    ) -> DynSynTripleSerializer<W> {
        match syntax_.into_subject() {
            syntax::N_TRIPLES => DynSynTripleSerializer::new(InnerTripleSerializer::NTriples(
                NtSerializer::new_with_config(write, self.get_config::<NtConfig>()),
            )),
            syntax::TURTLE => DynSynTripleSerializer::new(InnerTripleSerializer::Turtle(
                TurtleSerializer::new_with_config(write, self.get_config::<TurtleConfig>()),
            )),
            #[cfg(feature = "rdf_xml")]
            syntax::RDF_XML => DynSynTripleSerializer::new(InnerTripleSerializer::RdfXml(
                RdfXmlSerializer::new_with_config(write, self.get_config::<RdfXmlConfig>()),
            )),

            // All triples serializable syntaxes addressed.
            _ => unreachable!(),
        }
    }

    /// Create new [`DynSynTripleSerializer`] instance, that can be stringified after serialization, for given `syntax_`.
    #[inline]
    pub fn new_stringifier(
        &self,
        syntax_: TriplesSerializableSyntax,
    ) -> DynSynTripleSerializer<Vec<u8>> {
        self.new_serializer(syntax_, Vec::new())
    }
}

/// --------------------------------------------
///                                  tests
/// --------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gdp_rs::Proven;
    use once_cell::sync::Lazy;
    use rstest::*;
    use sophia_api::{
        graph::Graph,
        parser::TripleParser,
        serializer::{Stringifier, TripleSerializer},
        term::SimpleTerm,
    };
    use sophia_turtle::serializer::{nt::NtConfig, turtle::TurtleConfig};

    use super::*;
    use crate::{
        parser::triples::DynSynTripleParserFactory,
        serializer::test_data::{TESTS_NTRIPLES, TESTS_RDF_XML, TESTS_TURTLE},
        syntax::invariant::triples_serializable::*,
        tests::TRACING,
        ConfigMap,
    };

    static SERIALIZER_FACTORY: Lazy<DynSynTripleSerializerFactory> =
        Lazy::new(|| DynSynTripleSerializerFactory::new(None));

    static SERIALIZER_FACTORY_WITH_PRETTY_CONFIG: Lazy<DynSynTripleSerializerFactory> =
        Lazy::new(|| {
            let mut config_map = ConfigMap::new();
            config_map.insert::<TurtleConfig>(TurtleConfig::new().with_pretty(true));
            config_map.insert::<NtConfig>(NtConfig::default());

            #[cfg(feature = "rdf_xml")]
            config_map.insert::<RdfXmlConfig>(RdfXmlConfig::default());

            DynSynTripleSerializerFactory::new(Some(config_map))
        });

    /// As DynSyn parsers can be non-cyclically tested, we can use them here.
    static TRIPLE_PARSER_FACTORY: Lazy<DynSynTripleParserFactory> =
        Lazy::new(DynSynTripleParserFactory::default);

    #[rstest]
    #[case(TS_TURTLE, TESTS_TURTLE[0], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[1], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[2], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[3], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[4], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[5], false)]
    #[case(TS_TURTLE, TESTS_TURTLE[0], true)]
    #[case(TS_TURTLE, TESTS_TURTLE[1], true)]
    #[case(TS_TURTLE, TESTS_TURTLE[2], true)]
    #[case(TS_TURTLE, TESTS_TURTLE[3], true)]
    #[case(TS_TURTLE, TESTS_TURTLE[4], true)]
    #[case(TS_TURTLE, TESTS_TURTLE[5], true)]
    #[case(TS_N_TRIPLES, TESTS_NTRIPLES[0], false)]
    #[case(TS_N_TRIPLES, TESTS_NTRIPLES[0], true)]
    #[case(TS_RDF_XML, TESTS_RDF_XML[0], false)]
    #[case(TS_RDF_XML, TESTS_RDF_XML[0], true)]
    pub fn correctly_roundtrips_for_syntax(
        #[case] syntax_: TriplesSerializableSyntax,
        #[case] rdf_doc: &str,
        #[case] pretty: bool,
    ) {
        Lazy::force(&TRACING);
        let parser = TRIPLE_PARSER_FACTORY
            .new_parser(Proven::try_new(syntax_.into_subject()).unwrap(), None);
        let g1: HashSet<[SimpleTerm; 3]> = parser.parse_str(rdf_doc).collect_triples().unwrap();

        let factory = if pretty {
            &SERIALIZER_FACTORY_WITH_PRETTY_CONFIG
        } else {
            &SERIALIZER_FACTORY
        };

        let out = factory
            .new_stringifier(syntax_)
            .serialize_triples(g1.triples())
            .unwrap()
            .to_string();
        let g2: HashSet<[SimpleTerm; 3]> = parser.parse_str(&out).collect_triples().unwrap();
        assert!(sophia_isomorphism::isomorphic_graphs(&g1, &g2).unwrap());
    }
}

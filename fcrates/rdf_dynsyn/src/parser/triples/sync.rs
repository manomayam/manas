use std::io::BufRead;

use sophia_api::prelude::{Iri, TripleParser};
use sophia_turtle::parser::{nt::NTriplesParser, turtle::TurtleParser};
#[cfg(feature = "rdf_xml")]
use sophia_xml::parser::RdfXmlParser;

use super::{factory::DynSynTripleParserFactory, source::DynSynTripleSource};
use crate::syntax::{self, invariant::triples_parsable::TriplesParsableSyntax};

/// This is a sum-type that wraps around different triple-parsers from sophia.
#[derive(Debug, Clone)]
pub enum InnerTripleParser {
    NTriples(NTriplesParser),
    Turtle(TurtleParser),
    #[cfg(feature = "rdf_xml")]
    RdfXml(RdfXmlParser),
}

impl From<NTriplesParser> for InnerTripleParser {
    #[inline]
    fn from(p: NTriplesParser) -> Self {
        Self::NTriples(p)
    }
}

impl From<TurtleParser> for InnerTripleParser {
    #[inline]
    fn from(p: TurtleParser) -> Self {
        Self::Turtle(p)
    }
}

#[cfg(feature = "rdf_xml")]
impl From<RdfXmlParser> for InnerTripleParser {
    #[inline]
    fn from(p: RdfXmlParser) -> Self {
        Self::RdfXml(p)
    }
}

impl InnerTripleParser {
    /// Create a sum-parser for given syntax.
    pub fn new(syntax_: TriplesParsableSyntax, base_iri: Option<Iri<String>>) -> Self {
        match syntax_.into_subject() {
            syntax::N_TRIPLES => NTriplesParser {}.into(),
            syntax::TURTLE => TurtleParser { base: base_iri }.into(),
            #[cfg(feature = "rdf_xml")]
            syntax::RDF_XML => RdfXmlParser { base: base_iri }.into(),
            // All triple parsable syntaxes are addressed.
            _ => unreachable!(),
        }
    }
}

/// This parser implements [`sophia_api::parser::TripleParser`]
/// trait, and can be instantiated at runtime against any of
/// supported syntaxes using [`DynSynTripleParserFactory`]
/// factory.
///
/// It can currently parse triples from documents in any of
/// concrete_syntaxes: [`n-triples`](crate::syntax::N_TRIPLES),
/// [`turtle`](crate::syntax::TURTLE), [`rdf-xml`](crate::syntax::RDF_XML). For docs in any of these
/// syntaxes, this parser will stream triples through
/// [`DynSynTripleSource`] instance.
///
#[derive(Debug, Clone)]
pub struct DynSynTripleParser(InnerTripleParser);

impl DynSynTripleParser {
    /// Create a new parser with given params.
    #[inline]
    pub fn new(syntax_: TriplesParsableSyntax, base_iri: Option<Iri<String>>) -> Self {
        Self(InnerTripleParser::new(syntax_, base_iri))
    }
}

impl<R> TripleParser<R> for DynSynTripleParser
where
    R: BufRead,
{
    type Source = DynSynTripleSource<R>;

    fn parse(&self, data: R) -> Self::Source {
        match &self.0 {
            InnerTripleParser::NTriples(p) => DynSynTripleSource(p.parse(data).into()),
            InnerTripleParser::Turtle(p) => DynSynTripleSource(p.parse(data).into()),
            #[cfg(feature = "rdf_xml")]
            InnerTripleParser::RdfXml(p) => DynSynTripleSource(p.parse(data).into()),
        }
    }
}

impl DynSynTripleParserFactory {
    /// Create new [`DynSynTripleParser`] instance, for given
    /// `syntax_`, `base_iri`.
    #[inline]
    pub fn new_parser(
        &self,
        syntax_: TriplesParsableSyntax,
        base_iri: Option<Iri<String>>,
    ) -> DynSynTripleParser {
        DynSynTripleParser::new(syntax_, base_iri)
    }
}

// ----------------------------------------
//                                      tests
// ----------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use once_cell::sync::Lazy;
    use sophia_api::{
        parser::{IntoParsable, TripleParser},
        source::TripleSource,
        term::SimpleTerm,
    };
    use sophia_isomorphism::isomorphic_graphs;
    use sophia_turtle::parser::{nt::NTriplesParser, turtle::TurtleParser};

    use super::*;
    use crate::{
        parser::test_data::*,
        syntax::invariant::triples_parsable::{TP_N_TRIPLES, TP_RDF_XML, TP_TURTLE},
        tests::TRACING,
    };

    static DYNSYN_TRIPLE_PARSER_FACTORY: Lazy<DynSynTripleParserFactory> =
        Lazy::new(DynSynTripleParserFactory::default);

    fn check_graph_parse_isomorphism<'b, B, P1, P2>(p1: &P1, p2: &P2, qs: &'b str)
    where
        P1: TripleParser<B>,
        P2: TripleParser<B>,
        &'b str: IntoParsable<Target = B>,
    {
        let mut g1 = HashSet::<[SimpleTerm; 3]>::new();
        p1.parse_str(qs).add_to_graph(&mut g1).unwrap();

        let mut g2 = HashSet::<[SimpleTerm; 3]>::new();
        p2.parse_str(qs).add_to_graph(&mut g2).unwrap();

        assert!(isomorphic_graphs(&g1, &g2).unwrap());
    }

    #[test]
    pub fn correctly_parses_turtle() {
        Lazy::force(&TRACING);
        check_graph_parse_isomorphism(
            &TurtleParser {
                base: Some(BASE_IRI1.clone()),
            },
            &DYNSYN_TRIPLE_PARSER_FACTORY.new_parser(TP_TURTLE, Some(BASE_IRI1.clone())),
            GRAPH_STR_TURTLE,
        );
    }

    #[test]
    pub fn correctly_parses_ntriples() {
        Lazy::force(&TRACING);
        check_graph_parse_isomorphism(
            &NTriplesParser {},
            &DYNSYN_TRIPLE_PARSER_FACTORY.new_parser(TP_N_TRIPLES, Some(BASE_IRI1.clone())),
            GRAPH_STR_N_TRIPLES,
        );
    }

    #[cfg(feature = "rdf_xml")]
    #[test]
    pub fn correctly_parses_rdf_xml() {
        Lazy::force(&TRACING);
        check_graph_parse_isomorphism(
            &RdfXmlParser {
                base: Some(BASE_IRI1.clone()),
            },
            &DYNSYN_TRIPLE_PARSER_FACTORY.new_parser(TP_RDF_XML, Some(BASE_IRI1.clone())),
            GRAPH_STR_RDF_XML,
        );
    }
}

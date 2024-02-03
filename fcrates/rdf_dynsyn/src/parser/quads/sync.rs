use std::{io::BufRead, sync::Arc};

use sophia_api::prelude::{Iri, QuadParser};
use sophia_turtle::parser::{nq::NQuadsParser, trig::TriGParser};

use super::{factory::DynSynQuadParserFactory, source::InnerQuadSource, DynSynQuadSource};
use crate::{
    parser::config::DynSynParserConfig,
    syntax::{self, invariant::quads_parsable::QuadsParsableSyntax},
};

#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdParser;

#[cfg(feature = "jsonld")]
use crate::parser::config::jsonld::DynDocumentLoaderFactory;

/// A sum-type that wraps around different quad-parsers from sophia.
enum InnerQuadParser {
    NQuads(NQuadsParser),
    TriG(TriGParser),
    #[cfg(feature = "jsonld")]
    JsonLd(JsonLdParser<DynDocumentLoaderFactory>),
}

impl std::fmt::Debug for InnerQuadParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NQuads(arg0) => f.debug_tuple("NQuads").field(arg0).finish(),
            Self::TriG(arg0) => f.debug_tuple("TriG").field(arg0).finish(),
            #[cfg(feature = "jsonld")]
            Self::JsonLd(_) => f.debug_tuple("JsonLd").finish(),
        }
    }
}

impl From<NQuadsParser> for InnerQuadParser {
    fn from(p: NQuadsParser) -> Self {
        Self::NQuads(p)
    }
}

impl From<TriGParser> for InnerQuadParser {
    fn from(p: TriGParser) -> Self {
        Self::TriG(p)
    }
}

#[cfg(feature = "jsonld")]
impl From<JsonLdParser<DynDocumentLoaderFactory>> for InnerQuadParser {
    fn from(p: JsonLdParser<DynDocumentLoaderFactory>) -> Self {
        Self::JsonLd(p)
    }
}

impl InnerQuadParser {
    /// Create a sum-parser for given syntax.
    pub fn new(
        syntax_: QuadsParsableSyntax,
        config: &DynSynParserConfig,
        base_iri: Option<Iri<String>>,
    ) -> Self {
        match syntax_.into_subject() {
            syntax::N_QUADS => NQuadsParser {}.into(),
            syntax::TRIG => TriGParser {
                base: base_iri.clone(),
            }
            .into(),
            #[cfg(feature = "jsonld")]
            syntax::JSON_LD => {
                let mut options = config.resolved_jsonld_options();

                options = if let Some(base) = base_iri {
                    options.with_base(Iri::new_unchecked(Arc::from(base.as_str())))
                } else {
                    options.with_no_base()
                };

                JsonLdParser::new_with_options(options).into()
            }
            // All quad parsable syntaxes are addressed.
            _ => unreachable!(),
        }
    }
}

/// This parser implements [`sophia_api::parser::QuadParser`]
/// trait, and can be instantiated at runtime against any of
/// supported syntaxes using [`DynSynQuadParserFactory`]
/// factory.
///
/// It can currently parse quads from documents in any of
/// concrete_syntaxes: [`n-quads`](crate::syntax::N_QUADS),
/// [`trig`](crate::syntax::TRIG). For docs in any of these
/// syntaxes, this parser will stream quads through
/// [`DynSynQuadSource`] instance.
///
#[derive(Debug)]
pub struct DynSynQuadParser(InnerQuadParser);

impl DynSynQuadParser {
    /// Create a new parser with given params.
    #[inline]
    pub(crate) fn new(
        syntax_: QuadsParsableSyntax,
        config: &DynSynParserConfig,
        base_iri: Option<Iri<String>>,
    ) -> Self {
        Self(InnerQuadParser::new(syntax_, config, base_iri))
    }
}

impl<R> QuadParser<R> for DynSynQuadParser
where
    R: BufRead,
{
    type Source = DynSynQuadSource<R>;

    fn parse(&self, data: R) -> Self::Source {
        match &self.0 {
            InnerQuadParser::NQuads(p) => DynSynQuadSource(p.parse(data).into()),
            InnerQuadParser::TriG(p) => DynSynQuadSource(p.parse(data).into()),
            #[cfg(feature = "jsonld")]
            InnerQuadParser::JsonLd(p) => DynSynQuadSource(InnerQuadSource::FJsonLd(p.parse(data))),
        }
    }
}

impl DynSynQuadParserFactory {
    /// Create a new [`DynSynQuadParser`] instance, for given
    /// `syntax_`, `base_iri`.
    #[inline]
    pub fn new_parser(
        &self,
        syntax_: QuadsParsableSyntax,
        base_iri: Option<Iri<String>>,
    ) -> DynSynQuadParser {
        DynSynQuadParser::new(syntax_, &self.config, base_iri)
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
        parser::{IntoParsable, QuadParser},
        quad::Spog,
        source::QuadSource,
        term::SimpleTerm,
    };
    use sophia_isomorphism::isomorphic_datasets;
    use sophia_turtle::parser::{nq::NQuadsParser, trig::TriGParser};

    use super::*;
    use crate::{
        parser::test_data::*,
        syntax::invariant::quads_parsable::{QP_N_QUADS, QP_TRIG},
        tests::TRACING,
    };

    static DYNSYN_QUAD_PARSER_FACTORY: Lazy<DynSynQuadParserFactory> =
        Lazy::new(DynSynQuadParserFactory::default);

    fn check_dataset_parse_isomorphism<'b, B, P1, P2>(p1: &P1, p2: &P2, qs: &'b str)
    where
        P1: QuadParser<B>,
        P2: QuadParser<B>,
        &'b str: IntoParsable<Target = B>,
    {
        let mut d1 = HashSet::<Spog<SimpleTerm>>::new();
        p1.parse_str(qs).add_to_dataset(&mut d1).unwrap();

        let mut d2 = HashSet::<Spog<SimpleTerm>>::new();
        p2.parse_str(qs).add_to_dataset(&mut d2).unwrap();

        assert!(isomorphic_datasets(&d1, &d2).unwrap());
    }

    #[test]
    pub fn correctly_parses_nquads() {
        Lazy::force(&TRACING);
        check_dataset_parse_isomorphism(
            &NQuadsParser {},
            &DYNSYN_QUAD_PARSER_FACTORY.new_parser(QP_N_QUADS, Some(BASE_IRI1.clone())),
            DATASET_STR_N_QUADS,
        );
    }

    #[test]
    pub fn correctly_parses_trig() {
        Lazy::force(&TRACING);
        check_dataset_parse_isomorphism(
            &TriGParser {
                base: Some(BASE_IRI1.clone()),
            },
            &DYNSYN_QUAD_PARSER_FACTORY.new_parser(QP_TRIG, Some(BASE_IRI1.clone())),
            DATASET_STR_TRIG,
        );
    }
}

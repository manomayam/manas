use std::{fmt::Debug, io};

use sophia_api::{
    serializer::{QuadSerializer, Stringifier},
    source::{QuadSource, StreamResult},
};

use sophia_turtle::serializer::{nq::NqSerializer, trig::TrigSerializer};

use super::factory::DynSynQuadSerializerFactory;
use crate::syntax::{self, invariant::quads_serializable::QuadsSerializableSyntax};

#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdSerializer;

#[cfg(feature = "jsonld")]
use crate::parser::config::jsonld::DynDocumentLoader;

/// This is a sum-type that wraps around different
/// quad-serializers from sophia.
pub(crate) enum InnerQuadSerializer<W: io::Write> {
    NQuads(NqSerializer<W>),
    Trig(TrigSerializer<W>),
    #[cfg(feature = "jsonld")]
    JsonLd(JsonLdSerializer<W, DynDocumentLoader>),
}

impl<W: io::Write> Debug for InnerQuadSerializer<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NQuads(_) => f.debug_tuple("NQuads").finish(),
            Self::Trig(_) => f.debug_tuple("Trig").finish(),
            #[cfg(feature = "jsonld")]
            Self::JsonLd(_) => f.debug_tuple("JsonLd").finish(),
        }
    }
}

/// A [`QuadSerializer`], that can be instantiated at run time
/// against any of supported rdf-syntaxes. We can get it's tuned instance from
/// [`DynSynQuadSerializerFactory::new_serializer`] factory method.
///
/// It can currently serialize quad-sources/datasets into
/// documents in any of concrete_syntaxes: [`n-quads`](crate::syntax::invariant::quads_serializable::QS_N_QUADS),
/// [`trig`](crate::syntax::invariant::quads_serializable::QS_TRIG), [`json-ld`](crate::syntax::invariant::quads_serializable::QS_JSON_LD),. Other syntaxes that
/// cannot represent quads are not supported
///
/// For each supported serialization syntax, it also supports
/// corresponding formatting options that sophia supports.
///

pub struct DynSynQuadSerializer<W: io::Write>(InnerQuadSerializer<W>);

impl<W: io::Write> DynSynQuadSerializer<W> {
    #[inline]
    pub(crate) fn new(inner_serializer: InnerQuadSerializer<W>) -> Self {
        Self(inner_serializer)
    }
}

impl<W: io::Write> QuadSerializer for DynSynQuadSerializer<W> {
    type Error = io::Error;

    fn serialize_quads<QS>(&mut self, source: QS) -> StreamResult<&mut Self, QS::Error, Self::Error>
    where
        QS: QuadSource,
        Self: Sized,
    {
        match &mut self.0 {
            InnerQuadSerializer::NQuads(s) => match s.serialize_quads(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            },
            InnerQuadSerializer::Trig(s) => match s.serialize_quads(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e),
            },
            #[cfg(feature = "jsonld")]
            InnerQuadSerializer::JsonLd(s) => match s.serialize_quads(source) {
                Ok(_) => Ok(self),
                Err(e) => Err(e.map_sink(|se| io::Error::new(io::ErrorKind::InvalidInput, se))),
            },
        }
    }
}

impl Stringifier for DynSynQuadSerializer<Vec<u8>> {
    fn as_utf8(&self) -> &[u8] {
        match &self.0 {
            InnerQuadSerializer::NQuads(s) => s.as_utf8(),
            InnerQuadSerializer::Trig(s) => s.as_utf8(),
            #[cfg(feature = "jsonld")]
            InnerQuadSerializer::JsonLd(s) => s.as_utf8(),
        }
    }
}

impl DynSynQuadSerializerFactory {
    /// Create new [`DynSynQuadSerializer`] instance, for given `syntax_`, `write`,
    pub fn new_serializer<W: io::Write>(
        &self,
        syntax_: QuadsSerializableSyntax,
        write: W,
    ) -> DynSynQuadSerializer<W> {
        match syntax_.into_subject() {
            syntax::N_QUADS => DynSynQuadSerializer::new(InnerQuadSerializer::NQuads(
                NqSerializer::new_with_config(
                    write,
                    self.config.nquads.clone().unwrap_or_default(),
                ),
            )),
            syntax::TRIG => DynSynQuadSerializer::new(InnerQuadSerializer::Trig(
                TrigSerializer::new_with_config(
                    write,
                    self.config.trig.clone().unwrap_or_default(),
                ),
            )),
            #[cfg(feature = "jsonld")]
            syntax::JSON_LD => DynSynQuadSerializer::new(InnerQuadSerializer::JsonLd(
                JsonLdSerializer::new_with_options(write, self.config.resolved_jsonld_options()),
            )),

            // All quads serializable syntaxes addressed.
            _ => unreachable!(),
        }
    }

    /// Create new [`DynSynQuadSerializer`] instance, that can be stringified after serialization, for given `syntax_`.
    #[inline]
    pub fn new_stringifier(
        &self,
        syntax_: QuadsSerializableSyntax,
    ) -> DynSynQuadSerializer<Vec<u8>> {
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
        dataset::Dataset,
        parser::QuadParser,
        quad::Spog,
        serializer::{QuadSerializer, Stringifier},
        term::SimpleTerm,
    };
    use sophia_turtle::serializer::{nq::NqConfig, trig::TrigConfig};

    use super::*;
    use crate::{
        parser::quads::DynSynQuadParserFactory,
        serializer::{
            config::DynSynSerializerConfig,
            test_data::{TESTS_NQUADS, TESTS_TRIG},
        },
        syntax::invariant::quads_serializable::*,
        tests::TRACING,
    };

    static SERIALIZER_FACTORY: Lazy<DynSynQuadSerializerFactory> =
        Lazy::new(|| DynSynQuadSerializerFactory::new(Default::default()));

    static SERIALIZER_FACTORY_WITH_PRETTY_CONFIG: Lazy<DynSynQuadSerializerFactory> =
        Lazy::new(|| {
            let config = DynSynSerializerConfig::default()
                .with_trig_config(TrigConfig::new().with_pretty(true))
                .with_nquads_config(NqConfig::default());

            DynSynQuadSerializerFactory::new(config)
        });

    /// As DynSyn parsers can be non-cyclically tested, we can use them here.
    static QUAD_PARSER_FACTORY: Lazy<DynSynQuadParserFactory> =
        Lazy::new(DynSynQuadParserFactory::default);

    #[rstest]
    #[case(QS_TRIG, TESTS_TRIG[0], false)]
    #[case(QS_TRIG, TESTS_TRIG[1], false)]
    #[case(QS_TRIG, TESTS_TRIG[2], false)]
    #[case(QS_TRIG, TESTS_TRIG[3], false)]
    #[case(QS_TRIG, TESTS_TRIG[4], false)]
    #[case(QS_TRIG, TESTS_TRIG[5], false)]
    #[case(QS_TRIG, TESTS_TRIG[0], true)]
    #[case(QS_TRIG, TESTS_TRIG[1], true)]
    #[case(QS_TRIG, TESTS_TRIG[2], true)]
    #[case(QS_TRIG, TESTS_TRIG[3], true)]
    #[case(QS_TRIG, TESTS_TRIG[4], true)]
    #[case(QS_TRIG, TESTS_TRIG[5], true)]
    #[case(QS_N_QUADS, TESTS_NQUADS[0], false)]
    #[case(QS_N_QUADS, TESTS_NQUADS[0], true)]
    pub fn correctly_roundtrips_for_syntax(
        #[case] syntax_: QuadsSerializableSyntax,
        #[case] rdf_doc: &str,
        #[case] pretty: bool,
    ) {
        Lazy::force(&TRACING);
        let parser =
            QUAD_PARSER_FACTORY.new_parser(Proven::try_new(syntax_.into_subject()).unwrap(), None);
        let d1: HashSet<Spog<SimpleTerm>> = parser.parse_str(rdf_doc).collect_quads().unwrap();

        let factory = if pretty {
            &SERIALIZER_FACTORY_WITH_PRETTY_CONFIG
        } else {
            &SERIALIZER_FACTORY
        };

        let out = factory
            .new_stringifier(syntax_)
            .serialize_quads(d1.quads())
            .unwrap()
            .to_string();
        let d2: HashSet<Spog<SimpleTerm>> = parser.parse_str(&out).collect_quads().unwrap();
        assert!(sophia_isomorphism::isomorphic_datasets(&d1, &d2).unwrap());
    }
}

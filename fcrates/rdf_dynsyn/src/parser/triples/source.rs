use std::{error::Error, io::BufRead};

use rio_api::parser::TriplesParser as RioTriplesParser;
use rio_turtle::{NTriplesParser as RioNTriplesParser, TurtleParser as RioTurtleParser};
#[cfg(feature = "rdf_xml")]
use rio_xml::RdfXmlParser as RioRdfXmlParser;
use sophia_api::source::{StreamResult, TripleSource};
use sophia_rio::parser::StrictRioSource;

use crate::{model::DynSynTriple, parser::error::DynSynParseError};

/// This is a sum-type that wraps around different triple-streaming-sources.
/// (currently those, which implements [`TripleSource`](sophia_api::source::TripleSource)), that are produced by different sophia triple parsers.
#[allow(clippy::large_enum_variant)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum InnerTripleSource<R: BufRead> {
    FNTriples(StrictRioSource<RioNTriplesParser<R>>),
    FTurtle(StrictRioSource<RioTurtleParser<R>>),
    #[cfg(feature = "rdf_xml")]
    FRdfXml(StrictRioSource<RioRdfXmlParser<R>>),
}

impl<R: BufRead> From<StrictRioSource<RioNTriplesParser<R>>> for InnerTripleSource<R> {
    #[inline]
    fn from(qs: StrictRioSource<RioNTriplesParser<R>>) -> Self {
        Self::FNTriples(qs)
    }
}

impl<R: BufRead> From<StrictRioSource<RioTurtleParser<R>>> for InnerTripleSource<R> {
    #[inline]
    fn from(qs: StrictRioSource<RioTurtleParser<R>>) -> Self {
        Self::FTurtle(qs)
    }
}

#[cfg(feature = "rdf_xml")]
impl<R: BufRead> From<StrictRioSource<RioRdfXmlParser<R>>> for InnerTripleSource<R> {
    #[inline]
    fn from(qs: StrictRioSource<RioRdfXmlParser<R>>) -> Self {
        Self::FRdfXml(qs)
    }
}

/// A [`TripleSource`] type, returned by dynsyn triple parsers..
pub struct DynSynTripleSource<R: BufRead>(pub(crate) InnerTripleSource<R>);

impl<R: BufRead> DynSynTripleSource<R> {
    /// Call `f` for at least one adapted-triple (if any) that is
    /// adapted from underlying rio triple source.
    ///
    fn try_for_some_adapted_rio_triple<Parser, SinkErr, F>(
        // underlying triple source
        ts: &mut StrictRioSource<Parser>,
        mut f: F,
    ) -> StreamResult<bool, DynSynParseError, SinkErr>
    where
        Parser: RioTriplesParser,
        Parser::Error: Error + Send + Sync + 'static,
        SinkErr: Error,

        F: FnMut(DynSynTriple<'_>) -> Result<(), SinkErr>,
    {
        TripleSource::try_for_some_triple(ts, |t| f(DynSynTriple(t.into())))
            .map_err(|e| e.map_source(|se| DynSynParseError(Box::new(se))))
    }
}

impl<R> TripleSource for DynSynTripleSource<R>
where
    R: BufRead,
{
    type Error = DynSynParseError;

    type Triple<'x> = DynSynTriple<'x>;

    fn try_for_some_triple<E, F>(&mut self, f: F) -> StreamResult<bool, Self::Error, E>
    where
        E: Error,
        F: FnMut(Self::Triple<'_>) -> Result<(), E>,
    {
        match &mut self.0 {
            InnerTripleSource::FNTriples(ts) => Self::try_for_some_adapted_rio_triple(ts, f),

            InnerTripleSource::FTurtle(ts) => Self::try_for_some_adapted_rio_triple(ts, f),

            #[cfg(feature = "rdf_xml")]
            InnerTripleSource::FRdfXml(ts) => Self::try_for_some_adapted_rio_triple(ts, f),
        }
    }
}

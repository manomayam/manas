use std::{error::Error, io::BufRead};

use rio_api::parser::QuadsParser as RioQuadsParser;
use rio_turtle::{NQuadsParser as RioNQuadsParser, TriGParser as RioTriGParser};
use sophia_api::source::{QuadSource, StreamResult};
use sophia_rio::parser::StrictRioSource;

#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdQuadSource;

use crate::{model::DynSynQuad, parser::error::DynSynParseError};

/// This is a sum-type that wraps around different quad-streaming-sources.
/// (currently those, which implements [`QuadSource`](sophia_api::source::QuadSource)), that are produced by different sophia quad parsers.
pub(crate) enum InnerQuadSource<R: BufRead> {
    FNQuads(StrictRioSource<RioNQuadsParser<R>>),
    FTriG(StrictRioSource<RioTriGParser<R>>),
    #[cfg(feature = "jsonld")]
    FJsonLd(JsonLdQuadSource),
}

impl<R: BufRead> From<StrictRioSource<RioNQuadsParser<R>>> for InnerQuadSource<R> {
    fn from(qs: StrictRioSource<RioNQuadsParser<R>>) -> Self {
        Self::FNQuads(qs)
    }
}

impl<R: BufRead> From<StrictRioSource<RioTriGParser<R>>> for InnerQuadSource<R> {
    fn from(qs: StrictRioSource<RioTriGParser<R>>) -> Self {
        Self::FTriG(qs)
    }
}

/// A [`QuadSource`] type, returned by dynsyn quad parsers..
pub struct DynSynQuadSource<R: BufRead>(pub(crate) InnerQuadSource<R>);

impl<R: BufRead> DynSynQuadSource<R> {
    /// Call `f` for at least one adapted-quad (if any) that is
    /// adapted from underlying rio quad source.
    ///
    fn try_for_some_adapted_rio_quad<Parser, SinkErr, F>(
        // underlying quad source
        qs: &mut StrictRioSource<Parser>,
        mut f: F,
    ) -> StreamResult<bool, DynSynParseError, SinkErr>
    where
        Parser: RioQuadsParser,
        Parser::Error: Error + Send + Sync + 'static,
        SinkErr: Error,

        F: FnMut(DynSynQuad<'_>) -> Result<(), SinkErr>,
    {
        QuadSource::try_for_some_quad(qs, |q| f(DynSynQuad(q.into())))
            .map_err(|e| e.map_source(|se| DynSynParseError(Box::new(se))))
    }

    #[cfg(feature = "jsonld")]
    fn try_for_some_adapted_jsonld_quad<SinkErr, F>(
        // underlying quad source
        qs: &mut JsonLdQuadSource,
        mut f: F,
    ) -> StreamResult<bool, DynSynParseError, SinkErr>
    where
        SinkErr: Error,
        F: FnMut(DynSynQuad<'_>) -> Result<(), SinkErr>,
    {
        use tracing::error;

        QuadSource::try_for_some_quad(qs, |q| f(DynSynQuad(q.into()))).map_err(|e| {
            e.map_source(|se| {
                error!("Error in parsing jsonld quad. {:?}", se);
                DynSynParseError(Box::new(se))
            })
        })
    }
}

impl<R> QuadSource for DynSynQuadSource<R>
where
    R: BufRead,
{
    type Error = DynSynParseError;

    type Quad<'x> = DynSynQuad<'x>;

    fn try_for_some_quad<E, F>(&mut self, f: F) -> StreamResult<bool, Self::Error, E>
    where
        E: Error,
        F: FnMut(Self::Quad<'_>) -> Result<(), E>,
    {
        match &mut self.0 {
            InnerQuadSource::FNQuads(qs) => Self::try_for_some_adapted_rio_quad(qs, f),

            InnerQuadSource::FTriG(qs) => Self::try_for_some_adapted_rio_quad(qs, f),

            #[cfg(feature = "jsonld")]
            InnerQuadSource::FJsonLd(qs) => Self::try_for_some_adapted_jsonld_quad(qs, f),
        }
    }
}

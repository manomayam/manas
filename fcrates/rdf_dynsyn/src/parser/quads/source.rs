use std::{error::Error, io::BufRead};

use rio_api::parser::QuadsParser as RioQuadsParser;
use rio_turtle::{NQuadsParser as RioNQuadsParser, TriGParser as RioTriGParser};
use sophia_api::source::{QuadSource, Source, StreamResult};
use sophia_rio::parser::StrictRioQuadSource;

#[cfg(feature = "jsonld")]
use sophia_jsonld::JsonLdQuadSource;

use crate::{model::DynSynQuad, parser::error::DynSynParseError};

/// This is a sum-type that wraps around different quad-streaming-sources.
/// (currently those, which implements [`QuadSource`](sophia_api::source::QuadSource)), that are produced by different sophia quad parsers.
pub(crate) enum InnerQuadSource<R: BufRead> {
    FNQuads(StrictRioQuadSource<RioNQuadsParser<R>>),
    FTriG(StrictRioQuadSource<RioTriGParser<R>>),
    #[cfg(feature = "jsonld")]
    FJsonLd(JsonLdQuadSource),
}

impl<R: BufRead> From<StrictRioQuadSource<RioNQuadsParser<R>>> for InnerQuadSource<R> {
    fn from(qs: StrictRioQuadSource<RioNQuadsParser<R>>) -> Self {
        Self::FNQuads(qs)
    }
}

impl<R: BufRead> From<StrictRioQuadSource<RioTriGParser<R>>> for InnerQuadSource<R> {
    fn from(qs: StrictRioQuadSource<RioTriGParser<R>>) -> Self {
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
        qs: &mut StrictRioQuadSource<Parser>,
        mut f: F,
    ) -> StreamResult<bool, DynSynParseError, SinkErr>
    where
        Parser: RioQuadsParser,
        Parser::Error: Error + Send + Sync + 'static,
        SinkErr: Error + Send + Sync + 'static,
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
        SinkErr: Error + Send + Sync + 'static,
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

impl<R> Source for DynSynQuadSource<R>
where
    R: BufRead,
{
    type Item<'x> = DynSynQuad<'x>;

    type Error = DynSynParseError;

    fn try_for_some_item<E, F>(&mut self, f: F) -> StreamResult<bool, Self::Error, E>
    where
        E: Error + Send + Sync + 'static,
        F: FnMut(Self::Item<'_>) -> Result<(), E>,
    {
        match &mut self.0 {
            InnerQuadSource::FNQuads(qs) => Self::try_for_some_adapted_rio_quad(qs, f),

            InnerQuadSource::FTriG(qs) => Self::try_for_some_adapted_rio_quad(qs, f),

            #[cfg(feature = "jsonld")]
            InnerQuadSource::FJsonLd(qs) => Self::try_for_some_adapted_jsonld_quad(qs, f),
        }
    }
}

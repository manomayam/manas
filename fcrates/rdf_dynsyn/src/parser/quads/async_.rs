use std::io::BufReader;

use async_compat::CompatExt;
use bytes::Bytes;
use futures::{stream::BoxStream, AsyncRead, TryStream};
use sophia_api::{
    parser::QuadParser,
    prelude::Iri,
    quad::{Quad, Spog},
    source::QuadSource,
    term::{FromTerm, Term},
};
use tokio::{sync::mpsc, task::spawn_blocking};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::SyncIoBridge;
use tracing::error;

use super::{factory::DynSynQuadParserFactory, sync::DynSynQuadParser};
use crate::{
    parser::error::DynSynParseError, syntax::invariant::quads_parsable::QuadsParsableSyntax,
    util::stream::bytes_stream_to_async_reader,
};

/// Async quad parser.
pub struct DynSynAsyncQuadParser(pub(crate) DynSynQuadParser);

impl DynSynAsyncQuadParser {
    /// Parse quads from given async reader.
    pub async fn parse<T, R>(
        &self,
        data: R,
    ) -> BoxStream<'static, Result<Spog<T>, DynSynParseError>>
    where
        R: AsyncRead + Send + 'static + Unpin,
        T: Term + Send + 'static + FromTerm,
    {
        // Get quad source.
        let mut quad_source = self
            .0
            .parse(BufReader::new(SyncIoBridge::new(data.compat())));

        let (quads_tx, quads_rx) = mpsc::channel::<Result<Spog<T>, DynSynParseError>>(32);

        spawn_blocking(move || {
            let mut receiver_closed = false;

            while !receiver_closed {
                let r = quad_source.for_some_quad(&mut |q| {
                    if quads_tx
                        .blocking_send(Ok((
                            [q.s().into_term(), q.p().into_term(), q.o().into_term()],
                            q.g().map(|gn| gn.into_term()),
                        )))
                        .is_err()
                    {
                        // Error in sending quad.
                        // Implies receiver is closed for whatever reason.
                        receiver_closed = true;
                    }
                });

                match r {
                    Ok(has_more) => {
                        // Break loop, if no more quads.
                        if !has_more {
                            break;
                        }
                    }
                    Err(parse_error) => {
                        error!("Error in parsing quads. Error:\n {}", parse_error);
                        // Send error, and stop parsing.
                        let _ = quads_tx.blocking_send(Err(parse_error));
                        break;
                    }
                }
            }
        });

        Box::pin(ReceiverStream::new(quads_rx))
    }

    /// Parse quads from given bytes stream.
    #[inline]
    pub async fn parse_stream<T, S>(
        &self,
        data: S,
    ) -> BoxStream<'static, Result<Spog<T>, DynSynParseError>>
    where
        T: Term + FromTerm + Send + 'static,
        S: TryStream<Ok = Bytes> + Send + 'static + Unpin,
        S::Error: 'static + Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        self.parse(bytes_stream_to_async_reader(data)).await
    }
}

impl DynSynQuadParserFactory {
    /// Create a new [`DynSynAsyncQuadParser`] instance, for
    /// given `syntax_`, `base_iri`.
    #[inline]
    pub fn new_async_parser(
        &self,
        syntax_: QuadsParsableSyntax,
        base_iri: Option<Iri<String>>,
    ) -> DynSynAsyncQuadParser {
        DynSynAsyncQuadParser(DynSynQuadParser::new(syntax_, &self.config, base_iri))
    }
}

use std::io::BufReader;

use async_compat::CompatExt;
use bytes::Bytes;
use futures::{stream::BoxStream, AsyncRead, TryStream};
use sophia_api::{
    parser::TripleParser,
    prelude::Iri,
    source::TripleSource,
    term::{FromTerm, Term},
    triple::Triple,
};
use tokio::{sync::mpsc, task::spawn_blocking};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::SyncIoBridge;
use tracing::error;

use super::{factory::DynSynTripleParserFactory, sync::DynSynTripleParser};
use crate::{
    parser::error::DynSynParseError, syntax::invariant::triples_parsable::TriplesParsableSyntax,
    util::stream::bytes_stream_to_async_reader,
};

/// Async triple parser.
pub struct DynSynAsyncTripleParser(pub(crate) DynSynTripleParser);

impl DynSynAsyncTripleParser {
    /// Parse triples from given async reader.
    pub async fn parse<T, R>(&self, data: R) -> BoxStream<'static, Result<[T; 3], DynSynParseError>>
    where
        R: AsyncRead + Send + 'static + Unpin,
        T: Term + Send + 'static + FromTerm,
    {
        // Get triple source.
        let mut triple_source = self
            .0
            .parse(BufReader::new(SyncIoBridge::new(data.compat())));

        let (triples_tx, triples_rx) = mpsc::channel::<Result<[T; 3], DynSynParseError>>(32);

        spawn_blocking(move || {
            let mut receiver_closed = false;

            while !receiver_closed {
                let r = triple_source.for_some_triple(&mut |t| {
                    if triples_tx
                        .blocking_send(Ok([
                            t.s().into_term(),
                            t.p().into_term(),
                            t.o().into_term(),
                        ]))
                        .is_err()
                    {
                        // Error in sending triple.
                        // Implies receiver is closed for whatever reason.
                        receiver_closed = true;
                    }
                });

                match r {
                    Ok(has_more) => {
                        // Break loop, if no more triples.
                        if !has_more {
                            break;
                        }
                    }
                    Err(parse_error) => {
                        error!("Error in parsing triples. Error:\n {}", parse_error);
                        // Send error, and stop parsing.
                        let _ = triples_tx.blocking_send(Err(parse_error));
                        break;
                    }
                }
            }
        });

        Box::pin(ReceiverStream::new(triples_rx))
    }

    /// Parse triples from given bytes stream.
    #[inline]
    pub async fn parse_stream<T, S>(
        &self,
        data: S,
    ) -> BoxStream<'static, Result<[T; 3], DynSynParseError>>
    where
        T: Term + FromTerm + Send + 'static,
        S: TryStream<Ok = Bytes> + Send + 'static + Unpin,
        S::Error: 'static + Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        self.parse(bytes_stream_to_async_reader(data)).await
    }
}

impl DynSynTripleParserFactory {
    /// Create a new [`DynSynAsyncTripleParser`] instance, for
    /// given `syntax_`, `base_iri`.
    #[inline]
    pub fn new_async_parser(
        &self,
        syntax_: TriplesParsableSyntax,
        base_iri: Option<Iri<String>>,
    ) -> DynSynAsyncTripleParser {
        DynSynAsyncTripleParser(DynSynTripleParser::new(syntax_, base_iri))
    }
}

use std::io;

use bytes::Bytes;
use futures::{AsyncRead, Stream, StreamExt, TryStream, TryStreamExt};
use tokio::runtime::Handle;

pub fn bytes_stream_to_async_reader<S>(data: S) -> impl AsyncRead
where
    S: TryStream<Ok = Bytes> + Send + 'static + Unpin,
    S::Error: 'static + Into<Box<dyn std::error::Error + Send + Sync>>,
{
    data.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .into_async_read()
}

/// A blocking iterator backed by non-blocking stream.
pub struct BlockingStreamIterator<S: Stream + Unpin> {
    /// Inner stream.
    stream: S,

    /// Tokio runtime.
    rt: Handle,
}

impl<S: Stream + Unpin> BlockingStreamIterator<S> {
    /// Get new [`BlockingStreamIterator`] backed by given stream.
    ///
    /// # Panics
    ///
    /// This will panic if called outside the context of a Tokio runtime.
    // TODO must make run-time agnostic / interoperable.
    #[inline]
    #[track_caller]
    pub fn new(stream: S) -> io::Result<Self> {
        Self::new_with_handle(stream, Handle::current())
    }

    /// Get new [`BlockingStreamIterator`] backed by given stream, and given runtime handle.
    #[inline]
    pub fn new_with_handle(stream: S, rt: Handle) -> io::Result<Self> {
        Ok(Self { stream, rt })
    }
}

impl<S: Stream + Unpin> Iterator for BlockingStreamIterator<S> {
    type Item = S::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.rt.block_on(self.stream.next())
    }
}

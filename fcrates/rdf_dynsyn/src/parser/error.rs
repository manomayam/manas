//! This modules provides error type for errors in parsing.
//!

use sophia_api::source::StreamResult;

use crate::util::BoxError;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
/// An error that abstracts over other syntax parsing errors.
pub struct DynSynParseError(pub BoxError);

/// Alias for errors returned by dynsyn streams.
pub type DynSynStreamResult<T, SinkErr> = StreamResult<T, DynSynParseError, SinkErr>;

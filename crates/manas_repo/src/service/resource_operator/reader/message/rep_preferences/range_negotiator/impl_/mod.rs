//! I define few implementations of [`RangeNegotiator`](super::RangeNegotiator).

/// I define an implementation of [`RangeNegotiator`] that
/// negotiates for complete data.
mod complete;

/// I define an implementation of [`RangeNegotiator`] that
/// negotiates based on conditional range headers.
mod conditional;

/// I define an implementation of [`RangeNegotiator`] that
/// negotiates based on result of derived-content-negotiation.
mod dconneg_layered;

pub use complete::CompleteRangeNegotiator;
pub use conditional::ConditionalRangeNegotiator;
pub use dconneg_layered::*;

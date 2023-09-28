//! This modules provides utils for parsing quads.
//!

mod factory;
mod source;
mod sync;

#[cfg(feature = "async")]
mod async_;

#[cfg(feature = "async")]
pub use async_::DynSynAsyncQuadParser;
pub use factory::DynSynQuadParserFactory;
pub use source::DynSynQuadSource;
pub use sync::DynSynQuadParser;

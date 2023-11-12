//! This modules provides utils for triples serialization.
//!

mod factory;
mod sync;

#[cfg(feature = "async")]
mod async_;

#[cfg(feature = "async")]
pub use async_::DynSynAsyncTripleSerializer;
pub use factory::DynSynTripleSerializerFactory;
pub use sync::DynSynTripleSerializer;

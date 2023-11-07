#[cfg(feature = "async")]
pub mod stream;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

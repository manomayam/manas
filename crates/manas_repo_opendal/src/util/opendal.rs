//! Module for utilities over opendal types.
//!

use opendal::ErrorKind;

/// An extension trait for `opendal::Result`.
pub trait OpendalResultExt<T>: seal::Sealed {
    /// Handle object not found case by returning an option.
    /// Pass on any other error.
    fn found(self) -> opendal::Result<Option<T>>;
}

mod seal {
    pub trait Sealed {}

    impl<T> Sealed for opendal::Result<T> {}
}

impl<T> OpendalResultExt<T> for opendal::Result<T> {
    fn found(self) -> opendal::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}

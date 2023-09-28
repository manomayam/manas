//! Extends `Last-Modified` typed header

use std::time::SystemTime;

use chrono::{DateTime, Utc};
use headers::LastModified;

/// A sealed trait, that adds few convenient methods to [`LastModified`].
pub trait LastModifiedExt: seal::Sealed {
    ///  Convert to offset date time.
    fn to_date_time(&self) -> DateTime<Utc>;

    /// Convert from offset date time.
    fn from_date_time(val: DateTime<Utc>) -> Self;

    /// Derive an etag from last-modified.
    // TODO may be redundant.
    fn derive_etag(&self) -> String;
}

impl LastModifiedExt for LastModified {
    #[inline]
    fn to_date_time(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from(SystemTime::from(*self))
    }

    #[inline]
    fn from_date_time(val: DateTime<Utc>) -> Self {
        Self::from(SystemTime::from(val))
    }

    #[inline]
    fn derive_etag(&self) -> String {
        // TODO: Must be weak etag instead?.
        format!("\"{}\"", self.to_date_time().timestamp())
    }
}

mod seal {
    use headers::LastModified;

    pub trait Sealed {}

    impl Sealed for LastModified {}
}

//! I define types for representing representation-metadata.
//!

pub mod derived_etag;

use std::ops::{Deref, DerefMut};

use headers::{ContentLength, ContentRange, ETag, LastModified};
use http_uri::HttpUri;
use typed_record::{ClonableTypedRecord, TypedRecord, TypedRecordKey};

use self::derived_etag::DerivedETag;
use crate::header::common::media_type::{MediaType, APPLICATION_OCTET_STREAM};

/// Struct for representing metadata of a representation.
/// Following keys are defined by default:
///
/// - [`KContentType`]: Content-Type of the representation.
/// - [`KCompleteContentLength`]: `Content-Length` of the complete representation.
/// - [`KContentRange`]: `Content-Range of the representation.
/// - [`KLastModified`]: `Last-Modified` time of the representation.
#[derive(Default, Clone)]
pub struct RepresentationMetadata(ClonableTypedRecord);

impl std::fmt::Debug for RepresentationMetadata {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RepresentationMetadata").finish()
    }
}

impl Deref for RepresentationMetadata {
    type Target = ClonableTypedRecord;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RepresentationMetadata {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RepresentationMetadata {
    #[inline]
    /// Get an empty new [`RepresentationMetadata`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get `Content-Type` of the representation.
    #[inline]
    pub fn content_type(&self) -> &MediaType {
        self.get_rv::<KContentType>()
            .unwrap_or(&APPLICATION_OCTET_STREAM)
    }

    /// Get [`RepresentationMetadata`] with given kv pair inserted.
    #[inline]
    pub fn with<K: TypedRecordKey>(mut self, v: K::Value) -> Self
    where
        Self: Sized,
    {
        self.insert_rec_item::<K>(v);
        self
    }

    /// Get [`RepresentationMetadata`] with given optional kv pair inserted.
    #[inline]
    pub fn with_opt<K: TypedRecordKey>(mut self, v: Option<K::Value>) -> Self
    where
        Self: Sized,
    {
        if let Some(v) = v {
            self.insert_rec_item::<K>(v);
        } else {
            self.remove_rec_item::<K>();
        }
        self
    }
}

#[cfg(feature = "impl-representation")]
mod rdf_ext {
    use std::ops::Deref;

    use rdf_dynsyn::{correspondence::Correspondent, syntax::RdfSyntax};

    use super::RepresentationMetadata;

    impl RepresentationMetadata {
        /// Get the rdf syntax corresponding to content-type.
        #[inline]
        pub fn rdf_syntax<S>(&self) -> Option<Correspondent<S>>
        where
            S: Deref<Target = RdfSyntax>,
            Correspondent<S>: for<'a> TryFrom<&'a mime::Mime>,
        {
            Correspondent::try_from(self.content_type().deref()).ok()
        }
    }
}

/// [`TypedRecordKey`] for representation content-type of the representation.
#[derive(Debug, Clone)]
pub struct KContentType;

impl TypedRecordKey for KContentType {
    type Value = MediaType;
}

/// [`TypedRecordKey`] for Content-Length of the complete representation.
#[derive(Debug, Clone)]
pub struct KCompleteContentLength;

impl TypedRecordKey for KCompleteContentLength {
    type Value = ContentLength;
}

/// [`TypedRecordKey`] for Content-Range of the representation.
#[derive(Debug, Clone)]
pub struct KContentRange;

impl TypedRecordKey for KContentRange {
    type Value = ContentRange;
}

/// [`TypedRecordKey`] for `Last-Modified`  of the representation.
#[derive(Debug, Clone)]
pub struct KLastModified;

impl TypedRecordKey for KLastModified {
    type Value = LastModified;
}

/// [`TypedRecordKey`] for `ETag` of the representation.
#[derive(Debug, Clone)]
pub struct KETag;

impl TypedRecordKey for KETag {
    type Value = ETag;
}

/// [`TypedRecordKey`] for `DerivedETag` of the representation.
#[derive(Debug, Clone)]
pub struct KDerivedETag;

impl TypedRecordKey for KDerivedETag {
    type Value = DerivedETag;
}

/// [`TypedRecordKey`] for md5 of the representation data.
#[derive(Debug, Clone)]
pub struct KMd5;

impl TypedRecordKey for KMd5 {
    type Value = String;
}

/// [`TypedRecordKey`] for base uri of the representation data.
#[derive(Debug, Clone)]
pub struct KBaseUri;

impl TypedRecordKey for KBaseUri {
    type Value = HttpUri;
}

#[cfg(feature = "test-utils")]
// #[cfg(test)]
/// Module for [`RepresentationMetadata`] related mock helpers.
pub mod mock {
    use std::ops::RangeBounds;

    use chrono::{DateTime, Utc};
    use claims::assert_ok;
    use headers::ContentRange;
    use typed_record::TypedRecord;

    use super::{KContentRange, KContentType, KDerivedETag, KLastModified, RepresentationMetadata};
    use crate::header::last_modified::LastModifiedExt;

    ///extension trait for easily creating mock `RepresentationMetadata` objects.
    pub trait RepresentationMetadataMockExt: seal::Sealed {
        /// Get rep metadata set with given content-type.
        /// `content_type_str` must be valid media-type string.
        /// Or else method will panic.
        fn with_content_type(self, content_type_str: &str) -> Self;

        /// Get rep metadata set with given last-modified.
        /// `last_modified_rfc3339` must be valid  rfc3339 datetime string.
        /// Or else method will panic.
        fn with_last_modified(self, last_modified_rfc3339: &str) -> Self;

        /// Get rep metadata set with given derived etag.
        /// `derived_etag_str` must be valid derived-etag string representation.
        /// Or else method will panic.
        fn with_detag(self, derived_etag_str: &str) -> Self;

        /// Get rep metadata set with given content range.
        /// Will panic if content range is invalid.
        fn with_bytes_content_range(
            self,
            range: impl RangeBounds<u64>,
            full_length: Option<u64>,
        ) -> Self;
    }

    impl RepresentationMetadataMockExt for RepresentationMetadata {
        fn with_content_type(mut self, content_type_str: &str) -> Self {
            let content_type = assert_ok!(content_type_str.parse());
            self.insert_rec_item::<KContentType>(content_type);
            self
        }

        fn with_last_modified(mut self, last_modified_rfc3339: &str) -> Self {
            let last_modified = LastModifiedExt::from_date_time(assert_ok!(
                DateTime::parse_from_rfc3339(last_modified_rfc3339)
                    .map(|dt| dt.with_timezone(&Utc))
            ));

            self.insert_rec_item::<KLastModified>(last_modified);
            self
        }

        fn with_detag(mut self, derived_etag_str: &str) -> Self {
            let derived_etag = assert_ok!(derived_etag_str.parse());
            self.insert_rec_item::<KDerivedETag>(derived_etag);
            self
        }

        fn with_bytes_content_range(
            mut self,
            range: impl RangeBounds<u64>,
            complete_length: Option<u64>,
        ) -> Self {
            let content_rage = assert_ok!(ContentRange::bytes(range, complete_length));
            self.insert_rec_item::<KContentRange>(content_rage);
            self
        }
    }

    mod seal {
        use crate::representation::metadata::RepresentationMetadata;

        pub trait Sealed {}

        impl Sealed for RepresentationMetadata {}
    }
}

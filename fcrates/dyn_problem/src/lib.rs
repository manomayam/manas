//! This crate provides a reified dynamic error type, for
//! using in cases, where one may want to be flexible about
//!  kinds of error for an operation.
//!
//! The [`Problem`] type allows to specify kind/type of an
//! error as a uri dynamically.
//! Along with that, the type allows to record a string message,
//! and lets us to attach custom extensions too.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod type_;

use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use http::{Extensions, StatusCode};
use http_api_problem::{ApiError, ApiErrorBuilder};
#[cfg(feature = "ext-typed-record")]
use typed_record::{TypedRecord, TypedRecordKey};

/// [`Problem`] is a reified error type, encoding it's problem type as a uri.
pub struct Problem(ApiError);

/// A type alias for results with [`Problem`] as their error type.
pub type ProbResult<T> = Result<T, Problem>;

#[cfg(feature = "alias-future")]
/// A type alias for boxed futures resolving to results with [`Problem`] error type.
pub type ProbFuture<'a, T> = futures::future::BoxFuture<'a, Result<T, Problem>>;

#[cfg(feature = "alias-future")]
/// A type alias for boxed streams yielding items of results with [`Problem`] error type.
pub type ProbStream<'a, T> = futures::stream::BoxStream<'a, Result<T, Problem>>;

impl Default for Problem {
    #[inline]
    fn default() -> Self {
        Self(ApiError::new(StatusCode::IM_A_TEAPOT))
    }
}

impl Problem {
    /// Get a new [`Problem`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a new [`ProblemBuilder`].
    #[inline]
    pub fn builder() -> ProblemBuilder {
        ProblemBuilder(ApiError::builder(StatusCode::IM_A_TEAPOT))
    }
}

impl Display for Problem {
    // NOTE: adapted from [`Display::fmt`] of `ApiError`, with `status` removed.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match (self.title().as_ref(), self.detail_message()) {
            (Some(title), Some(detail)) => return write!(f, " - {} - {}", title, detail),
            (Some(title), None) => return write!(f, " - {}", title),
            (None, Some(detail)) => return write!(f, " - {}", detail),
            (None, None) => (),
        }

        if let Some(type_url) = self.type_url().as_ref() {
            return write!(f, " of type {}", type_url);
        }

        if let Some(instance) = self.instance().as_ref() {
            return write!(f, " on {}", instance);
        }

        Ok(())
    }
}

impl Debug for Problem {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Error for Problem {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl Deref for Problem {
    type Target = ApiError;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Problem {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ApiError> for Problem {
    #[inline]
    fn from(inner: ApiError) -> Self {
        Self(inner)
    }
}

// impl From<Problem> for ApiError {
//     #[inline]
//     fn from(builder: Problem) -> Self {
//         builder.0
//     }
// }

/// Builder for [`Problem`].
pub struct ProblemBuilder(ApiErrorBuilder);

impl ProblemBuilder {
    /// This is an optional title which can be used to create a valuable output
    /// for consumers.
    #[inline]
    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.0 = self.0.title(title);
        self
    }

    /// A message that describes the error in a human readable form.
    ///
    #[inline]
    pub fn message<M: Display>(mut self, message: M) -> Self {
        self.0 = self.0.message(message);
        self
    }

    /// A URL that points to a detailed description of the error.
    #[inline]
    pub fn type_url<U: Display>(mut self, type_url: U) -> Self {
        self.0 = self.0.type_url(type_url);
        self
    }

    /// Sets the `instance`
    ///
    /// A URI reference that identifies the specific
    /// occurrence of the problem.  It may or may not yield further
    /// information if dereferenced.
    #[inline]
    pub fn instance<T: Display>(mut self, instance: T) -> Self {
        self.0 = self.0.instance(instance);
        self
    }

    /// Adds an extension.
    ///
    /// Existing values will be overwritten
    #[inline]
    pub fn extension<T: Send + Sync + Clone + 'static>(mut self, val: T) -> Self {
        self.0 = self.0.extension(val);
        self
    }

    /// Modify the extension values from within a closure
    #[inline]
    pub fn with_extensions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Extensions) -> Extensions,
    {
        self.0 = self.0.with_extensions(f);
        self
    }

    /// Sets the source error.
    #[inline]
    pub fn source<E: Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.0 = self.0.source(source);
        self
    }

    /// Sets the source error from given type erased error.
    #[inline]
    pub fn source_in_a_box<E: Into<Box<dyn Error + Send + Sync + 'static>>>(
        mut self,
        source: E,
    ) -> Self {
        self.0 = self.0.source_in_a_box(source);
        self
    }

    /// Consumes builder, and return result [`Problem`].
    #[inline]
    pub fn finish(self) -> Problem {
        Problem(self.0.finish())
    }
}

impl Deref for ProblemBuilder {
    type Target = ApiErrorBuilder;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProblemBuilder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ApiErrorBuilder> for ProblemBuilder {
    #[inline]
    fn from(inner: ApiErrorBuilder) -> Self {
        Self(inner)
    }
}

impl From<ProblemBuilder> for ApiErrorBuilder {
    #[inline]
    fn from(builder: ProblemBuilder) -> Self {
        builder.0
    }
}

/// An extension trait for [`ProblemBuilder`], that makes
/// using extensions as typed record easier.
#[cfg(feature = "ext-typed-record")]
pub trait ProblemBuilderExt: Sized {
    /// Extend with given record entry
    fn extend_with<K: TypedRecordKey>(self, v: K::Value) -> Self;

    /// Extend with given optional record entry
    fn extend_with_opt<K: TypedRecordKey>(self, v: Option<K::Value>) -> Self;
}

#[cfg(feature = "ext-typed-record")]
impl ProblemBuilderExt for ApiErrorBuilder {
    #[inline]
    fn extend_with<K: TypedRecordKey>(mut self, v: K::Value) -> Self {
        self.extensions.insert_rec_item::<K>(v);
        self
    }

    #[inline]
    fn extend_with_opt<K: TypedRecordKey>(mut self, v: Option<K::Value>) -> Self {
        if let Some(v) = v {
            self.extensions.insert_rec_item::<K>(v);
        }
        self
    }
}

#[cfg(feature = "ext-typed-record")]
impl ProblemBuilderExt for ProblemBuilder {
    #[inline]
    fn extend_with<K: TypedRecordKey>(mut self, v: K::Value) -> Self {
        self.0 = self.0.extend_with::<K>(v);
        self
    }

    #[inline]
    fn extend_with_opt<K: TypedRecordKey>(mut self, v: Option<K::Value>) -> Self {
        self.0 = self.0.extend_with_opt::<K>(v);
        self
    }
}

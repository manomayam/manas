//! I provide few path helpers for custom opendal service
//! implementations.
//!

use std::{borrow::Cow, ops::Deref};

use gdp_rs::{binclassified::BinaryClassified, Proven};
use opendal::Error;

/// A struct representing normalized path.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NormalPath<'p>(Cow<'p, str>);

impl<'p> Deref for NormalPath<'p> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'p> TryFrom<Cow<'p, str>> for NormalPath<'p> {
    type Error = Error;

    fn try_from(p: Cow<'p, str>) -> Result<Self, Self::Error> {
        if p.split('/').any(|s| s == "." || s == "..") || p.starts_with('/') || p.contains("//") {
            return Err(Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "Invalid path",
            ));
        }

        Ok(Self(p))
    }
}

impl<'p> NormalPath<'p> {
    /// Try to create new normal path from opendal path.
    #[inline]
    pub fn try_new(opendal_path: &'p str) -> Result<Self, Error> {
        Self::try_from(Cow::Borrowed(if opendal_path == "/" {
            ""
        } else {
            opendal_path
        }))
    }

    /// Get ias str.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Check if path is a ns path.
    #[inline]
    pub fn is_ns_path(&self) -> bool {
        self.is_empty() || self.ends_with('/')
    }

    /// Check if path is a file path.
    #[inline]
    pub fn is_file_path(&self) -> bool {
        !self.is_ns_path()
    }

    /// Assert that path is an ns path.
    pub fn assert_is_ns_path(self) -> Result<NsPath<'p>, Error> {
        NsPath::try_new(self)
            .map_err(|_| Error::new(opendal::ErrorKind::NotADirectory, "Path is not a dir path"))
    }

    /// Assert that path is an ns path.
    pub fn assert_is_file_path(self) -> Result<FilePath<'p>, Error> {
        FilePath::try_new(self)
            .map_err(|_| Error::new(opendal::ErrorKind::IsADirectory, "Path is not a file path"))
    }

    /// Get owned value.
    #[inline]
    pub fn into_owned(self) -> NormalPath<'static> {
        NormalPath(Cow::Owned(self.0.into_owned()))
    }
}

pub use predicates::*;

/// An invariant of [`NormalPath`] which can only represent namespace paths.
///
pub type NsPath<'p> = Proven<NormalPath<'p>, IsNsPath>;

/// An invariant of [`NormalPath`] which can only represent file paths.
///
pub type FilePath<'p> = Proven<NormalPath<'p>, IsFilePath>;

/// Binary classified [`NormalPath`].
pub type ClassifiedPath<'p> = BinaryClassified<NormalPath<'p>, PathClassification>;

mod predicates {
    use std::borrow::Cow;

    use gdp_rs::{
        binclassified::{BinaryClassPredicate, BinaryClassification},
        predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
    };

    use super::*;

    /// A predicate over [`NsPath`] that asserts it to be an ns
    /// path.
    #[derive(Debug)]
    pub struct IsNsPath;

    impl<'p> Predicate<NormalPath<'p>> for IsNsPath {
        fn label() -> Cow<'static, str> {
            "IsNsPath".into()
        }
    }

    impl<'p> SyncEvaluablePredicate<NormalPath<'p>> for IsNsPath {
        type EvalError = IsNotNsPath;

        fn evaluate_for(sub: &NormalPath<'p>) -> Result<(), Self::EvalError> {
            if sub.is_ns_path() {
                Ok(())
            } else {
                Err(IsNotNsPath)
            }
        }
    }

    impl<'p> PurePredicate<NormalPath<'p>> for IsNsPath {}

    /// Path is not a namespace path.
    #[derive(Debug, Clone, thiserror::Error)]
    #[error("Path is not a namespace path.")]
    pub struct IsNotNsPath;

    /// A predicate over [`NsPath`] that asserts it to be a file
    /// path.
    #[derive(Debug)]
    pub struct IsFilePath;

    impl<'p> Predicate<NormalPath<'p>> for IsFilePath {
        fn label() -> Cow<'static, str> {
            "IsFilePath".into()
        }
    }

    impl<'p> SyncEvaluablePredicate<NormalPath<'p>> for IsFilePath {
        type EvalError = IsNotFilePath;

        fn evaluate_for(sub: &NormalPath<'p>) -> Result<(), Self::EvalError> {
            if sub.is_file_path() {
                Ok(())
            } else {
                Err(IsNotFilePath)
            }
        }
    }

    impl<'p> PurePredicate<NormalPath<'p>> for IsFilePath {}

    /// Path is not a file path.
    #[derive(Debug, Clone, thiserror::Error)]
    #[error("Path is not a file path.")]
    pub struct IsNotFilePath;

    /// An implementation of [`BinaryClassification`] over paths.
    pub struct PathClassification;

    impl<'p> BinaryClassification<NormalPath<'p>> for PathClassification {
        type LeftPredicate = IsNsPath;

        type RightPredicate = IsFilePath;
    }

    impl<'p> BinaryClassPredicate<NormalPath<'p>> for IsNsPath {
        type BinClassification = PathClassification;
    }

    impl<'p> BinaryClassPredicate<NormalPath<'p>> for IsFilePath {
        type BinClassification = PathClassification;
    }
}

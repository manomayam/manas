//! I provide few path helpers for custom opendal service
//! implementations.
//!

use std::ops::Deref;

use ecow::EcoString;
use gdp_rs::{binclassified::BinaryClassified, Proven};
use opendal::Error;

/// A struct representing normalized path.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NormalPath(EcoString);

impl Deref for NormalPath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<EcoString> for NormalPath {
    type Error = Error;

    fn try_from(p: EcoString) -> Result<Self, Self::Error> {
        if p.split('/').any(|s| s == "." || s == "..") || p.starts_with('/') || p.contains("//") {
            return Err(Error::new(
                opendal::ErrorKind::ConfigInvalid,
                "Invalid path",
            ));
        }

        Ok(Self(p))
    }
}

impl NormalPath {
    /// Try to create new normal path from opendal path.
    #[inline]
    pub fn try_new(opendal_path: &str) -> Result<Self, Error> {
        Self::try_from(EcoString::from(if opendal_path == "/" {
            ""
        } else {
            opendal_path
        }))
    }

    /// Get as str.
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
    pub fn assert_is_ns_path(self) -> Result<NsPath, Error> {
        NsPath::try_new(self)
            .map_err(|_| Error::new(opendal::ErrorKind::NotADirectory, "Path is not a dir path"))
    }

    /// Assert that path is an ns path.
    pub fn assert_is_file_path(self) -> Result<FilePath, Error> {
        FilePath::try_new(self)
            .map_err(|_| Error::new(opendal::ErrorKind::IsADirectory, "Path is not a file path"))
    }
}

pub use predicates::*;

/// An invariant of [`NormalPath`] which can only represent namespace paths.
///
pub type NsPath = Proven<NormalPath, IsNsPath>;

/// An invariant of [`NormalPath`] which can only represent file paths.
///
pub type FilePath = Proven<NormalPath, IsFilePath>;

/// Binary classified [`NormalPath`].
pub type ClassifiedPath = BinaryClassified<NormalPath, PathClassification>;

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

    impl Predicate<NormalPath> for IsNsPath {
        fn label() -> Cow<'static, str> {
            "IsNsPath".into()
        }
    }

    impl SyncEvaluablePredicate<NormalPath> for IsNsPath {
        type EvalError = IsNotNsPath;

        fn evaluate_for(sub: &NormalPath) -> Result<(), Self::EvalError> {
            if sub.is_ns_path() {
                Ok(())
            } else {
                Err(IsNotNsPath)
            }
        }
    }

    impl PurePredicate<NormalPath> for IsNsPath {}

    /// Path is not a namespace path.
    #[derive(Debug, Clone, thiserror::Error)]
    #[error("Path is not a namespace path.")]
    pub struct IsNotNsPath;

    /// A predicate over [`NsPath`] that asserts it to be a file
    /// path.
    #[derive(Debug)]
    pub struct IsFilePath;

    impl Predicate<NormalPath> for IsFilePath {
        fn label() -> Cow<'static, str> {
            "IsFilePath".into()
        }
    }

    impl SyncEvaluablePredicate<NormalPath> for IsFilePath {
        type EvalError = IsNotFilePath;

        fn evaluate_for(sub: &NormalPath) -> Result<(), Self::EvalError> {
            if sub.is_file_path() {
                Ok(())
            } else {
                Err(IsNotFilePath)
            }
        }
    }

    impl PurePredicate<NormalPath> for IsFilePath {}

    /// Path is not a file path.
    #[derive(Debug, Clone, thiserror::Error)]
    #[error("Path is not a file path.")]
    pub struct IsNotFilePath;

    /// An implementation of [`BinaryClassification`] over paths.
    pub struct PathClassification;

    impl BinaryClassification<NormalPath> for PathClassification {
        type LeftPredicate = IsNsPath;

        type RightPredicate = IsFilePath;
    }

    impl BinaryClassPredicate<NormalPath> for IsNsPath {
        type BinClassification = PathClassification;
    }

    impl BinaryClassPredicate<NormalPath> for IsFilePath {
        type BinClassification = PathClassification;
    }
}

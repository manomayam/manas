//! I define types for representing uri-segment-safe tokens.
//!
use std::{fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref};

use super::invariant::NonEmptyCleanSegmentStr;

/// A trait for representing a segment safe token.
///
/// It ensures that token is a static, and allows for passing it as type param.
pub trait TSegmentSafeToken:
    'static + Debug + Clone + PartialEq + Eq + Hash + Send + Sync + Unpin
{
    /// Get delim token.
    fn token() -> &'static NonEmptyCleanSegmentStr;
}

/// A struct that represents a conflict free token,
/// that doesn't conflict with token provided by token type param.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConflictFreeToken<T: TSegmentSafeToken>(NonEmptyCleanSegmentStr, PhantomData<T>);

#[derive(Debug, Clone, thiserror::Error)]
#[error("Given token is not conflict free.")]
/// Error of Given token not being conflict-free..
pub struct TokenIsNotConflictFree;

impl<T: TSegmentSafeToken> TryFrom<NonEmptyCleanSegmentStr> for ConflictFreeToken<T> {
    type Error = TokenIsNotConflictFree;

    #[inline]
    fn try_from(value: NonEmptyCleanSegmentStr) -> Result<Self, Self::Error> {
        if value.contains(T::token().as_ref().as_ref()) {
            return Err(TokenIsNotConflictFree);
        }
        Ok(Self(value, PhantomData))
    }
}

impl<D: TSegmentSafeToken> TryFrom<&str> for ConflictFreeToken<D> {
    type Error = TokenIsNotConflictFree;

    #[inline]
    fn try_from(token_str: &str) -> Result<Self, Self::Error> {
        let clean_token =
            NonEmptyCleanSegmentStr::try_new_from(token_str).map_err(|_| TokenIsNotConflictFree)?;
        clean_token.try_into()
    }
}

impl<D: TSegmentSafeToken> Deref for ConflictFreeToken<D> {
    type Target = NonEmptyCleanSegmentStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//! This crate provides few recipes of manas solid server
//!

#![warn(missing_docs)]
#![deny(unused_qualifications)]

use std::ops::Deref;

pub mod dtbr;
pub mod pep;
pub mod podverse;
pub mod recipe;
pub mod repo;
pub mod space;
pub mod storage;
pub mod tracing;

/// Crate level wrapper type for quick extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CW<T>(pub T);

impl<T> Deref for CW<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

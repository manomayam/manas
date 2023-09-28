//! I define types for using [Ghosts-of-Departed-Proofs](https://kataskeue.com/gdp.pdf) pattern in rust.
//!

#![warn(missing_docs)]
#![deny(unused_qualifications)]

pub mod binclassified;
pub mod inference_rule;
pub mod predicate;
pub mod proven;

use std::{future::Future, pin::Pin};

pub use proven::Proven;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

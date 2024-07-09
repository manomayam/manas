//! This crate provides utilities to employ [Ghosts-of-Departed-Proofs](https://kataskeue.com/gdp.pdf)
//! pattern in rust projects.
//! This enables type drive development following [Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) principle.
//!
//! The idea is that, instead of checking or validating the values of a Type for
//! satisfying certain properties, one create a wrapper refined new type that
//! can only constructed from the values satisfying desired predicates.
//! Thus, one can centralize the validation mechanism, and use typesystem to
//! enforce right invariant.
//! 
//! This crate provides a type [`Proven`], which is generic over the predicate type.
//! It can be instantiated only from valid subject values of a type, satisfying the 
//! predicate function.
//! 
//! The crate also provides combinatrics over multiple predicates, so that one
//! compose a compound predicate type from list of other predicate type.
//! 

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

pub mod binclassified;
pub mod inference_rule;
pub mod predicate;
pub mod proven;

use std::{future::Future, pin::Pin};

pub use proven::Proven;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

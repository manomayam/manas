//! This helper crate provides few utils to generate code for `manas_specs` crate.
//! It is not part of public api of the manas project.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#[deny(unused_qualifications)]
pub mod gen_spec_mod;
pub mod templates;
pub mod util;

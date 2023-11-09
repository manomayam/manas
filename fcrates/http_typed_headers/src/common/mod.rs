//! I define few common structs to define
//! representing common rules for typed headers.
//!

pub mod field;
#[cfg(feature = "media-type")]
pub mod media_type;
#[cfg(feature = "qvalue")]
pub mod qvalue;

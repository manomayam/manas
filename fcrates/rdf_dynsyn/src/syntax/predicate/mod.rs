//! I define few predicates over rdf syntaxes.
//!

mod is_dynsyn_parsable;
mod is_dynsyn_serializable;

mod is_dataset_encoding;
mod is_graph_encoding;

pub use is_dataset_encoding::*;
pub use is_dynsyn_parsable::*;
pub use is_dynsyn_serializable::*;
pub use is_graph_encoding::*;

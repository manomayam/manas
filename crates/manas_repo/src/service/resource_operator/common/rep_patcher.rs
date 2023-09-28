//! I define representation patcher service.
//!

use std::{collections::HashSet, fmt::Debug};

use dyn_problem::define_anon_problem_types;
use manas_space::resource::operation::SolidResourceOperation;

/// A patcher patches given representation.
/// And it also resolves effective resource ops of the patch
/// operation.
///
pub trait RepPatcher: Debug + Send + Sync + 'static {
    /// Resolve effective operations this patch performed over
    /// target resource.
    fn effective_ops(&self) -> HashSet<SolidResourceOperation>;
}

define_anon_problem_types!(
    /// Un supported patch source content type.
    INCOMPATIBLE_PATCH_SOURCE_CONTENT_TYPE: ("Un supported patch source content type.");

    /// Invalid encoded source rep.
    INVALID_ENCODED_SOURCE_REP: ("Invalid encoded source rep.");

    /// Patch semantics error.
    PATCH_SEMANTICS_ERROR: ("Patch semantics error.");
);

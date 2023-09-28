//! I define few predicates over [`ODRObject`].
//!

use std::marker::PhantomData;

use gdp_rs::binclassified::BinaryClassification;

use self::{is_file_object::IsFileObject, is_namespace_object::IsNamespaceObject};
use super::ODRObject;
use crate::object_store::ODRObjectStoreSetup;

/// I define [`IsNamespaceObject`] predicate over a [`ODRObject`].
pub mod is_namespace_object;

/// I define [`IsFileObject`] predicate over a [`ODRObject`].
pub mod is_file_object;

/// An implementation of [`BinaryClassification`] over
/// odr objects based on kind of the object.
pub struct ObjectKindBasedClassification<OstSetup> {
    _phantom: PhantomData<fn(OstSetup)>,
}

impl<'id, OstSetup: ODRObjectStoreSetup> BinaryClassification<ODRObject<'id, OstSetup>>
    for ObjectKindBasedClassification<OstSetup>
{
    type LeftPredicate = IsNamespaceObject<OstSetup>;

    type RightPredicate = IsFileObject<OstSetup>;
}

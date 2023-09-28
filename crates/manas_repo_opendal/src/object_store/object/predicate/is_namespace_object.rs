use std::marker::PhantomData;

use gdp_rs::{
    binclassified::BinaryClassPredicate,
    predicate::{Predicate, PurePredicate, SyncEvaluablePredicate},
};

use super::ObjectKindBasedClassification;
use crate::object_store::{object::ODRObject, ODRObjectStoreSetup};

/// A [`Predicate`] over subject [`ODRObject`] stating that
/// it is a namespace object.
#[derive(Clone)]
pub struct IsNamespaceObject<OstSetup: ODRObjectStoreSetup> {
    _phantom: PhantomData<fn(OstSetup) -> bool>,
}

impl<OstSetup: ODRObjectStoreSetup> std::fmt::Debug for IsNamespaceObject<OstSetup> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsNamespaceObject").finish()
    }
}

impl<'id, OstSetup: ODRObjectStoreSetup> Predicate<ODRObject<'id, OstSetup>>
    for IsNamespaceObject<OstSetup>
{
    fn label() -> std::borrow::Cow<'static, str> {
        "IsNamespaceObject".into()
    }
}

/// Check if a given object is a namespace object.
pub(super) fn is_namespace_object<OstSetup: ODRObjectStoreSetup>(
    object: &ODRObject<'_, OstSetup>,
) -> bool {
    // If path ends with a slash, or is empty then it is a namespace object.
    object.id.root_relative_path.is_namespace_path()
}

impl<'id, OstSetup: ODRObjectStoreSetup> SyncEvaluablePredicate<ODRObject<'id, OstSetup>>
    for IsNamespaceObject<OstSetup>
{
    type EvalError = IsNotANamespaceObject;

    #[inline]
    fn evaluate_for(sub: &ODRObject<'id, OstSetup>) -> Result<(), Self::EvalError> {
        if is_namespace_object(sub) {
            Ok(())
        } else {
            Err(IsNotANamespaceObject)
        }
    }
}

impl<'id, OstSetup: ODRObjectStoreSetup> PurePredicate<ODRObject<'id, OstSetup>>
    for IsNamespaceObject<OstSetup>
{
}

impl<'id, OstSetup: ODRObjectStoreSetup> BinaryClassPredicate<ODRObject<'id, OstSetup>>
    for IsNamespaceObject<OstSetup>
{
    type BinClassification = ObjectKindBasedClassification<OstSetup>;
}

#[derive(Debug, thiserror::Error)]
#[error("Given odr object is not a namespace object.")]
/// Error of an odr object not being a namespace object.
pub struct IsNotANamespaceObject;

// #[cfg(test)]
// #[cfg(feature = "test-utils")]
// mod tests {
//     use crate::object_store::{
//         mock::MockODRObjectStore, object_id::normal_rootless_uri_path::NormalRootlessUriPath,
//         object_space::mock::MockODRObjectSpace, ODRObjectStoreExt,
//     };
//     use claims::{assert_err, assert_ok};
//     use gdp_rs::Proven;
//     use rstest::*;

//     use super::*;

//     /// Mock storage root uri.
//     static MOCK_STORAGE_ROOT_URI_STR: &str = "http://ex.org/mock/";

//     #[rstest]
//     #[case("", true)]
//     #[case("a", false)]
//     #[case("a$aux/", true)]
//     #[case("a/b/c/", true)]
//     #[case("a/b/c/d", false)]
//     fn predicate_eval_works_correctly(
//         #[case] odr_object_path: &'static str,
//         #[case] expected: bool,
//     ) {
//         let object_store = MockODRObjectStore::<0>::new_mock(MockODRObjectSpace::new_mock(
//             MOCK_STORAGE_ROOT_URI_STR,
//         ));

//         let odr_object = object_store
//             .odr_object(unsafe { NormalRootlessUriPath::new_unchecked(odr_object_path.into()) })
//             .unwrap();

//         if expected {
//             assert_ok!(Proven::<_, IsNamespaceObject<_>>::try_new(odr_object));
//         } else {
//             assert_err!(Proven::<_, IsNamespaceObject<_>>::try_new(odr_object));
//         }
//     }
// }

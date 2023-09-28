use std::marker::PhantomData;

use crate::object_store::{
    backend::ODRObjectStoreBackend, object_space::ODRObjectSpaceSetup, ODRObjectStoreSetup,
};

/// A basic implementation of [`ODRObjectStoreSetup`].
pub struct BasicODRObjectStoreSetup<OspSetup, Backend> {
    _phantom: PhantomData<fn(OspSetup, Backend)>,
}

impl<OspSetup, Backend> std::fmt::Debug for BasicODRObjectStoreSetup<OspSetup, Backend> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicODRObjectStoreSetup").finish()
    }
}

impl<OspSetup, Backend> ODRObjectStoreSetup for BasicODRObjectStoreSetup<OspSetup, Backend>
where
    OspSetup: ODRObjectSpaceSetup,
    Backend: ODRObjectStoreBackend,
{
    type AssocStSpace = OspSetup::AssocStSpace;

    type ObjectSpaceSetup = OspSetup;

    type Backend = Backend;
}

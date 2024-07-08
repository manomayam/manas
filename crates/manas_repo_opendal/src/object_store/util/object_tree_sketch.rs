//! I define few types for sketching an object tree.
//!

use std::borrow::Cow;

use async_recursion::async_recursion;
use build_fs_tree::FileSystemTree;
use tower::BoxError;

use crate::object_store::{
    object::invariant::{
        ODRFileObject, ODRFileObjectExt, ODRNamespaceObject, ODRNamespaceObjectExt,
    },
    object_id::normal_rootless_uri_path::NormalRootlessUriPath,
    ODRObjectStore, ODRObjectStoreSetup,
};

/// A type for representing odr object tree sketch.
#[derive(Debug, Clone)]
pub struct ODRObjectTreeSketch<'s> {
    /// Path of object tree root relative to object space root.
    tree_root_path: NormalRootlessUriPath<'static>,

    /// Object tree sketch.
    tree_content_sketch: Cow<'s, FileSystemTree<String, String>>,
}

impl<'s> ODRObjectTreeSketch<'s> {
    /// Create a new [`ODRObjectTreeSketch`] with given params.
    pub fn try_new(
        tree_root_path: NormalRootlessUriPath<'static>,
        tree_content_sketch: Cow<'s, FileSystemTree<String, String>>,
    ) -> Result<Self, InvalidODRObjectTreeSketchParams> {
        let is_namespace_path = tree_root_path.is_namespace_path();
        let is_dir_content_sketch = tree_content_sketch.dir_content().is_some();

        if is_namespace_path != is_dir_content_sketch {
            return Err(InvalidODRObjectTreeSketchParams::ObjectKindConflict);
        }

        Ok(Self {
            tree_root_path,
            tree_content_sketch,
        })
    }

    /// Setup the tree represented by this sketch in given object store.
    #[async_recursion]
    pub async fn setup<OstSetup>(
        &self,
        object_store: &ODRObjectStore<OstSetup>,
    ) -> Result<(), BoxError>
    where
        OstSetup: ODRObjectStoreSetup,
    {
        let root_odr_object = object_store.odr_object(self.tree_root_path.clone())?;

        match self.tree_content_sketch.as_ref() {
            // If tree root is a file, persist it's content.
            FileSystemTree::File(file_data) => {
                ODRFileObject::<'static, _>::try_new(root_odr_object)
                    .expect("Must be file object, as previously checked.")
                    .write(file_data.as_bytes().to_vec(), &Default::default())
                    .await?;
                Ok(())
            }

            // If tree root is a directory, then create
            // it's corresponding object, and recursively create it's children.
            FileSystemTree::Directory(child_branches) => {
                // Create backend object.
                ODRNamespaceObject::try_new(root_odr_object)
                    .expect("Must be namespace object, as checked before.")
                    .create()
                    .await?;

                for (cb_root_name, cb_content_sketch) in child_branches {
                    // Compute child branch root path.
                    let cb_root_path = unsafe {
                        NormalRootlessUriPath::new_unchecked(
                            format!(
                                "{}{}{}",
                                self.tree_root_path.as_ref(),
                                cb_root_name,
                                // Append slash if child branch root is a directory.
                                if cb_content_sketch.file_content().is_some() {
                                    ""
                                } else {
                                    "/"
                                }
                            )
                            .into(),
                        )
                    };

                    let cb_sketch = ODRObjectTreeSketch::try_new(
                        cb_root_path,
                        Cow::Borrowed(cb_content_sketch),
                    )
                    .expect("Must be valid params.");

                    cb_sketch.setup(object_store).await?;
                }
                Ok(())
            }
        }
    }
}

/// An error type for invalid object tree sketch params.
#[derive(Debug, thiserror::Error)]
pub enum InvalidODRObjectTreeSketchParams {
    /// Object kind conflict.
    #[error("Object kind conflict.")]
    ObjectKindConflict,
}

//! I provide few implementations of [`SinglePodRecipeSetup`](super::SinglePodRecipeSetup).
//!

#[cfg(feature = "backend-fs")]
crate::define_single_pod_noauth_recipe_setup!(fs);

#[cfg(feature = "backend-s3")]
crate::define_single_pod_noauth_recipe_setup!(s3);

#[cfg(feature = "backend-gcs")]
crate::define_single_pod_noauth_recipe_setup!(gcs);

/// Define single pod recipe setup.
#[macro_export(local_inner_macros)]
macro_rules! define_single_pod_noauth_recipe_setup {
    ($backend:ident) => {
        paste::paste! {
            pub use [<$backend:lower>]::*;

            mod [<$backend:lower >] {
                use manas_repo_opendal::object_store::backend::impl_::[<$backend:lower>]::[<$backend:camel Backend>];

                use $crate::{
                    recipe::impl_::single_pod_noauth::setup::SinglePodNoAuthRecipeSetup
                };

                /// An implementation of [`SinglePodRecipeSetup`]
                #[derive(Debug)]
                pub struct [<$backend:camel NoAuthRecipeSetup>];

                impl SinglePodNoAuthRecipeSetup for [<$backend:camel NoAuthRecipeSetup>] {
                    const BACKEND_NAME: &'static str = std::stringify!([<$backend:lower>]);

                    type BackendBuilder = opendal::services::[<$backend:camel>];

                    type Backend = [<$backend:camel Backend>];
                }
            }

        }
    };
}

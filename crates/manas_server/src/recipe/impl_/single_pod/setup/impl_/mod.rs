//! I provide few implementations of [`SinglePodRecipeSetup`](super::SinglePodRecipeSetup).
//!

#[cfg(all(feature = "backend-fs", feature = "pdp-wac"))]
crate::define_single_pod_recipe_setup!(fs, wac);

#[cfg(all(feature = "backend-fs", feature = "pdp-acp"))]
crate::define_single_pod_recipe_setup!(fs, acp);

#[cfg(all(feature = "backend-s3", feature = "pdp-wac"))]
crate::define_single_pod_recipe_setup!(s3, wac);

#[cfg(all(feature = "backend-s3", feature = "pdp-acp"))]
crate::define_single_pod_recipe_setup!(s3, acp);

#[cfg(all(feature = "backend-gcs", feature = "pdp-wac"))]
crate::define_single_pod_recipe_setup!(gcs, wac);

#[cfg(all(feature = "backend-gcs", feature = "pdp-acp"))]
crate::define_single_pod_recipe_setup!(gcs, acp);

/// Define single pod recipe setup.
#[macro_export(local_inner_macros)]
macro_rules! define_single_pod_recipe_setup {
    ($backend:ident, $pdp:ident) => {
        paste::paste! {
            pub use [<$backend:lower _$pdp:lower>]::*;

            mod [<$backend:lower _$pdp:lower>] {
                use std::collections::HashSet;

                use manas_access_control::model::pdp::impl_::[<$pdp:lower>]::[<$pdp:camel DecisionPoint>];
                use manas_repo_opendal::object_store::backend::impl_::[<$backend:lower>]::[<$backend:camel Backend>];
                use rdf_utils::model::triple::ArcTriple;

                use $crate::{
                    pep::[<$pdp:snake:upper _INITIAL_ROOT_ACR_TEMPLATE_STR>],
                    recipe::impl_::single_pod::setup::SinglePodRecipeSetup, space::RcpStorageSpace,
                };

                /// An implementation of [`SinglePodRecipeSetup`]
                #[derive(Debug)]
                pub struct [<$backend:camel $pdp:camel RecipeSetup>];

                impl SinglePodRecipeSetup for [<$backend:camel $pdp:camel RecipeSetup>] {
                    const BACKEND_NAME: &'static str = std::stringify!([<$backend:lower>]);

                    const PDP_NAME: &'static str = std::stringify!([<$pdp:lower>]);

                    const INITIAL_ROOT_ACR_TEMPLATE: &'static str = [<$pdp:snake:upper _INITIAL_ROOT_ACR_TEMPLATE_STR>];

                    type PDP = [<$pdp:camel DecisionPoint>]<RcpStorageSpace, HashSet<ArcTriple>>;

                    type BackendBuilder = opendal::services::[<$backend:camel>];

                    type Backend = [<$backend:camel Backend>];
                }
            }

        }
    };
}

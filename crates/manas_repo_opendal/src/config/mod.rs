//! I define types to record ODR configuration.
//!

use std::sync::Arc;

use rdf_dynsyn::DynSynFactorySet;

/// Configuration struct for ODR.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ODRConfig {
    // /// Size bounds on user supplied rep data.
    // pub user_supplied_rep_data_size_bounds: ODRUserSuppliedRepDataSizeBounds,
    /// Container  representation policy.
    pub container_rep_policy: ODRContainerRepPolicy,

    /// Dynsyn factories.
    pub dynsyn_factories: Arc<DynSynFactorySet>,
}

// /// A struct representing bounds on user supplied data.
// #[non_exhaustive]
// #[derive(Debug, Clone, Default)]
// pub struct ODRUserSuppliedRepDataSizeBounds {
//     /// Maximum size for user supplied container rep data.
//     /// It is recommended to set this to finite value, as container
//     /// rep data will be loaded entirely into memory for validation.
//     pub max_container_rep_data_size: Option<u64>,

//     /// Maximum size for user supplied non container rep data.
//     pub max_non_container_rep_data_size: Option<u64>,

//     /// Maximum size for user supplied rdf source aux resources  rep data.
//     /// It is recommended to set this to finite value, as ldp-rs aux
//     /// rep data will be loaded entirely into memory for validation.
//     pub max_rdf_source_aux_rep_data_size: Option<u64>,

//     /// Maximum size for user supplied patch rep data.
//     /// It is recommended to set this to finite value, as patch
//     /// rep data will be loaded entirely into memory for patching.
//     pub max_patch_doc_payload_size: Option<u64>,
// }

// impl ODRUserSuppliedRepDataSizeBounds {
//     /// Resolve max rep data size for resource with given context.
//     pub(crate) fn resolve_max_rep_data_size<Setup: ODRSetup>(
//         &self,
//         res_context: &ODRResourceContext,
//     ) -> Option<u64> {
//         if res_context.kind() == SolidResourceKind::Container {
//             self.max_container_rep_data_size
//         } else if res_context.slot().is_rdf_source_aux_res_slot() {
//             self.max_rdf_source_aux_rep_data_size
//         } else {
//             self.max_non_container_rep_data_size
//         }
//     }

// /// Resolve max rep data size for resource with given slot.
// pub fn resolve_max_rep_data_size<Setup: ODRSetup>(
//     &self,
//     res_context: &ODRResourceContext<'_, Setup>,
// ) -> Option<u64> {
//     if res_context.res_kind() == ResourceKind::Container {
//         self.max_container_rep_data_size
//     } else if res_context.slot().is_rdf_source_aux_res_slot() {
//         self.max_rdf_source_aux_rep_data_size
//     } else {
//         self.max_non_container_rep_data_size
//     }
// }
// }

/// Policy for container representation in ODR.
#[derive(Debug, Default, Clone)]
pub struct ODRContainerRepPolicy {
    // /// Number of maximum containment triples to serve in container representation.
    // pub max_containment_triples: Option<usize>,
    // TODO Must include some overflow hint in container rep if max exceeded.
}

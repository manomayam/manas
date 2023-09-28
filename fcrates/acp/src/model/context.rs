//! I define handle and description types for acp access contexts.
//!

use std::sync::Arc;

use rdf_utils::{
    define_handle_and_description_types,
    model::{description::SimpleDescription, graph::InfallibleGraph},
};
use rdf_vocabularies::ns::acp;

use super::resource::HResource;

define_handle_and_description_types!(
    /// A handle type for [acp access context](https://solid.github.io/authorization-panel/acp-specification/#context).
    ///
    /// > Instances of the Context class describe instances of resource access.
    HContext;
    /// A type alias for acp access context description.
    DContext;
    [
        /// > The target attribute describes requested resources.
        (target, &acp::target, HResource);
    ]
);

impl<G: InfallibleGraph> DContext<G, G> {
    /// Convert into nre context with inner graph wrapped in an arc.
    #[inline]
    pub fn into_with_arced_graph(self) -> DContext<G, Arc<G>> {
        let (h, g) = self.into_parts();
        DContext::new(h, Arc::new(g))
    }
}

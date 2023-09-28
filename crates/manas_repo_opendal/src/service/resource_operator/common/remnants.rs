//! I define few utils to deal with remnant objects.
//!

use std::sync::Arc;

use futures::{stream::FuturesUnordered, FutureExt, TryFutureExt, TryStreamExt};
use tracing::warn;

use crate::{
    object_store::object_space::assoc::rel_type::AssocRelType,
    resource_context::ODRResourceContext, setup::ODRSetup,
};

/// Try purging any remnants associated with resource with given slot.
#[tracing::instrument()]
pub async fn purge_remnants<Setup>(
    res_context: &Arc<ODRResourceContext<Setup>>,
) -> Result<(), opendal::Error>
where
    Setup: ODRSetup,
{
    // Delete recursively for each assoc object path, and
    // return error if any of them fail.
    AssocRelType::ALL
        .iter()
        // Get recursive delete future on odr object path.
        .map(|assoc_rel_type| {
            let object = &res_context.as_ref().assoc_odr_object_map()[*assoc_rel_type];

            object.is_exist().and_then(|is_exist| {
                if is_exist {
                    warn!(
                        "Remnant object exists at {}",
                        object.id().root_relative_path.as_ref()
                    );
                    object.delete_recursive().boxed()
                } else {
                    futures::future::ready(Ok(())).boxed()
                }
            })
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await
        .map(|_| ())
}

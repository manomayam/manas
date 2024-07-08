//! I define types for resolving container index.
//!

use std::{ops::Deref, sync::Arc};

use dashmap::DashMap;
use futures::TryFutureExt;
use manas_http::{
    header::{common::media_type::MediaType, last_modified::LastModifiedExt},
    representation::{
        impl_::common::data::quads_stream::BoxQuadsStream,
        metadata::{KCompleteContentLength, KContentType, KLastModified},
    },
};
use manas_space::resource::kind::SolidResourceKind;
use once_cell::sync::Lazy;
use rdf_utils::model::term::{ArcIri, ArcTerm, CompatTerm};
use rdf_vocabularies::ns;
use sophia_api::{ns::NsTerm, prelude::Iri, term::Term};
use tracing::{error, warn};
use typed_record::TypedRecord;

use super::{query_altfm_obj_content, ODRResourceStatusTokenInputs};
use crate::{
    object_store::{
        object::{
            invariant::{DecodedODRObjectStream, ODRNamespaceObjectExt},
            ODRObject,
        },
        object_space::assoc::rel_type::AssocRelType,
    },
    resource_context::{
        invariant::{ODRClassifiedResourceContext, ODRContainerContext},
        ODRResourceContext,
    },
    service::resource_operator::common::status_token::variant::ODRExistingRepresentedResourceToken,
    setup::ODRSetup,
};

/// ldp types for a container.
pub static CONTAINER_LDP_TYPES: &[NsTerm] = &[
    ns::ldp::BasicContainer,
    ns::ldp::Container,
    ns::ldp::Resource,
];

// Private static cache for content type iana iris.
static CONTENT_TYPE_RFC6570_IRI_CACHE: Lazy<DashMap<String, ArcIri>> = Lazy::new(DashMap::new);

/// Get iana iri for given content type.
pub fn rfc6570_iri_for_content_type(content_type: &MediaType) -> ArcIri {
    if !CONTENT_TYPE_RFC6570_IRI_CACHE.contains_key(content_type.essence_str()) {
        // Body kind URI for bytes is the expansion of the URI Template [RFC6570]
        // `http://www.w3.org/ns/iana/media-types/{+iana-media-type}#Resource`,
        // where iana-media-type corresponds to a value from the IANA Media Types.
        CONTENT_TYPE_RFC6570_IRI_CACHE.insert(
            content_type.essence_str().to_owned(),
            Iri::new_unchecked(Arc::from(format!(
                "http://www.w3.org/ns/iana/media-types/{}#Resource",
                content_type.essence_str()
            ))),
        );
    }

    CONTENT_TYPE_RFC6570_IRI_CACHE
        .get(content_type.essence_str())
        .expect("Must be some")
        .clone()
}

/// A struct for representing inputs to resolve container index.
#[derive(Debug, Clone)]
pub struct ODRContainerIndexInputs<Setup: ODRSetup> {
    /// Context of the container resource.
    pub c_res_context: ODRContainerContext<Setup>,
}

impl<Setup: ODRSetup> ODRContainerIndexInputs<Setup> {
    /// Resolve container index.
    #[tracing::instrument(
        skip_all,
        name = "ODRContainerIndexInputs::resolve",
        fields(
            res_uri = self.c_res_context.uri().as_str()
        )
    )]
    pub async fn resolve(&self) -> Result<BoxQuadsStream, opendal::Error> {
        let c_res_context = self.c_res_context.as_ref().as_ref();

        let repo_context = c_res_context.repo_context().clone();

        let container_uri = c_res_context.uri().clone();

        // Resolve container name.
        let container_name = container_uri.deref().into_term::<ArcTerm>();

        // Get predicate terms.
        let p_contains: ArcTerm = ns::ldp::contains.into_term();
        let p_size: ArcTerm = ns::stat::size.into_term();
        let p_type: ArcTerm = ns::rdf::type_.into_term();
        let p_modified: ArcTerm = ns::dcterms::modified.into_term();

        // Get container indicator object, i.e base object.
        let indicator_object = c_res_context
            .assoc_odr_object_map()
            .base_object()
            .as_left_classified()
            .expect("A container's associated base object must be a namespace object.");

        let ns_listing: DecodedODRObjectStream<Setup::ObjectStoreSetup> = indicator_object
            .list()
            .inspect_err(|_| error!("error in getting associated objects of contained resources."))
            .await?;

        Ok(Box::pin(async_stream::stream! {
            for await item in ns_listing {
                let odr_object: ODRObject<Setup::ObjectStoreSetup> = match item {
                    Ok(obj) => obj,
                    Err(e) => {
                        // Skip if there is any error.
                        error!("Error in listing odr object. Error:\n {}", e);
                        // Yield error.
                        yield Err(e.into());
                        continue
                    }
                };

                // Compute association for odr object.
                let assoc_rev_link = match odr_object.assoc_rev_link() {
                    Ok(rev_link) => rev_link,
                    Err(e) => {
                        warn!("Error in computing association rev link for an object. Error:\n {}", e);
                        // Skip the non associated object.
                        continue;
                    }
                };

                // If object is not a plausible base object then skip it.
                if assoc_rev_link.rev_rel_type != AssocRelType::Base {
                        continue;
                };

                // compute the child resource slot.
                let assoc_res_context = match ODRResourceContext::try_new(assoc_rev_link.target.uri, repo_context.clone()) {
                    Ok(slot) => ODRClassifiedResourceContext::new(Arc::new(slot)),
                    Err(e) => {
                        warn!("Error in computing resource slot for child resource. Error:\n {}", e);
                        // Skip the slotless association.
                        continue;
                    }
                };

                let child_name = assoc_res_context.uri().deref().into_term::<ArcTerm>();

                /// Yield the containment quad.
                yield Ok(([container_name.clone(), p_contains.clone(), child_name.clone()], None));

                // Get base object metadata.
                // In most cloud backends, this will be no op,
                // as metadata would have been cached during listing.
                let base_obj_metadata = match odr_object.metadata().await {
                    Ok(metadata) => metadata,
                    Err(_) => {
                        warn!("Error in getting object metadata.");
                        // Skip yielding metadata quads.
                        continue;
                    }
                };

                // Construct status inputs.
                let mut child_status_inputs = ODRResourceStatusTokenInputs {
                    res_context:assoc_res_context.clone(),
                    slot_path_is_represented: true,
                    base_obj_metadata: Some(base_obj_metadata),
                    // Ignore altcontent object for containers.
                    // TODO should better be considered in case of containers.
                    altcontent_obj_metadata: None,
                    altfm_obj_content: None,
                };

                // If backend is not cty capable, then consider altfm object.
                if !assoc_res_context.repo_context().object_store.is_cty_capable_backend() {
                    child_status_inputs.altfm_obj_content = match query_altfm_obj_content(assoc_res_context.clone()).await {
                        Ok(content) => content,
                        Err(e) => {
                            warn!("Error in querying fat alt metadata for resource. Error:\n {}", e);
                            // Skip yielding metadata quads.
                            continue;
                        }
                    };
                }

                let child_status_token = ODRExistingRepresentedResourceToken::try_from(child_status_inputs).expect("Must be valid, as base_object exists.");

                // Resolve child rep metadata.
                let child_rep_metadata = match child_status_token.try_resolve_user_supplied_rep_metadata() {
                    Ok(metadata) => metadata,
                    Err(e) => {
                        warn!("Error in resolving representation metadata for the child resource. Error:\n {}", e);
                        // Skip yielding metadata quads.
                        continue;
                    }
                };

                let child_context = assoc_res_context.as_ref().as_ref();

                // Yield the metadata quads.

                // If container, set ldp types.
                if child_context.kind() == SolidResourceKind::Container {
                    for type_ in CONTAINER_LDP_TYPES.iter() {
                        yield Ok(([child_name.clone(), p_type.clone(), type_.into_term()], None));
                    }
                }
                // Else set rfc6570 resource type from it's
                // content type..
                else if let Some(child_rep_content_type) = child_rep_metadata.get_rv::<KContentType>() {
                    yield Ok(([child_name.clone(), p_type.clone(), rfc6570_iri_for_content_type(child_rep_content_type).into_term()], None));
                };

                // Resource last modified.
                if let Some(rep_last_modified) = child_rep_metadata.get_rv::<KLastModified>() {
                    yield Ok(([child_name.clone(), p_modified.clone(), CompatTerm(rep_last_modified.to_date_time()).into_term()], None));
                }

                // Rep's content length.
                if child_context.kind() != SolidResourceKind::Container {
                    if let Some(rep_content_length) = child_rep_metadata.get_rv::<KCompleteContentLength>() {
                        yield Ok(([child_name.clone(), p_size.clone(), CompatTerm(rep_content_length.0).into_term()], None));
                    }
                }
            }
        }))
    }
}

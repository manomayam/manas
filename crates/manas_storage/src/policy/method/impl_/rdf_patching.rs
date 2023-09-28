use std::ops::Deref;

use http::Method;
use if_chain::if_chain;
use manas_http::header::{accept_patch::AcceptPatch, common::media_type::MediaType};
use rdf_dynsyn::{correspondence::Correspondent, syntax::RdfSyntax};
use vec1::Vec1;

use crate::policy::method::MethodPolicy;

/// A basic implementation of [`MethodPolicy`], that
/// resolves accept-patch for only rdf resources.
#[derive(Debug, Clone)]
pub struct RdfPatchingMethodPolicy {
    /// List of supported methods.
    pub supported_methods: Vec<Method>,

    /// Optional list of one or more supported rdf patch doc media types.
    pub supported_rdf_patch_types: Option<Vec1<MediaType>>,
}

impl Default for RdfPatchingMethodPolicy {
    fn default() -> Self {
        Self {
            // Support all methods by default.
            supported_methods: vec![
                Method::HEAD,
                Method::GET,
                Method::PUT,
                Method::PATCH,
                Method::POST,
                Method::DELETE,
            ],
            // Support n3 patch by default.
            supported_rdf_patch_types: Some(Vec1::new(
                "text/n3".parse().expect("Must be valid media type"),
            )),
        }
    }
}

impl MethodPolicy for RdfPatchingMethodPolicy {
    #[inline]
    fn supported_methods(&self) -> &[Method] {
        &self.supported_methods
    }

    fn accept_patch_for_existing(&self, rep_content_type: &MediaType) -> Option<AcceptPatch> {
        if_chain! {
            // If content_type has correspondent rdf concrete syntax,
            if let Ok(correspondent_syntax) = Correspondent::<RdfSyntax>::try_from(rep_content_type.deref());
            if correspondent_syntax.is_total;
            // If any rdf resource patch types supported
            if let Some(rdf_patch_types)  = self.supported_rdf_patch_types.as_ref();
            then {
                Some(AcceptPatch {
                    media_types: rdf_patch_types.clone()
                })
            }
            else {
                None
            }
        }
    }

    #[inline]
    fn accept_patch_for_non_existing(&self) -> Option<AcceptPatch> {
        self.supported_rdf_patch_types
            .as_ref()
            .map(|rdf_patch_types| AcceptPatch {
                media_types: rdf_patch_types.clone(),
            })
    }
}

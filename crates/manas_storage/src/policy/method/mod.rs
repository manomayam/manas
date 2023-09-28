//! I define traits and types to represent storage's method
//! policy.
//!

pub mod impl_;

use headers::{Allow, HeaderMap, HeaderMapExt};
use http::Method;
use if_chain::if_chain;
use manas_http::header::{
    accept_patch::AcceptPatch, accept_post::AcceptPost, accept_put::AcceptPut,
    common::media_type::MediaType,
};
use manas_space::{
    resource::{kind::SolidResourceKind, slot::SolidResourceSlot},
    SolidStorageSpace,
};

/// A trait for defining method policy of a storage.
pub trait MethodPolicy {
    /// Get list of supported methods.
    fn supported_methods(&self) -> &[Method];

    /// Resolve Accept-Post header, for a container.
    fn accept_post_for_container(&self) -> Option<AcceptPost> {
        Some(AcceptPost {
            media_ranges: vec![mime::STAR_STAR],
        })
    }

    /// Resolve Accept-Put header, for existing resource with given rep content-type.
    #[allow(unused_variables)]
    fn accept_put_for_existing(&self, rep_content_type: &MediaType) -> Option<AcceptPut> {
        Some(AcceptPut {
            media_ranges: vec![mime::STAR_STAR],
        })
    }

    /// Resolve Accept-Put header, if resource doesn't exist at a request target.
    #[inline]
    fn accept_put_for_non_existing(&self) -> Option<AcceptPut> {
        Some(AcceptPut {
            media_ranges: vec![mime::STAR_STAR],
        })
    }

    /// Resolve Accept-Patch header, for existing resource with given rep content-type.
    fn accept_patch_for_existing(&self, rep_content_type: &MediaType) -> Option<AcceptPatch>;

    /// Resolve Accept-Patch header, if resource doesn't exist at a request target.
    fn accept_patch_for_non_existing(&self) -> Option<AcceptPatch>;
}

mod seal {
    use super::MethodPolicy;

    pub trait Sealed {}

    impl<T: MethodPolicy> Sealed for T {}
}

/// An extension trait for [`MethodPolicy`] with utility methods.
pub trait MethodPolicyExt: MethodPolicy + seal::Sealed {
    /// Methods allowed for creating a new resource with given uri.
    const NEW_RESOURCE_ALLOWED_METHODS: &'static [Method] = &[Method::PUT, Method::PATCH];

    /// Resolves allow for an existing resource.
    fn resolve_allowed_methods_for_existing<StSpace: SolidStorageSpace>(
        &self,
        res_slot: &SolidResourceSlot<StSpace>,
    ) -> Vec<Method> {
        self.supported_methods()
            .iter()
            .filter_map(|method| {
                // Disallow delete on storage root or it's acl.
                if (*method == Method::DELETE && (res_slot.is_root_slot() || res_slot.is_root_acl_slot()))
                // Disallow post on non containers.
                || (*method == Method::POST && res_slot.res_kind() != SolidResourceKind::Container)
                {
                    None
                } else {
                    Some(method.clone())
                }
            })
            .collect()
    }

    /// Resolve allowed methods for non existing resource.
    fn resolve_allowed_methods_for_non_existing(&self) -> Vec<Method> {
        Self::NEW_RESOURCE_ALLOWED_METHODS
            .iter()
            .filter_map(|m| {
                if self.supported_methods().contains(m) {
                    Some(m.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Set `Allow` , `Accept-<Method>` headers for an existing resource.
    #[inline]
    fn set_allow_accept_headers_for_existing<StSpace: SolidStorageSpace>(
        &self,
        headers: &mut HeaderMap,
        res_slot: &SolidResourceSlot<StSpace>,
        rep_content_type: &MediaType,
    ) {
        // Resolve allowed methods.
        let allowed_methods = self.resolve_allowed_methods_for_existing(res_slot);

        // Set `Accept-Post`.
        if_chain! {
            if allowed_methods.contains(&Method::POST);
            if let Some(accept_post) = self.accept_post_for_container();
            then {
                headers.typed_insert(accept_post);
            }
        }

        // Set `Accept-Put`.
        if_chain! {
            if allowed_methods.contains(&Method::PUT);
            if let Some(accept_put) = self.accept_put_for_existing(
                rep_content_type
            );
            then {
                headers.typed_insert(accept_put);
            }
        }

        // Set `Accept-Patch`.
        if_chain! {
            if allowed_methods.contains(&Method::PATCH);
            if let Some(accept_patch) = self.accept_patch_for_existing(
                rep_content_type
            );
            then {
                headers.typed_insert(accept_patch);
            }
        }

        // Set Allow
        headers.typed_insert(Allow::from_iter(allowed_methods));
    }

    /// Set `Allow`, `Accept-<Method>` header for a non existing resource
    #[inline]
    fn set_allow_accept_headers_for_non_existing(&self, headers: &mut HeaderMap) {
        // Resolve allowed methods.
        let allowed_methods = self.resolve_allowed_methods_for_non_existing();

        // Set `Accept-Put`.
        if_chain! {
            if allowed_methods.contains(&Method::PUT);
            if let Some(accept_put) = self.accept_put_for_non_existing();
            then {
                headers.typed_insert(accept_put);
            }
        }

        // Set `Accept-Patch`.
        if_chain! {
            if allowed_methods.contains(&Method::PATCH);
            if let Some(accept_patch) = self.accept_patch_for_non_existing();
            then {
                headers.typed_insert(accept_patch);
            }
        }

        // Set Allow
        headers.typed_insert(Allow::from_iter(allowed_methods));
    }
}

impl<T: MethodPolicy> MethodPolicyExt for T {}

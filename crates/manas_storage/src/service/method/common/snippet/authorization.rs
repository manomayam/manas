//! I provide few snippets to deal with authorization
//! handling in method implementations.
//!

use http::StatusCode;
use http_api_problem::ApiError;
use manas_access_control::model::{KResolvedAccessControl, KResolvedHostAccessControl};
use typed_record::TypedRecord;

use crate::{SgCredentials, SolidStorage};

/// Attach authorization context to api error fields.
pub fn attach_authorization_context<Storage: SolidStorage>(error: &mut ApiError) {
    if error.status() != StatusCode::FORBIDDEN {
        return;
    }

    let mut context = vec![];
    if let Some(r_acl) = error
        .extensions()
        .get_rv::<KResolvedAccessControl<SgCredentials<Storage>>>()
    {
        if !r_acl.is_allowed() {
            context.push(r_acl.clone())
        }
    }

    if let Some(rh_acl) = error
        .extensions()
        .get_rv::<KResolvedHostAccessControl<SgCredentials<Storage>>>()
    {
        if !rh_acl.is_allowed() {
            context.push(rh_acl.clone())
        }
    }

    error.add_field("authorization_context", context);
}

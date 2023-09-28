//! I provide existing resource status token implementation for ODR.
//!

use std::{ops::Deref, sync::Arc};

use manas_http::representation::metadata::RepresentationMetadata;
use manas_repo::service::resource_operator::common::status_token::ExistingResourceToken;

use super::{ODRExistingNonRepresentedResourceToken, ODRExistingRepresentedResourceToken};
use crate::{
    context::ODRContext,
    service::resource_operator::common::status_token::{
        inputs::ODRResourceStatusTokenInputs, ODRResourceStatusTokenTypes,
    },
    setup::ODRSetup,
};

/// A struct to represent existing resource status
/// token for odr.
#[derive(Debug, Clone)]
pub struct ODRBaseExistingResourceToken<Setup: ODRSetup>(
    pub ExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>,
);

impl<Setup: ODRSetup> Deref for ODRBaseExistingResourceToken<Setup> {
    type Target = ODRResourceStatusTokenInputs<Setup>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            ExistingResourceToken::Represented(t) => t,
            ExistingResourceToken::NonRepresented(t) => t,
        }
    }
}

impl<Setup: ODRSetup> From<ODRBaseExistingResourceToken<Setup>>
    for ODRResourceStatusTokenInputs<Setup>
{
    #[inline]
    fn from(value: ODRBaseExistingResourceToken<Setup>) -> Self {
        match value.0 {
            ExistingResourceToken::Represented(t) => t.into(),
            ExistingResourceToken::NonRepresented(t) => t.into(),
        }
    }
}

impl<Setup: ODRSetup> From<ExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>>
    for ODRBaseExistingResourceToken<Setup>
{
    fn from(token: ExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>) -> Self {
        Self(token)
    }
}

impl<Setup: ODRSetup> From<ODRBaseExistingResourceToken<Setup>>
    for ExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>
{
    #[inline]
    fn from(token: ODRBaseExistingResourceToken<Setup>) -> Self {
        token.0
    }
}

impl<Setup: ODRSetup> From<ODRExistingRepresentedResourceToken<Setup>>
    for ODRBaseExistingResourceToken<Setup>
{
    #[inline]
    fn from(value: ODRExistingRepresentedResourceToken<Setup>) -> Self {
        Self(ExistingResourceToken::Represented(value))
    }
}

impl<Setup: ODRSetup> From<ODRExistingNonRepresentedResourceToken<Setup>>
    for ODRBaseExistingResourceToken<Setup>
{
    #[inline]
    fn from(value: ODRExistingNonRepresentedResourceToken<Setup>) -> Self {
        Self(ExistingResourceToken::NonRepresented(value))
    }
}

impl<Setup: ODRSetup> ODRBaseExistingResourceToken<Setup> {
    /// Get the repo context.
    #[inline]
    pub fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        self.res_context.repo_context()
    }

    /// Get resource status inputs.
    #[inline]
    pub fn status_inputs(&self) -> &ODRResourceStatusTokenInputs<Setup> {
        self.deref()
    }

    /// Resolve rep validators.
    #[inline]
    pub fn resolve_rep_validators(&self) -> Option<RepresentationMetadata> {
        match &self.0 {
            ExistingResourceToken::NonRepresented(_) => None,
            ExistingResourceToken::Represented(er_token) => Some(er_token.resolve_rep_validators()),
        }
    }

    /// Get as represented token, if it is one.
    #[inline]
    pub fn as_represented(&self) -> Option<&ODRExistingRepresentedResourceToken<Setup>> {
        match &self.0 {
            ExistingResourceToken::NonRepresented(_) => None,
            ExistingResourceToken::Represented(er_token) => Some(er_token),
        }
    }
}

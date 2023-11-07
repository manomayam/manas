//! I provide resource status token implementations for ODR.
//!

use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use futures::TryFutureExt;
use manas_repo::service::resource_operator::common::status_token::{
    ResourceStatusToken, ResourceStatusTokenTypes,
};
use manas_space::resource::uri::SolidResourceUri;
use tracing::{debug, error};

use self::{
    inputs::ODRResourceStatusTokenInputs,
    variant::{
        ODRBaseExistingResourceToken, ODRBaseNonExistingResourceToken,
        ODRExistingNonRepresentedResourceToken, ODRExistingRepresentedResourceToken,
        ODRNonExistingMutexExistingResourceToken, ODRNonExistingMutexNonExistingResourceToken,
    },
};
use crate::{
    context::ODRContext,
    resource_context::{invariant::ODRClassifiedResourceContext, ODRResourceContext},
    setup::ODRSetup,
    OpendalRepo,
};

pub mod inputs;
pub mod variant;

/// An implementation of [`ResourceStatusTokenTypes`] for odr.
#[derive(Debug, Clone)]
pub struct ODRResourceStatusTokenTypes<Setup> {
    _phantom: PhantomData<Setup>,
}

impl<Setup: ODRSetup> ResourceStatusTokenTypes for ODRResourceStatusTokenTypes<Setup> {
    type Repo = OpendalRepo<Setup>;

    type ExistingNonRepresented = ODRExistingNonRepresentedResourceToken<Setup>;

    type ExistingRepresented = ODRExistingRepresentedResourceToken<Setup>;

    type NonExistingMutexExisting = ODRNonExistingMutexExistingResourceToken<Setup>;

    type NonExistingMutexNonExisting = ODRNonExistingMutexNonExistingResourceToken<Setup>;
}

/// A struct for representing a base version of odr
/// resource status token.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum ODRBaseResourceStatusToken<Setup: ODRSetup> {
    /// Existing
    Existing(ODRBaseExistingResourceToken<Setup>),

    /// Non-existing
    NonExisting(ODRBaseNonExistingResourceToken<Setup>),
}

impl<Setup: ODRSetup> From<ODRResourceStatusTokenInputs<Setup>>
    for ODRBaseResourceStatusToken<Setup>
{
    fn from(inputs: ODRResourceStatusTokenInputs<Setup>) -> Self {
        // If slot path is represented
        if inputs.slot_path_is_represented {
            // If base object exists, then resource is
            // represented.
            if inputs.base_obj_metadata.is_some() {
                debug!(
                    "Resource is represented. Base object metadata: {:?}",
                    inputs.base_obj_metadata
                );
                return Self::Existing(ODRExistingRepresentedResourceToken(inputs).into());
            }

            // Else if resource is aux, then it exists
            // sans-representation.
            if inputs.res_context.slot().is_aux_slot() {
                debug!("Resource is existing, non represented auxiliary.");
                return Self::Existing(ODRExistingNonRepresentedResourceToken(inputs).into());
            }
        }

        debug!("Resource is not existing. Inputs: {:?}", inputs);

        // If none of the above exit cases is satisfied,
        // resource is considered non existing.
        Self::NonExisting(ODRBaseNonExistingResourceToken {
            uri: inputs.res_context.uri().clone(),
            repo_context: inputs.res_context.repo_context().clone(),
            inputs: Some(inputs),
        })
    }
}

impl<Setup: ODRSetup> ODRBaseResourceStatusToken<Setup> {
    /// Try to get current base resource status token for
    /// resource with given context.
    pub async fn try_current(
        res_context: Arc<ODRResourceContext<Setup>>,
    ) -> Result<Self, opendal::Error> {
        Ok(
            // Resolve base inputs
            ODRResourceStatusTokenInputs::try_current(ODRClassifiedResourceContext::new(
                res_context,
            ))
            .await?
            // And then classify.
            .into(),
        )
    }

    /// Try to get current classified resource status token for
    /// resource with given uri.
    pub async fn try_current_for(
        repo_context: Arc<ODRContext<Setup>>,
        res_uri: SolidResourceUri,
    ) -> Result<Self, opendal::Error> {
        // If res slot can be assigned,
        if let Ok(res_context) = ODRResourceContext::try_new(res_uri.clone(), repo_context.clone())
        {
            Self::try_current(Arc::new(res_context)).await
        } else {
            debug!(
                "Cannot assign slot for resource with uri <{}>",
                res_uri.as_str()
            );
            Ok(Self::NonExisting(ODRBaseNonExistingResourceToken {
                uri: res_uri,
                repo_context,
                inputs: None,
            }))
        }
    }

    /// Get if token is of existing resource.
    #[inline]
    pub fn is_of_existing(&self) -> bool {
        matches!(self, Self::Existing(_))
    }
}

#[async_trait]
impl<Setup: ODRSetup> async_convert::TryFrom<ODRBaseResourceStatusToken<Setup>>
    for ResourceStatusToken<ODRResourceStatusTokenTypes<Setup>>
{
    type Error = opendal::Error;

    async fn try_from(base_token: ODRBaseResourceStatusToken<Setup>) -> Result<Self, Self::Error> {
        match base_token {
            // If resource exists.
            ODRBaseResourceStatusToken::Existing(base_e_token) => {
                Ok(ResourceStatusToken::Existing(base_e_token.into()))
            }

            // If resource doesn't exists.
            ODRBaseResourceStatusToken::NonExisting(base_ne_token) => {
                let ne_token = async_convert::TryFrom::try_from(base_ne_token)
                    .inspect_err(|e| {
                        error!(
                            "Unknown io error in resolving mutex resource status. Error:\n {}",
                            e
                        );
                    })
                    .await?;
                Ok(ResourceStatusToken::NonExisting(ne_token))
            }
        }
    }
}

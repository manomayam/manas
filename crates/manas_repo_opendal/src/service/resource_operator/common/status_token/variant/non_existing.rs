//! I provide non-existing resource status token implementation
//! for ODR.
//!

use std::sync::Arc;

use async_trait::async_trait;
use futures::TryFutureExt;
use manas_repo::service::resource_operator::common::status_token::NonExistingResourceToken;
use manas_space::resource::uri::SolidResourceUri;
use tracing::{debug, error};

use super::{
    ODRNonExistingMutexExistingResourceToken, ODRNonExistingMutexNonExistingResourceToken,
};
use crate::{
    context::ODRContext,
    resource_context::ODRResourceContext,
    service::resource_operator::common::status_token::{
        inputs::ODRResourceStatusTokenInputs, ODRBaseResourceStatusToken,
        ODRResourceStatusTokenTypes,
    },
    setup::ODRSetup,
};

/// A struct to represent non-existing resource status
/// token for odr.
#[derive(Debug, Clone)]
pub struct ODRBaseNonExistingResourceToken<Setup: ODRSetup> {
    /// Uri of the resource.
    pub(in super::super) uri: SolidResourceUri,

    /// Repo context.
    pub(in super::super) repo_context: Arc<ODRContext<Setup>>,

    /// Optional inputs.
    /// If `None`, slot couldn't be assigned either.
    pub(in super::super) inputs: Option<ODRResourceStatusTokenInputs<Setup>>,
}

impl<Setup: ODRSetup> From<NonExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>>
    for ODRBaseNonExistingResourceToken<Setup>
{
    fn from(token: NonExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>) -> Self {
        match token {
            NonExistingResourceToken::MutexExisting(t) => Self {
                uri: t.uri,
                repo_context: t.repo_context,
                inputs: t.own_inputs,
            },
            NonExistingResourceToken::MutexNonExisting(t) => Self {
                uri: t.uri,
                repo_context: t.repo_context,
                inputs: t.own_inputs,
            },
        }
    }
}

#[async_trait]
impl<Setup: ODRSetup> async_convert::TryFrom<ODRBaseNonExistingResourceToken<Setup>>
    for NonExistingResourceToken<ODRResourceStatusTokenTypes<Setup>>
{
    type Error = opendal::Error;

    async fn try_from(token: ODRBaseNonExistingResourceToken<Setup>) -> Result<Self, Self::Error> {
        // Resolve slot of the mutex resource.
        let mutex_res_context = token
            .inputs
            .as_ref()
            .and_then(|inputs| {
                inputs
                    .res_context
                    .as_inner()
                    .as_ref()
                    .mutex_resource_context()
            })
            .or_else(|| {
                ODRResourceContext::try_new_mutex(token.uri.clone(), token.repo_context.clone())
            });

        if let Some(context) = mutex_res_context {
            debug!(
                "Resolved mutex resource uri: {}, Base object path: {:?}",
                context.uri(),
                context.assoc_odr_object_map().base_object().id()
            );
            let mutex_token = ODRBaseResourceStatusToken::try_current(Arc::new(context))
                .inspect_err(|_| {
                    error!("Error in resolving base status token for mutex resource of resource. Uri: {}", token.uri.as_str());
                })
                .await?;

            Ok(match mutex_token {
                ODRBaseResourceStatusToken::Existing(t) => {
                    debug!("Mutex resource exists");
                    NonExistingResourceToken::MutexExisting(
                        ODRNonExistingMutexExistingResourceToken {
                            uri: token.uri,
                            repo_context: token.repo_context.clone(),
                            own_inputs: token.inputs,
                            mutex_inputs: t.into(),
                        },
                    )
                }
                ODRBaseResourceStatusToken::NonExisting(t) => {
                    debug!("Mutex resource doesn't exists.");
                    NonExistingResourceToken::MutexNonExisting(
                        ODRNonExistingMutexNonExistingResourceToken {
                            uri: token.uri,
                            repo_context: token.repo_context.clone(),
                            own_inputs: token.inputs,
                            _mutex_inputs: t.inputs,
                        },
                    )
                }
            })
        } else {
            debug!("Slot is not assignable for mutex resource.");
            Ok(NonExistingResourceToken::MutexNonExisting(
                ODRNonExistingMutexNonExistingResourceToken {
                    uri: token.uri,
                    repo_context: token.repo_context.clone(),
                    own_inputs: token.inputs,
                    _mutex_inputs: None,
                },
            ))
        }
    }
}

impl<Setup: ODRSetup> ODRBaseNonExistingResourceToken<Setup> {
    /// Get the repo context.
    #[inline]
    pub fn repo_context(&self) -> &Arc<ODRContext<Setup>> {
        &self.repo_context
    }
}

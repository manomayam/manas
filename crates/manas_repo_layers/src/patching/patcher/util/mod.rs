//! I provide few utility functions over direct patchers.
//!

use dyn_problem::{type_::UNKNOWN_IO_ERROR, ProbResult};
use futures::TryFutureExt;
use manas_repo::{
    service::resource_operator::{
        common::{
            rep_update_action::RepUpdateAction,
            status_token::{ExistingNonRepresentedResourceToken, ExistingResourceToken},
        },
        reader::rep_preferences::{
            range_negotiator::impl_::CompleteRangeNegotiator, ContainerRepresentationPreference,
            RepresentationPreferences,
        },
    },
    Repo, RepoExistingResourceToken, RepoExt,
};
use manas_space::resource::state::SolidResourceState;
use tower::{Service, ServiceExt};
use tracing::{debug, error};

use super::DirectRepPatcher;

/// An extension trait for [`DirectRepPatcher`].
/// Try to resolve effective rep for given current state and
/// update action..
pub async fn try_resolve_effective_rep<R>(
    current_res_state: SolidResourceState<R::StSpace, R::Representation>,
    rep_update_action: RepUpdateAction<R>,
) -> ProbResult<R::Representation>
where
    R: Repo,
    R::RepPatcher: DirectRepPatcher<R::StSpace, R::Representation>,
{
    match rep_update_action {
        RepUpdateAction::SetWith(rep) => {
            debug!("Update action is set_with.");
            Ok(rep)
        }
        RepUpdateAction::PatchWith(mut patcher) => {
            debug!("Update action is patch_with.");
            patcher
                .ready()
                .and_then(|svc| svc.call(current_res_state))
                .await
        }
    }
}

/// An extension trait for [`DirectRepPatcher`].
/// Try to deref current resource state, and resolve effective rep for given current state and
/// update action..
pub async fn try_deref_resolve_effective_rep<R>(
    current_res_token: RepoExistingResourceToken<R>,
    rep_update_action: RepUpdateAction<R>,
) -> ProbResult<(RepoExistingResourceToken<R>, R::Representation)>
where
    R: Repo,
    R::RepPatcher: DirectRepPatcher<R::StSpace, R::Representation>,
{
    let mut rep_patcher = match rep_update_action {
        RepUpdateAction::SetWith(rep) => {
            debug!("Update action is set_with.");
            return Ok((current_res_token, rep));
        }
        RepUpdateAction::PatchWith(patcher) => {
            debug!("Update action is patch_with.");
            patcher
        }
    };

    let (res_token, res_state) = match current_res_token {
        ExistingResourceToken::NonRepresented(enr_token) => {
            let slot = enr_token.slot().clone();
            (
                ExistingResourceToken::NonRepresented(enr_token),
                SolidResourceState {
                    slot,
                    representation: None,
                },
            )
        }
        ExistingResourceToken::Represented(er_token) => {
            // Query resource state.
            let resp = R::read_basic_with_token(
                er_token,
                Default::default(),
                RepresentationPreferences {
                    // TODO must decide clearly.
                    container_rep_preference: ContainerRepresentationPreference::Minimal,
                    non_container_rep_range_negotiator: Box::new(CompleteRangeNegotiator),
                },
            )
            .map_err(|e| {
                error!("Error in querying resource state. {e}");
                UNKNOWN_IO_ERROR
                    .new_problem_builder()
                    .message("Error in querying resource state")
                    .source(e)
                    .finish()
            })
            .await?;

            (
                ExistingResourceToken::Represented(resp.tokens.res_token),
                resp.state.into_inner(),
            )
        }
    };

    // Apply patch.
    let effective_rep = rep_patcher
        .ready()
        .and_then(|svc| svc.call(res_state))
        .await?;

    Ok((res_token, effective_rep))
}

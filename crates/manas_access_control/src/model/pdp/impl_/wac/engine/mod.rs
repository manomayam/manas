//! I define wac authorization engine.
//!

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use acp::{
    attribute_match_svc::{AttributeMatchRequest, BoxedAttributeMatchService},
    engine::INTERNAL_MATCHER_ERROR,
    model::{
        access_mode::HAccessMode, acr::DAccessControlResource, context::DContext,
        resource::HResource,
    },
};
use dyn_problem::{ProbResult, Problem};
use futures::{stream::FuturesUnordered, TryFutureExt};
use http_uri::invariant::NormalAbsoluteHttpUri;
use rdf_utils::{
    define_handle_and_description_types,
    model::{
        description::{Description, DescriptionExt, SimpleDescription},
        graph::InfallibleGraph,
        handle::{HAny, Handle},
        term::ArcTerm,
    },
};
use rdf_vocabularies::ns;
use sophia_api::{
    term::{matcher::Any, Term},
    triple::Triple,
};
use tower::ServiceExt;
use tracing::{debug, error, info};
use unwrap_infallible::UnwrapInfallible;

use self::attribute_match_svc::{AgentClassMatchService, AgentMatchService, OriginMatchService};
use crate::model::AccessGrantSet;

pub mod attribute_match_svc;

/// A struct to represent resolved acl context.
#[derive(Debug, Clone)]
pub struct ResolvedAclContext<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    /// Access target uri.
    pub target_uri: NormalAbsoluteHttpUri,

    /// Uri of resolved acl's subject resource.
    pub resolved_acl_subject_uri: NormalAbsoluteHttpUri,

    /// Resolved acl
    pub resolved_acl: DAccessControlResource<G, WG>,
}

/// [`WacEngine`] resolves permissions to access controlled resources
/// in conformance with WAC access control resolution algorithm
pub struct WacEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    /// Attribute match services.
    subject_attribute_match_svcs: Arc<HashMap<ArcTerm, BoxedAttributeMatchService<ArcTerm, G, WG>>>,
}

impl<G, WG> Clone for WacEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            subject_attribute_match_svcs: self.subject_attribute_match_svcs.clone(),
        }
    }
}

impl<G, WG> Debug for WacEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WacEngine").finish()
    }
}

impl<G> Default for WacEngine<G, Arc<G>>
where
    G: InfallibleGraph + Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new(Arc::new(
            [
                (
                    ns::acl::agent,
                    Box::new(AgentMatchService) as BoxedAttributeMatchService<ArcTerm, G, Arc<G>>,
                ),
                (ns::acl::agentClass, Box::new(AgentClassMatchService)),
                (ns::acl::origin, Box::new(OriginMatchService)),
                // TODO
                // (ns::acl::agentGroup, Box::new(AgentGroupMatchService)),
            ]
            .into_iter()
            .map(|(a, s)| (a.into_term(), s))
            .collect(),
        ))
    }
}

impl<G, WG> WacEngine<G, WG>
where
    G: InfallibleGraph + Clone,
    WG: Borrow<G> + Clone + Debug + Send + Sync,
{
    /// Get a new [`WacEngine] with given subject attribute match services.
    #[inline]
    pub fn new(
        subject_attribute_match_svcs: Arc<
            HashMap<ArcTerm, BoxedAttributeMatchService<ArcTerm, G, WG>>,
        >,
    ) -> Self {
        Self {
            subject_attribute_match_svcs,
        }
    }

    /// Get the list of supported subject attributes.
    #[inline]
    pub fn supported_attrs(&self) -> impl Iterator<Item = ArcTerm> + '_ {
        self.subject_attribute_match_svcs.keys().cloned()
    }

    /// Resolve [access control](https://solid.github.io/web-access-control-spec/#authorization-evaluation)
    /// for given access context with given acls
    ///
    /// @see <https://github.com/solid/web-access-control-spec/issues/63#issuecomment-533507081> for terminologies used.
    ///
    pub async fn resolve_access_control(
        &self,
        resolved_acl_context: Option<ResolvedAclContext<G, WG>>,
        access_context: DContext<G, WG>,
    ) -> Result<AccessGrantSet, Problem> {
        // If no resolved acl context, then return empty set.
        let resolved_acl_context = if let Some(context) = resolved_acl_context {
            context
        } else {
            return Ok(Default::default());
        };

        // Gather applicable authorizations.
        let authorizations = self.gather_applicable_authorizations(resolved_acl_context);

        let allowed_access_modes = self
            .grant_access_modes(authorizations, access_context)
            .inspect_err(|e| error!("Error in resolving granted access modes. Error:\n {}", e))
            .await?;

        Ok(allowed_access_modes)
    }

    /// Gather [applicable authorizations](https://solid.github.io/web-access-control-spec/#authorization-conformance).
    ///
    /// > An applicable Authorization has the following properties:
    /// > - At least one rdf:type property whose object is acl:Authorization.
    /// > - At least one acl:accessTo or acl:default property value (Access Objects).
    ///
    /// -- remaining checks will be done later, as they can be
    /// generalized.
    fn gather_applicable_authorizations(
        &self,
        resolved_acl_context: ResolvedAclContext<G, WG>,
    ) -> Vec<DAuthorization<G, WG>> {
        resolved_acl_context
            .resolved_acl
            .graph()
            .triples_matching(Any, [ns::rdf::type_], [ns::acl::Authorization])
            .filter_map(|t| {
                HAuthorization::try_new(t.unwrap_infallible().s().into_term::<ArcTerm>()).ok()
            })
            .filter_map(|h| {
                let authorization =
                    DAuthorization::new(h, resolved_acl_context.resolved_acl.wgraph().clone());

                // Return only authorization with proper accessTo/default attribute.
                let access_object_predicate = if resolved_acl_context.target_uri
                    == resolved_acl_context.resolved_acl_subject_uri
                {
                    // Own acl, hence accessTo.
                    &ns::acl::accessTo
                } else {
                    // fallback acl, hence default.
                    &ns::acl::default
                };

                // Only subject resource of resolved acl is allowed to be attribute value!
                // See <https://github.com/solid/specification/issues/55>
                // See https://github.com/solid/authorization-panel/issues/191
                authorization
                    .has_any_with(
                        access_object_predicate,
                        &*resolved_acl_context.resolved_acl_subject_uri,
                    )
                    .then_some(authorization)
            })
            .collect()
    }

    async fn grant_access_modes(
        &self,
        authorizations: Vec<DAuthorization<G, WG>>,
        access_context: DContext<G, WG>,
    ) -> ProbResult<AccessGrantSet> {
        let mut allowed_access_modes = HashSet::new();

        // Gather allowed access modes from satisfied authorizations
        for authorization in authorizations.into_iter() {
            if self
                .is_matched_authorization(authorization.clone(), access_context.clone())
                .await?
            {
                allowed_access_modes.extend(authorization.h_mode());
            }
        }

        Ok(allowed_access_modes)
    }

    /// Resolves if an authorization is matched against access
    /// context..
    ///
    /// > An Authorization MUST be matched if and only if:
    ///
    /// > - it defines at least one access-subject attribute; and,
    /// > - at least one value of each defined attribute matches the Context.
    ///
    async fn is_matched_authorization(
        &self,
        authorization: DAuthorization<G, WG>,
        access_context: DContext<G, WG>,
    ) -> ProbResult<bool> {
        let attribute_match_futs = self
            .subject_attribute_match_svcs
            .iter()
            .map(|(attribute, svc)| {
                (
                    // attribute
                    attribute.clone(),
                    // Value matching futures.
                    authorization
                        .get_all(attribute)
                        .into_term_owning::<ArcTerm>()
                        .map(|value| {
                            svc.clone().oneshot(AttributeMatchRequest {
                                value,
                                context: access_context.clone(),
                            })
                        })
                        .collect::<FuturesUnordered<_>>(),
                )
            })
            .filter(|(_, futs)| !futs.is_empty())
            .collect::<HashMap<_, _>>();

        // An empty authorization is never satisfied.
        if attribute_match_futs.is_empty() {
            info!("Authorization doesn't  define any known attribute.");
            return Ok(false);
        }

        for (attribute, futs) in attribute_match_futs.into_iter() {
            let mut is_match = false;
            let mut last_error = None;

            for fut in futs {
                match fut.await {
                    Ok(true) => {
                        debug!("{:?} attribute matched", attribute);
                        is_match = true;
                        break;
                    }
                    Err(e) => {
                        info!(
                            "Unknown error in resolving attribute value match. Error:\n {}",
                            e
                        );
                        last_error = Some(e);
                    }
                    _ => {}
                }
            }

            if !is_match {
                if let Some(e) = last_error {
                    error!("Error in resolving attribute match.");
                    return Err(INTERNAL_MATCHER_ERROR
                        .new_problem_builder()
                        .source(e)
                        .finish());
                }

                info!(
                    "None of attribute values matched against context. attribute: {:?}",
                    attribute
                );
                return Ok(false);
            }
        }

        // At this point, the authorization is matched because
        // - there was at least one defined attribute about access subject, and
        // - at least one value of each defined attribute matched the context.
        Ok(true)
    }
}

define_handle_and_description_types!(
    /// Handle for acl:Authorization resources.
    HAuthorization;
    /// Description of acl:Authorization resources
    DAuthorization;
    [
        /// Type of the resource.
        (type_, &ns::rdf::type_, HAny);

        /// The acl:accessTo predicate denotes the resource to
        /// which access is being granted.
        (access_to, &ns::acl::accessTo, HResource);

        /// The acl:default predicate denotes the container
        /// resource whose Authorization can be applied to a
        /// resource lower in the collection hierarchy.
        (default, &ns::acl::default, HResource);

        /// The acl:mode predicate denotes a class of operations
        /// that the agents can perform on a resource.
        (mode, &ns::acl::mode, HAccessMode);
    ]
);

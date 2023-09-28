//! I define an implementation of acp engine.
//!

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use dyn_problem::{define_anon_problem_types, Problem};
use futures::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use rdf_utils::model::{description::DescriptionExt, graph::InfallibleGraph, term::ArcTerm};
use rdf_vocabularies::ns;
use sophia_api::term::Term;
use tower::ServiceExt;
use tracing::{debug, error, log::info};

use super::attribute_match_svc::{AttributeMatchRequest, BoxedAttributeMatchService};
use crate::{
    attribute_match_svc::impl_::{
        AgentMatchService, ClientMatchService, IssuerMatchService, VcMatchService,
    },
    model::{
        access_mode::HAccessMode, acr::DAccessControlResource, context::DContext,
        matcher::DMatcher, policy::DPolicy,
    },
};

// fn sy<T: Send>(v: T) -> T {
//     v
// }

/// Alias for type of set of access modes.
pub type AccessGrantSet = HashSet<HAccessMode<ArcTerm>>;

/// [`AcpEngine`] resolves permissions to access controlled resources
/// in conformance with ACP access control resolution algorithm
pub struct AcpEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    /// Attribute match services.
    attribute_match_svcs: Arc<HashMap<ArcTerm, BoxedAttributeMatchService<ArcTerm, G, WG>>>,
}

impl<G, WG> Clone for AcpEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            attribute_match_svcs: self.attribute_match_svcs.clone(),
        }
    }
}

impl<G, WG> Debug for AcpEngine<G, WG>
where
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpEngine").finish()
    }
}

impl<G> Default for AcpEngine<G, Arc<G>>
where
    G: InfallibleGraph + Clone + Send + Sync + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new(Arc::new(
            [
                (
                    ns::acp::agent,
                    Box::new(AgentMatchService) as BoxedAttributeMatchService<ArcTerm, G, Arc<G>>,
                ),
                (ns::acp::client, Box::new(ClientMatchService)),
                (ns::acp::issuer, Box::new(IssuerMatchService)),
                (ns::acp::vc, Box::new(VcMatchService)),
            ]
            .into_iter()
            .map(|(a, s)| (a.into_term(), s))
            .collect(),
        ))
    }
}

// TODO should refactor logic into respective models.

impl<G, WG> AcpEngine<G, WG>
where
    G: InfallibleGraph + Clone,
    WG: Borrow<G> + Clone + Debug + Send + Sync,
{
    /// Get a new [`AcpEngine] with given attribute match services.
    #[inline]
    pub fn new(
        attribute_match_svcs: Arc<HashMap<ArcTerm, BoxedAttributeMatchService<ArcTerm, G, WG>>>,
    ) -> Self {
        Self {
            attribute_match_svcs,
        }
    }

    /// Get the list of supported context attributes.
    #[inline]
    pub fn supported_attrs(&self) -> impl Iterator<Item = ArcTerm> + '_ {
        self.attribute_match_svcs.keys().cloned()
    }

    /// Resolve [access control](https://solid.github.io/authorization-panel/acp-specification/#resolved-access-control)
    ///  for given context against given acrs.
    /// Returns grant graph on successful resolution.
    ///
    /// >  An ACP engine MUST grant exactly those Access Modes allowed by Effective Policies.
    #[tracing::instrument(skip_all)]
    pub async fn resolve_access_control(
        &self,
        opt_acr: Option<DAccessControlResource<G, WG>>,
        ancestor_acrs: Vec<Option<DAccessControlResource<G, WG>>>,
        context: DContext<G, WG>,
    ) -> Result<AccessGrantSet, Problem> {
        // Gather effective policies.
        let effective_policies = self.gather_effective_policies(opt_acr, ancestor_acrs);

        // Resolve allowed access modes.
        let allowed_access_modes = self
            .grant_access_modes(effective_policies, context)
            .inspect_err(|e| error!("Error in resolving granted access modes. Error:\n {}", e))
            .await?;

        Ok(allowed_access_modes)
    }

    /// Gather [effective policies](https://solid.github.io/authorization-panel/acp-specification/#effective-policies).
    ///
    /// > Effective Policies are the Policies controlling access to a resource.
    /// > A Policy MUST control access to a resource if:
    ///
    /// > - it is applied by an Access Control of an ACR of the resource; or,
    /// > - it is applied by a member Access Control of an ACR of an ancestor of the resource.
    fn gather_effective_policies(
        &self,
        opt_acr: Option<DAccessControlResource<G, WG>>,
        ancestor_acrs: Vec<Option<DAccessControlResource<G, WG>>>,
    ) -> Vec<DPolicy<G, WG>> {
        // Direct access controls of own ACR.
        let own_acls = if let Some(acr) = &opt_acr {
            Box::new(acr.access_control()) as Box<dyn Iterator<Item = _>>
        } else {
            Box::new(std::iter::empty())
        };

        // Inherited member access controls fro ancestor ACRs.
        let inh_acls = ancestor_acrs
            .iter()
            .filter_map(|opt_acr| opt_acr.as_ref())
            .flat_map(|ancestor_acr| ancestor_acr.member_access_control());

        own_acls
            .chain(inh_acls)
            .fold(Vec::new(), |mut effective_policies, acl| {
                effective_policies.extend(acl.apply());
                effective_policies
            })
    }

    /// Resolves granted access modes.
    /// >    An Access Mode MUST be granted if and only if in the set of Effective Policies:
    /// > - a satisfied policy allows the Access Mode; and,
    /// > - no satisfied policy denies it.
    async fn grant_access_modes(
        &self,
        policies: Vec<DPolicy<G, WG>>,
        context: DContext<G, WG>,
    ) -> Result<AccessGrantSet, Problem> {
        let mut allowed_access_modes = HashSet::new();
        let mut denied_access_modes = HashSet::new();

        // Gather allowed and denied access modes from satisfied policies
        for policy in policies.into_iter() {
            if self
                .is_satisfied_policy(policy.clone(), context.clone())
                .await?
            {
                allowed_access_modes.extend(policy.h_allow());
                denied_access_modes.extend(policy.h_deny());
            }
        }

        // Deny overrules allow.
        denied_access_modes.iter().for_each(|mode| {
            allowed_access_modes.remove(mode);
        });

        Ok(allowed_access_modes)
    }

    /// Resolves if policy is satisfied.
    /// > A Policy MUST be satisfied if and only if:
    /// > - it references at least one Matcher via an acp:allOf or acp:anyOf property; and,
    /// > - all of its acp:allOf Matchers are satisfied; and,
    /// > - at least one of its acp:anyOf Matchers is satisfied; and,
    /// > - none of its acp:noneOf Matchers are satisfied.
    async fn is_satisfied_policy(
        &self,
        policy: DPolicy<G, WG>,
        context: DContext<G, WG>,
    ) -> Result<bool, Problem> {
        // If any 'none of' matcher is satisfied then the policy is not satisfied.
        // For now evaluate all matchers without short circuit,
        // to simplify handling of fallible evaluation.
        let none_of = policy.none_of().collect::<Vec<_>>();
        if none_of
            .into_iter()
            .map(|matcher| self.is_satisfied_matcher(matcher, context.clone()))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?
            .contains(&true)
        {
            debug!("Policy has a noneOf matcher that is satisfied.");
            return Ok(false);
        }

        // If any 'all of' matcher is not satisfied then the policy is not satisfied.
        let all_of = policy.all_of().collect::<Vec<_>>();
        if all_of
            .into_iter()
            .map(|matcher| self.is_satisfied_matcher(matcher, context.clone()))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?
            .contains(&false)
        {
            debug!("Policy has an allOf matcher that is not satisfied.");
            return Ok(false);
        }

        // If any 'any of' matcher is satisfied then the policy is satisfied.
        let any_of = policy.any_of().collect::<Vec<_>>();
        if any_of
            .into_iter()
            .map(|matcher| self.is_satisfied_matcher(matcher, context.clone()))
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            // Conservative on evaluation error, even though  few cases can be resolved.
            .await?
            .contains(&true)
        {
            debug!("Policy has an anyOf matcher that is satisfied.");
            return Ok(true);
        }

        // At this point there are
        // - no satisfied 'none of' matchers,
        // - no unsatisfied 'all of' matchers and
        // - no satisfied 'any of' matchers.

        // Hence, the policy is satisfied if it has
        // - an 'all of' condition and
        // - no 'any of' condition.
        Ok(policy.has_any(&ns::acp::allOf) && !policy.has_any(&ns::acp::anyOf))
    }

    /// Resolves if a matcher is satisfied against given context.
    ///
    /// > A Matcher MUST be satisfied if and only if:
    ///
    /// > - it defines at least one attribute; and,
    /// > - at least one value of each defined attribute matches the Context.
    ///
    /// > ACP engines MUST match the context attributes defined
    /// by this specification according to IRI equality and
    /// literal term equality.
    async fn is_satisfied_matcher(
        &self,
        matcher: DMatcher<G, WG>,
        context: DContext<G, WG>,
    ) -> Result<bool, Problem> {
        let attribute_match_futs = self
            .attribute_match_svcs
            .iter()
            .map(|(attribute, svc)| {
                (
                    // attribute
                    attribute.clone(),
                    // Value matching futures.
                    matcher
                        .get_all(attribute)
                        .into_term_owning::<ArcTerm>()
                        .map(|value| {
                            svc.clone().oneshot(AttributeMatchRequest {
                                value,
                                context: context.clone(),
                            })
                        })
                        .collect::<FuturesUnordered<_>>(),
                )
            })
            .filter(|(_, futs)| !futs.is_empty())
            .collect::<HashMap<_, _>>();

        // An empty matcher is never satisfied.
        if attribute_match_futs.is_empty() {
            info!("Matcher doesn't  define any known attribute.");
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

        // At this point, the matcher is satisfied because
        // - there was at least one defined attribute and
        // - at least one value of each defined attribute matched the context.
        Ok(true)
    }
}

define_anon_problem_types!(
    /// Internal matvher error.
    INTERNAL_MATCHER_ERROR: ("Internal matcher error.");
);

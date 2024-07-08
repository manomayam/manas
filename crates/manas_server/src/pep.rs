//! I define concrete types for the policy enforcement points for recipes.
//!

use std::{collections::HashSet, sync::Arc};

use manas_access_control::model::{
    pdp::PolicyDecisionPoint,
    pep::impl_::{
        solid_compat::SimplePolicyEnforcementPoint, trivial::TrivialPolicyEnforcementPoint,
    },
};
use manas_authentication::common::credentials::impl_::basic::BasicRequestCredentials;
use manas_http::{
    header::common::media_type::TEXT_TURTLE,
    representation::{
        impl_::{
            basic::BasicRepresentation, binary::BinaryRepresentation,
            common::data::bytes_inmem::BytesInmem,
        },
        metadata::{KContentType, RepresentationMetadata},
    },
    uri::invariant::HierarchicalTrailingSlashHttpUri,
};
use manas_repo_opendal::{
    object_store::backend::ODRObjectStoreBackend, service::prp::ODRPolicyRetrievalPoint,
};
use manas_space::resource::uri::SolidResourceUri;
use rdf_utils::model::triple::ArcTriple;
use serde::Serialize;
use upon::Engine;
use webid::WebId;

use crate::{repo::RcpBaseRepoSetup, space::RcpStorageSpace, storage::RcpStorageSetup};

/// Type of access control policy retrieval point for the recipes.
pub type RcpPRP<Backend> = ODRPolicyRetrievalPoint<RcpBaseRepoSetup<Backend>, HashSet<ArcTriple>>;

/// Type of trivial policy enforcement point for the recipe.
pub type RcpTrivialPEP = TrivialPolicyEnforcementPoint<RcpStorageSpace, BasicRequestCredentials>;

/// Type of simple pep for the recipe.
pub type RcpSimplePEP<Backend, PDP> =
    SimplePolicyEnforcementPoint<RcpStorageSpace, RcpPRP<Backend>, PDP>;

/// An alias trait for recipe storage setup with [`RcpSimplePEP`] as pep.
pub trait SimpleAccessRcpStorageSetup:
    RcpStorageSetup<PEP = RcpSimplePEP<Self::Backend_, Self::PDP>, Backend = Self::Backend_>
{
    /// Same as Self::Backend, but to avoid rrustc resolution cycle.
    type Backend_: ODRObjectStoreBackend;

    /// Type of the pdp.
    type PDP: PolicyDecisionPoint<StSpace = RcpStorageSpace, Graph = HashSet<ArcTriple>>;
}

impl<
        Backend: ODRObjectStoreBackend,
        PDP: PolicyDecisionPoint<StSpace = RcpStorageSpace, Graph = HashSet<ArcTriple>>,
        S: RcpStorageSetup<PEP = RcpSimplePEP<Backend, PDP>, Backend = Backend>,
    > SimpleAccessRcpStorageSetup for S
{
    type Backend_ = Backend;
    type PDP = PDP;
}

/// Wac initial root acr template str.
pub const WAC_INITIAL_ROOT_ACR_TEMPLATE_STR: &str =
    include_str!("../templates/wac_initial_root_acr.ttl.template");

/// Acp initial root acr template str.
pub const ACP_INITIAL_ROOT_ACR_TEMPLATE_STR: &str =
    include_str!("../templates/acp_initial_root_acr.ttl.template");

/// Context for initial root acr template.
#[derive(Debug, Serialize)]
pub struct InitialRootAcrTemplateContext {
    /// Storage root resource uri.
    pub storage_root_res_uri: HierarchicalTrailingSlashHttpUri,

    /// Owner id.
    pub owner_id: WebId,
}

/// Resolve initial root acr context.
pub fn resolve_initial_root_acr_content(
    template: &str,
    context: &InitialRootAcrTemplateContext,
) -> String {
    let mut engine = Engine::new();
    engine
        .add_template("initial_root_acr", template)
        .expect("Template must be valid.");

    engine
        .get_template("initial_root_acr")
        .expect("Template must exist")
        .render(context)
        .to_string()
        .expect("Must be valid")
}

/// Type alias for type of initial root acr rep factory.
pub type InitialRootAcrRepFactory =
    Arc<dyn Fn(SolidResourceUri) -> Option<BinaryRepresentation> + Send + Sync + 'static>;

/// Resolve initial root acr rep factory.
pub fn resolve_initial_root_acr_rep_factory(
    template: &str,
    context: &InitialRootAcrTemplateContext,
) -> InitialRootAcrRepFactory {
    let rep = BasicRepresentation {
        metadata: RepresentationMetadata::new().with::<KContentType>((TEXT_TURTLE).clone()),
        data: BytesInmem::from(resolve_initial_root_acr_content(template, context)),
        base_uri: None,
    };

    Arc::new(move |acr_uri| {
        let mut rep = rep.clone();
        rep.base_uri = Some(acr_uri.into_subject());

        Some(rep.into_binary())
    })
}

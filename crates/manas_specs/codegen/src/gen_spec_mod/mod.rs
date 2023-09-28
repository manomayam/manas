//! I define utils to generate a spec mod.
//!

use std::collections::{HashMap, HashSet};

use anyhow::Context;
use rdf_utils::model::term::ArcIriRef;
use rdf_vocabularies::ns;
use serde::Serialize;
use sophia_api::{graph::Graph, term::Term};

use crate::{
    templates::{SPEC_MOD_TEMPLATE_ID, TEMPLATE_REGISTRY},
    util::{graph::GraphExt, ident::sanitize_ident},
};

///A struct to hold codegen metadata about a requirement.
#[derive(Debug, Serialize)]
pub struct RequirementCodegenMeta {
    /// Id of the requirement.
    pub id: String,

    /// Rust id of the requirement.
    pub rid: String,

    /// Rust ids of the requirement's subjects.
    pub subject_rids: Vec<String>,

    /// Rust id of the requirement's level.
    pub level_rid: String,

    /// Requirement's statement.
    pub statement: String,
}

/// Codegen metadata for requirement subject.
#[derive(Debug, Serialize)]
pub struct ReqSubjectCodegenMeta {
    /// Id of the requirement subject.
    pub id: String,

    /// Rust identifier of the requirement subject.
    pub rid: String,
}

/// Codegen meta for a spec.
#[derive(Debug, Serialize)]
pub struct SpecCodegenMeta {
    /// Id of the spec.
    pub id: String,

    /// Rust id of the spec.
    pub rid: String,

    /// Title of the spec.
    pub title: String,

    /// Description of the spec.
    pub description: Option<String>,

    /// Subject's codegen metadata.
    pub subjects: Vec<ReqSubjectCodegenMeta>,

    /// Requirements's codegen metadata.
    pub requirements: Vec<RequirementCodegenMeta>,
}

/// Struct for codegen configuration.
pub struct SpecCodegenConfig {
    /// Spec rust identifier.
    pub spec_rid: String,

    /// Prefix for requirement rust identifiers.
    pub req_rid_prefix: String,

    /// Map from subject identifiers to their rust identifiers.
    pub req_sub_rid_map: HashMap<String, String>,
}

impl SpecCodegenConfig {
    /// Create a new default [`SpecCodegenConfig`] with given spec rust id.
    pub fn new_default(spec_rid: String) -> Self {
        Self {
            spec_rid,
            req_rid_prefix: "REQ_".into(),
            req_sub_rid_map: [
                (
                    ns::spec::Server.iri().unwrap().to_string(),
                    "SUBJECT_SERVER".to_owned(),
                ),
                (
                    ns::spec::Client.iri().unwrap().to_string(),
                    "SUBJECT_CLIENT".to_owned(),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }
}

/// A struct for representing a specification graph.
pub struct SpecGraph<G: Graph> {
    /// Id of the spec.
    pub id: ArcIriRef,

    /// Title of the spec.
    pub title: String,

    /// Graph of the spec.
    pub graph: G,

    /// Namespace base of the terms in spec.
    pub ns_base: ArcIriRef,
}

impl<G: Graph> SpecGraph<G> {
    /// Generate spec mod.
    pub fn gen_spec_mod(&self, config: &SpecCodegenConfig) -> Result<String, anyhow::Error> {
        let spec_codegen_meta = self
            .codegen_meta(config)
            .with_context(|| "Error in computing codegen meta.")?;

        TEMPLATE_REGISTRY
            .render(SPEC_MOD_TEMPLATE_ID, &spec_codegen_meta)
            .with_context(|| "Error in generating module from template")
    }

    /// Resolve codegen meta for the spec.
    pub fn codegen_meta(
        &self,
        config: &SpecCodegenConfig,
    ) -> Result<SpecCodegenMeta, anyhow::Error> {
        Ok(SpecCodegenMeta {
            id: self.id.as_str().to_owned(),
            rid: config.spec_rid.clone(),
            title: self.title.clone(),
            description: Some(self.title.clone()),
            subjects: config
                .req_sub_rid_map
                .iter()
                .map(|(id, rid)| ReqSubjectCodegenMeta {
                    id: id.clone(),
                    rid: rid.clone(),
                })
                .collect(),
            requirements: self
                .requirement_ids()
                .iter()
                .filter_map(|req_id| {
                    let req_meta = self.requirement_codegen_meta(req_id, config);
                    // println!("{:?}: {:?}", req_id, req_meta);
                    req_meta.ok()
                })
                .collect(),
        })
    }

    /// Get requirement ids.
    pub fn requirement_ids(&self) -> HashSet<ArcIriRef> {
        self.graph
            .iri_objects_of_statement_with_predicate(&ns::spec::requirement)
    }

    /// Get requirement meta
    pub fn requirement_codegen_meta(
        &self,
        req_id: &ArcIriRef,
        config: &SpecCodegenConfig,
    ) -> Result<RequirementCodegenMeta, anyhow::Error> {
        let req_subjects = self
            .graph
            .iri_objects_of_statements_with(req_id, &ns::spec::requirementSubject);

        let req_level = self
            .graph
            .iri_object_of_functional_statement_with(req_id, &ns::spec::requirementLevel)
            .ok_or_else(|| anyhow::anyhow!("No valid level"))?;

        let req_statement = self
            .graph
            .literal_object_of_functional_statement_with(req_id, &ns::spec::statement)
            .ok_or_else(|| anyhow::anyhow!("No valid statement"))?;

        Ok(RequirementCodegenMeta {
            id: req_id.as_str().to_owned(),
            rid: self
                .req_rid(req_id, config)
                .ok_or_else(|| anyhow::anyhow!("invalid requirement id"))?,
            subject_rids: req_subjects
                .iter()
                .filter_map(|sub| self.req_sub_rid(sub, config))
                .collect(),
            level_rid: self
                .req_level_rid(&req_level, config)
                .ok_or_else(|| anyhow::anyhow!("invalid requirement level"))?,
            statement: req_statement,
        })
    }

    /// Compute requirement's rust identifier.
    fn req_rid(&self, req_id: &ArcIriRef, config: &SpecCodegenConfig) -> Option<String> {
        let req_namespace_id: Option<&str> = if req_id.as_str().starts_with(self.ns_base.as_str()) {
            // Todo proper way.
            Some(&req_id.as_str()[self.ns_base.as_str().len()..])
        } else {
            None
        };
        req_namespace_id.map(|ns_id| {
            format!(
                "{}{}",
                config.req_rid_prefix,
                sanitize_ident(ns_id).to_uppercase()
            )
        })
    }

    /// Compute requirement subject's rust identifier.
    fn req_sub_rid(&self, req_sub_id: &ArcIriRef, config: &SpecCodegenConfig) -> Option<String> {
        let req_sub_id_str = req_sub_id.as_str();
        config.req_sub_rid_map.get(req_sub_id_str).cloned()
    }

    /// Compute requirement subject's rust identifier.
    fn req_level_rid(
        &self,
        req_level_id: &ArcIriRef,
        _config: &SpecCodegenConfig,
    ) -> Option<String> {
        [
            (ns::spec::MUST, "Must"),
            (ns::spec::MUSTNOT, "MustNot"),
            (ns::spec::SHOULD, "Should"),
            (ns::spec::SHOULDNOT, "ShouldNot"),
            (ns::spec::SHALL, "Shall"),
            (ns::spec::SHALLNOT, "ShallNot"),
            (ns::spec::REQUIRED, "Required"),
            (ns::spec::RECOMMENDED, "Recommended"),
            (ns::spec::MAY, "May"),
            (ns::spec::OPTIONAL, "Optional"),
        ]
        .iter()
        .find_map(|(level, rid_suffix)| {
            if Term::eq(level, req_level_id) {
                Some(format!("RequirementLevel::{}", rid_suffix))
            } else {
                None
            }
        })
    }
}

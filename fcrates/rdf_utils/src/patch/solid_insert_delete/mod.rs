//! I define types for implementing n3 encoded solid insert-delete patch.
//!
#![allow(unused_qualifications)]

#[allow(clippy::all)]
mod parser;

use std::{collections::HashSet, io::BufRead, sync::Arc};

use itertools::Itertools;
use mime::Mime;
use once_cell::sync::Lazy;
use rdf_vocabularies::ns;
use sophia_api::{
    dataset::{CollectibleDataset, Dataset, SetDataset},
    graph::{CollectibleGraph, Graph},
    ns::NsTerm,
    quad::Quad,
    source::{QuadSource, TripleSource},
    term::{matcher::Any, BnodeId, Term, TermKind},
};
use sophia_rio::parser::GeneralizedRioSource;
use tracing::{debug, error, info};
use unwrap_infallible::UnwrapInfallible;

use self::parser::N3SimpleParser;
use super::PatchEffectiveOperation;
use crate::{
    model::{
        dataset::{InfallibleDataset, InfallibleMutableDataset},
        graph::InfallibleGraph,
        term::{ArcIri, ArcTerm},
    },
    query::{BindingMap, Query},
};

/// `text/n3` media-type.
pub static TEXT_N3: Lazy<Mime> = Lazy::new(|| "text/n3".parse().expect("Must be valid"));

/// A struct representing valid solid insert-delete patch document.
#[derive(Debug)]
pub struct SolidInsertDeletePatchDoc<D>
where
    D: InfallibleDataset + 'static,
{
    // Patch document.
    patch_doc: Arc<D>,

    // Resolved patch.
    patch: SolidInsertDeletePatch<D>,
}

impl<D> Clone for SolidInsertDeletePatchDoc<D>
where
    D: InfallibleDataset,
{
    fn clone(&self) -> Self {
        Self {
            patch_doc: self.patch_doc.clone(),
            patch: self.patch.clone(),
        }
    }
}

impl<D> SolidInsertDeletePatchDoc<D>
where
    D: InfallibleDataset,
{
    /// Parse [`SolidInsertDeletePatchDoc`] from given reader.
    pub fn parse<B: BufRead>(
        reader: B,
        base_uri: Option<ArcIri>,
    ) -> Result<Self, InvalidSolidInsertDeletePatchDoc>
    where
        D: CollectibleDataset,
    {
        // Get parser.
        let parser = N3SimpleParser::new(
            reader,
            base_uri
                .map(|uri| oxiri::Iri::parse(uri.as_str().to_owned()).expect("Must be valid iri.")),
        );

        // Get sophia quad source.
        let qs = GeneralizedRioSource::<N3SimpleParser<B>>(parser);

        // Get patch doc dataset.
        let patch_doc = QuadSource::collect_quads::<D>(qs).map_err(|e| {
            error!("Error in parsing n3 patch document. Error:\n {}", e);
            InvalidSolidInsertDeletePatchDoc::InvalidN3Doc
        })?;

        Self::try_new(Arc::new(patch_doc))
    }

    /// Try to create new [`SolidInsertDeletePatchDoc`] from
    /// given patch document.
    #[tracing::instrument(name = "try_new_solid_inserts_deletes_patch_doc", skip_all)]
    pub fn try_new(patch_doc: Arc<D>) -> Result<Self, InvalidSolidInsertDeletePatchDoc> {
        // Get ids of patch resources.
        // Req: "A patch resource MUST contain a triple ?
        // patch rdf:type solid:InsertDeletePatch."
        // Req: The patch document MUST contain exactly one
        // patch resource,
        let patch_id: ArcTerm = patch_doc
            .quads_matching(
                Any,
                [ns::rdf::type_],
                [ns::solid::InsertDeletePatch],
                [Option::<ArcTerm>::None],
            )
            .map(|statement| statement.unwrap_infallible().s().into_term())
            .exactly_one()
            .map_err(|_| {
                error!("Patch document does not contain exactly one patch resource.");
                InvalidSolidInsertDeletePatchDoc::InvalidPatchResourceCardinality
            })?;

        // Req: A patch resource MUST be identified by a URI or blank node.
        if ![TermKind::Iri, TermKind::BlankNode].contains(&patch_id.kind()) {
            error!("Patch resource identifier is neither an iri nor a blank node.");
            return Err(InvalidSolidInsertDeletePatchDoc::InvalidPatchResourceIdentifier);
        }

        // Resolve patch resource.
        let patch = SolidInsertDeletePatch::try_new(patch_doc.clone(), patch_id.into_term())?;

        Ok(Self { patch_doc, patch })
    }

    /// Get patch doc.
    #[inline]
    pub fn patch_doc(&self) -> &Arc<D> {
        &self.patch_doc
    }

    /// Get patch.
    #[inline]
    pub fn patch(&self) -> &SolidInsertDeletePatch<D> {
        &self.patch
    }
}

/// An error type for error of invalid solid insert delete
/// patch document.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidSolidInsertDeletePatchDoc {
    /// Invalid n3 document.
    #[error("Invalid n3 document.")]
    InvalidN3Doc,

    /// Invalid patch resource cardinality.
    #[error("Invalid patch resource cardinality.")]
    InvalidPatchResourceCardinality,

    /// Invalid patch resource identifier.
    #[error("Invalid patch resource identifier.")]
    InvalidPatchResourceIdentifier,

    /// Invalid patch resource.
    #[error("Invalid patch resource. \n {0}")]
    InvalidPatchResource(#[from] InvalidSolidInsertDeletePatch),
}

/// [`SolidInsertDeletePatch`] is a kind of n3-patch resource as defined by
/// [Solid protocol](https://solidproject.org/ED/protocol#server-patch-n3-default),
/// and corresponds to `solid::InsertDeletePatch` type.
#[derive(Debug)]
pub struct SolidInsertDeletePatch<D>
where
    D: InfallibleDataset + 'static,
{
    /// Id of the patch resource.
    id: ArcTerm,

    /// Patch document.
    patch_doc: Arc<D>,

    /// Internal graph name of ?optional conditions formula.
    opt_conditions_gn: Option<Option<ArcTerm>>,

    /// Internal graph name of ?optional inserts formula.
    opt_inserts_gn: Option<Option<ArcTerm>>,

    /// Internal graph name of ?optional deletes formula.
    opt_deletes_gn: Option<Option<ArcTerm>>,
}

impl<D> Clone for SolidInsertDeletePatch<D>
where
    D: InfallibleDataset,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            patch_doc: self.patch_doc.clone(),
            opt_conditions_gn: self.opt_conditions_gn.clone(),
            opt_inserts_gn: self.opt_inserts_gn.clone(),
            opt_deletes_gn: self.opt_deletes_gn.clone(),
        }
    }
}

impl<D> SolidInsertDeletePatch<D>
where
    D: InfallibleDataset,
{
    /// Try to create new [`SolidInsertDeletePatch`] frm
    /// given patch doc, and id.
    #[allow(clippy::manual_flatten)]
    #[tracing::instrument(name = "try_new_solid_inserts_deletes_patch", skip_all)]
    fn try_new(patch_doc: Arc<D>, id: ArcTerm) -> Result<Self, InvalidSolidInsertDeletePatch> {
        let patch_doc_ref = patch_doc.as_ref();

        // Resolve internal identifiers for formulas.
        let opt_conditions_gn = Self::get_formula_graph_name(&patch_doc, &id, &ns::solid::where_)?;
        let opt_inserts_gn = Self::get_formula_graph_name(&patch_doc, &id, &ns::solid::inserts)?;
        let opt_deletes_gn = Self::get_formula_graph_name(&patch_doc, &id, &ns::solid::deletes)?;

        // Get all variables used in conditions formula.
        let condition_vars: HashSet<ArcTerm> = opt_conditions_gn
            .clone()
            .map(|gn| patch_doc_ref.graph_view(gn).var_set())
            .unwrap_or_default();

        // Validate `?inserts`, `?deletes` formulas.
        for opt_fgn in [opt_inserts_gn.clone(), opt_deletes_gn.clone()] {
            if let Some(fgn) = opt_fgn {
                let formula = patch_doc_ref.graph_view(fgn);

                // Req: "The ?insertions and ?deletions
                // formulae MUST NOT contain blank nodes."
                if formula.blank_nodes().next().is_some() {
                    error!("?inserts/?deletes formula contains invalid blank nodes.");
                    return Err(
                        InvalidSolidInsertDeletePatch::InsertsDeletesFormulaeContainBlankNodes,
                    );
                }

                // Req: "The ?insertions and ?deletions
                // formulae MUST NOT contain variables that
                // do not occur in the ?conditions formula."
                if !formula.var_set().is_subset(&condition_vars) {
                    error!("?inserts/?deletes formula contains variables not defined in ?conditions formula.");
                    return Err(
                        InvalidSolidInsertDeletePatch::InsertsDeletesFormulaeContainUnknownVariables,
                    );
                }
            }
        }

        Ok(Self {
            id,
            patch_doc,
            opt_conditions_gn,
            opt_inserts_gn,
            opt_deletes_gn,
        })
    }

    /// Get id of the patch resource.
    #[inline]
    pub fn id(&self) -> &ArcTerm {
        &self.id
    }

    /// Resolve effective op of the patch operation.
    #[inline]
    pub fn effective_ops(&self) -> HashSet<PatchEffectiveOperation> {
        let mut ops = HashSet::new();
        // Req: When ?conditions is non-empty, servers MUST
        // treat the request as a Read operation.
        if !self.is_empty_formula(self.opt_conditions_gn.clone()) {
            ops.insert(PatchEffectiveOperation::Read);
        }

        // Req: When ?insertions is non-empty, servers MUST
        // (also) treat the request as an Append operation.
        if !self.is_empty_formula(self.opt_inserts_gn.clone()) {
            ops.insert(PatchEffectiveOperation::Append);
        }

        // Req: When ?deletions is non-empty, servers MUST
        // treat the request as a Read and Write operation.
        if !self.is_empty_formula(self.opt_deletes_gn.clone()) {
            ops.insert(PatchEffectiveOperation::Read);
            ops.insert(PatchEffectiveOperation::Write);
        }

        ops
    }

    /// Get cited formula graph graph name from patch doc.
    fn get_formula_graph_name(
        patch_doc: &D,
        patch_id: &ArcTerm,
        predicate: &NsTerm,
    ) -> Result<Option<Option<ArcTerm>>, InvalidPatchFormulaCardinality> {
        patch_doc
            .quads_matching([patch_id], [predicate], Any, [Option::<ArcTerm>::None])
            .map(|statement| Some(statement.unwrap_infallible().o().into_term()))
            // SolidInsertDeletePatch allows at most one
            // formula for each predicate.
            .at_most_one()
            .map_err(|_| InvalidPatchFormulaCardinality)
    }

    /// Apply patch to target dataset.
    /// Returns effective dataset, and applied resolved patch.
    #[tracing::instrument(skip_all)]
    pub fn apply<TD: InfallibleMutableDataset + SetDataset>(
        &self,
        mut target_dataset: TD,
    ) -> Result<(TD, ResolvedPatch), SolidInsertDeletePatchError> {
        // Resolve effective patch.
        let resolved_patch = self.resolve_patch(&target_dataset)?;

        // Apply deletions.
        if !resolved_patch.deletions.is_empty() {
            // Req: "The triples resulting from ?deletions
            // are to be removed from the RDF dataset."
            let removed = target_dataset
                .remove_all(resolved_patch.deletions.as_dataset().quads())
                .map_err(|_| {
                    error!("Error in removing triples from target dataset.");
                    SolidInsertDeletePatchError::TargetDatasetMutationError
                })?;

            // Req: "If the set of triples resulting from ?deletions is non-empty and the dataset
            // does not contain all of these triples, the server MUST respond with a 409 status code."
            if removed != resolved_patch.deletions.len() {
                error!("Removed triple count is not same as formula triples count.");
                return Err(SolidInsertDeletePatchError::DeletionsIsNotSubSetOfTargetGraph);
            }

            debug!("Removed {} triples from target dataset.", removed);
        }

        // Req: "The triples resulting from ?insertions are
        // to be added to the RDF dataset, with each blank
        // node from ?insertions resulting in a newly created
        // blank node."

        // NOTE: blank node names were appended with random
        // suffix while resolving patch.
        target_dataset
            .insert_all(resolved_patch.insertions.as_dataset().quads())
            .map_err(|_| {
                error!("Error in inserting triples into target dataset.");
                SolidInsertDeletePatchError::TargetDatasetMutationError
            })?;

        info!("Patch application success.");

        Ok((target_dataset, resolved_patch))
    }

    /// Apply conditions, and resolve effective deletions and
    /// insertions.
    ///
    /// Req:
    /// - Start from the RDF dataset in the target document,
    /// or an empty RDF dataset if the target resource does
    /// not exist yet.
    ///
    /// - If ?conditions is non-empty, find all (possibly
    /// empty) variable mappings such that all of the
    /// resulting triples occur in the dataset.
    ///
    /// - If no such mapping exists, or if multiple mappings
    /// exist, the server MUST respond with a 409 status
    /// code. [Source]
    ///
    /// - The resulting variable mapping is propagated to
    /// the ?deletions and ?insertions formulae to obtain two
    /// sets of resulting triples.
    fn resolve_patch<TD>(
        &self,
        target_dataset: &TD,
    ) -> Result<ResolvedPatch, SolidInsertDeletePatchError>
    where
        TD: InfallibleMutableDataset + SetDataset,
    {
        // Get formulas.
        let conditions: Vec<[ArcTerm; 3]> = self.formula(self.opt_conditions_gn.clone());
        let mut deletions = self.formula(self.opt_deletes_gn.clone());
        let mut insertions = self.formula(self.opt_inserts_gn.clone());

        // Req:  If ?conditions is non-empty, find all
        // (possibly empty) variable mappings such that all
        // of the resulting triples occur in the dataset.
        if !conditions.is_empty() {
            // Query and resolve binding map.
            let binding_map = Query::Triples(conditions)
                .process(&target_dataset.graph_view(Option::<ArcTerm>::None))
                .exactly_one()
                .map_err(|_| {
                    error!("Invalid number of matched variable mappings");
                    SolidInsertDeletePatchError::InvalidMatchedVariableMappingsCardinality
                })?
                .map_err(|_| {
                    error!("Error in querying target dataset.");
                    SolidInsertDeletePatchError::TargetDatasetQueryError
                })?;

            // Apply bindings to deletions/insertions
            Self::apply_bindings(&binding_map, &mut deletions);
            Self::apply_bindings(&binding_map, &mut insertions);
        }

        // Return resolved patch.
        Ok(ResolvedPatch {
            deletions: deletions.into_iter().collect(),
            insertions: insertions.into_iter().collect(),
        })
    }

    /// Get formula from formula internal id..
    fn formula<G: CollectibleGraph + Default>(&self, opt_formula_gn: Option<Option<ArcTerm>>) -> G {
        opt_formula_gn
            .map(|fgn| {
                self.patch_doc
                    .graph_view(fgn)
                    .triples()
                    .collect_triples::<G>()
                    .expect("Infallible")
            })
            // Req: When not present, they are presumed to be the empty formula {}.
            .unwrap_or_default()
    }

    /// Get if formula is empty.
    fn is_empty_formula(&self, opt_formula_gn: Option<Option<ArcTerm>>) -> bool {
        opt_formula_gn
            .map(|fgn| self.patch_doc.graph_view(fgn).triples().next().is_none())
            .unwrap_or(true)
    }

    // Apply bindings to formula.
    fn apply_bindings(binding_map: &BindingMap, formula: &mut Vec<[ArcTerm; 3]>) {
        // Get a random suffix to make any blank node
        // identifiers random.
        let bnode_random_suffix = rand::random::<u16>();

        for triple in formula {
            for i in 0..3 {
                // If term is a variable, replace with it's bound value.
                if triple[i].kind() == TermKind::Variable {
                    let mut bound_val: ArcTerm =
                        binding_map[triple[i].variable().expect("Checked before").as_str()].clone();

                    // If bound val is a blank node, add
                    // random suffix to it's name to avoid
                    // collisions.
                    if bound_val.kind() == TermKind::BlankNode {
                        bound_val = BnodeId::new_unchecked(format!(
                            "{}_{}",
                            bound_val.bnode_id().expect("Checked.").as_str(),
                            bnode_random_suffix
                        ))
                        .into_term()
                    }
                    triple[i] = bound_val;
                }
            }
        }
    }
}

/// A struct representing resolved patch.
#[derive(Debug, Clone)]
pub struct ResolvedPatch {
    /// Triples to be deleted.
    deletions: HashSet<[ArcTerm; 3]>,

    /// Triples to be inserted.
    insertions: HashSet<[ArcTerm; 3]>,
}

/// An error denoting invalid insert-delete patch.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidSolidInsertDeletePatch {
    /// Invalid patch formula cardinality.
    #[error("Invalid patch formula cardinality")]
    InvalidPatchFormulaCardinality(#[from] InvalidPatchFormulaCardinality),

    /// \?inserts, ?deletes formulae contain blank nodes.
    #[error("?inserts, ?deletes formulae contain blank nodes.")]
    InsertsDeletesFormulaeContainBlankNodes,

    /// \?inserts, ?deletes formulae contain unknown variables.
    #[error("?inserts, ?deletes formulae contain unknown variables.")]
    InsertsDeletesFormulaeContainUnknownVariables,
}

/// An error type for error of invalid patch formula
/// cardinality.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Invalid patch formula cardinality")]
pub struct InvalidPatchFormulaCardinality;

/// Error in applying a solid insert-delete patch against a
/// target dataset.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SolidInsertDeletePatchError {
    /// Invalid matched variable mappings cardinality.
    #[error("Invalid matched variable mappings cardinality.")]
    InvalidMatchedVariableMappingsCardinality,

    /// Resolved deletions is not a subset of target graph.
    #[error("Resolved deletions is not a subset of target graph.")]
    DeletionsIsNotSubSetOfTargetGraph,

    /// Target dataset query error.
    #[error("Target dataset query error.")]
    TargetDatasetQueryError,

    /// Target dataset mutation error.
    #[error("Target dataset mutation error.")]
    TargetDatasetMutationError,
}

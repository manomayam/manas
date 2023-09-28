//! I define few ad-hoc utils to handle graphs.
//!

use std::{collections::HashSet, fs::File, io::BufReader, path::PathBuf};

use anyhow::Context;
use rdf_utils::model::term::{ArcIriRef, ArcTerm};
use sophia_api::{
    graph::{CollectibleGraph, Graph},
    source::TripleSource,
    term::{matcher::Any, Term},
    triple::Triple,
};
use sophia_turtle::parser::turtle;

mod seal {
    use sophia_api::graph::Graph;

    pub trait Sealed {}

    impl<G: Graph> Sealed for G {}
}

/// An extension trait for [`Graph`].
pub trait GraphExt: Graph + seal::Sealed {
    /// Get objects of statements with given predicate, ifthey are iris.
    fn iri_objects_of_statement_with_predicate<TP: Term>(&self, p: &TP) -> HashSet<ArcIriRef> {
        let mut oids = HashSet::new();
        for triple in self.triples_matching(Any, [p.borrow_term()], Any).flatten() {
            let o: ArcTerm = triple.o().into_term();
            if let ArcTerm::Iri(oid) = o {
                oids.insert(oid);
            }
        }

        oids
    }

    /// Get iri object of a functional statement with given subject and predicate.
    fn iri_object_of_functional_statement_with<TS: Term, TP: Term>(
        &self,
        s: &TS,
        p: &TP,
    ) -> Option<ArcIriRef> {
        self.iri_objects_of_statements_with(s, p).into_iter().next()
    }

    /// Get iri objects of statements with given subject and predicate.
    fn iri_objects_of_statements_with<TS: Term, TP: Term>(&self, s: &TS, p: &TP) -> Vec<ArcIriRef> {
        self.triples_matching([s.borrow_term()], [p.borrow_term()], Any)
            .filter_map(|tr| {
                tr.ok().and_then(|t| {
                    if let ArcTerm::Iri(o_iri) = t.o().into_term() {
                        Some(o_iri)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Get literal object of a functional statement with given subject and predicate.
    fn literal_object_of_functional_statement_with<TS: Term, TP: Term>(
        &self,
        s: &TS,
        p: &TP,
    ) -> Option<String> {
        self.literal_objects_of_statements_with(s, p)
            .into_iter()
            .next()
    }

    /// Get literal objects of statements with given subject and predicate.
    fn literal_objects_of_statements_with<TS: Term, TP: Term>(
        &self,
        s: &TS,
        p: &TP,
    ) -> Vec<String> {
        self.triples_matching([s.borrow_term()], [p.borrow_term()], Any)
            .filter_map(|tr| {
                tr.ok().and_then(|t| {
                    if let ArcTerm::LiteralLanguage(o_lit, _) = t.o().into_term::<ArcTerm>() {
                        Some(o_lit.as_ref().to_owned())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Try to parse the graph from given turtle file
    fn try_from_turtle_file(f_path: &PathBuf) -> Result<Self, anyhow::Error>
    where
        Self: CollectibleGraph,
        Self::Error: Send + Sync + 'static,
    {
        let f = File::open(f_path).with_context(|| {
            format!(
                "Error in opening dataset file: {}",
                f_path.to_string_lossy()
            )
        })?;
        let f = BufReader::new(f);
        let source = turtle::parse_bufread(f);

        source.collect_triples().with_context(|| {
            format!(
                "Error in loading graph from file: {}",
                f_path.to_string_lossy()
            )
        })
    }

    /// Try to parse the graph from given turtle string
    fn try_from_turtle_str(ttl_str: &str) -> Result<Self, anyhow::Error>
    where
        Self: CollectibleGraph,
        Self::Error: Send + Sync + 'static,
    {
        let source = turtle::parse_str(ttl_str);
        source
            .collect_triples()
            .with_context(|| "Error in loading dataset from str")
    }
}

impl<G: Graph> GraphExt for G {}

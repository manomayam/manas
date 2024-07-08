//! Query processing over RDF graphs and datasets.
//!
//! **Important**: this is a preliminary and incomplete implementation.
//! The API of this module is likely to change heavily in the future.

use std::{collections::HashMap, iter::once};

use resiter::map::*;
use sophia_api::{
    graph::{GResult, GTripleSource},
    prelude::Graph,
    term::{matcher::TermMatcher, FromTerm, Term, TermKind},
    triple::Triple,
};

use crate::model::term::ArcTerm;

trait TermExt: Term {
    /// Shim for 0.7 compat
    fn value(&self) -> String;
}

impl<T: Term> TermExt for T {
    fn value(&self) -> String {
        match self.kind() {
            TermKind::Iri => self.iri().unwrap().to_string(),
            TermKind::Literal => self.lexical_form().unwrap().to_string(),
            TermKind::BlankNode => self.bnode_id().unwrap().to_string(),
            TermKind::Triple => {
                let triple = self.triple().unwrap();
                format!(
                    "<<{} {} {}>>",
                    triple.s().value(),
                    triple.p().value(),
                    triple.o().value()
                )
            }
            TermKind::Variable => self.variable().unwrap().to_string(),
        }
    }
}

/// A map associating variable names to [`ArcTerm`]s.
pub type BindingMap = HashMap<String, ArcTerm>;

/// A query can be processed against a graph, producing a sequence of binding maps.
pub enum Query {
    /// [Basic graph pattern](https://www.w3.org/TR/sparql11-query/#BasicGraphPatterns)
    Triples(Vec<[ArcTerm; 3]>),
}

impl Query {
    fn prepare<G: Graph>(&mut self, graph: &G, initial_bindings: &BindingMap) {
        match self {
            Query::Triples(triples) => {
                // sorts triple from q according to how many results they may give
                let mut hints: Vec<_> = triples
                    .iter()
                    .map(|t| {
                        let tm = vec![
                            matcher(t.s(), initial_bindings),
                            matcher(t.p(), initial_bindings),
                            matcher(t.o(), initial_bindings),
                        ];
                        let hint = triples_matching(graph, &tm).size_hint();
                        (hint.1.unwrap_or(usize::MAX), hint.0)
                    })
                    .collect();
                for i in 1..hints.len() {
                    let mut j = i;
                    while j > 0 && hints[j - 1] > hints[j] {
                        hints.swap(j - 1, j);
                        triples.swap(j - 1, j);
                        j -= 1;
                    }
                }
            }
        }
    }

    /// Process this query against the given graph, and return an fallible iterator of BindingMaps.
    ///
    /// The iterator may fail (i.e. yield `Err`) if an operation on the graph fails.
    pub fn process<'s, G: Graph>(
        &'s mut self,
        graph: &'s G,
    ) -> Box<dyn Iterator<Item = GResult<G, BindingMap>> + 's> {
        self.process_with(graph, BindingMap::new())
    }

    /// Process this query against the given graph, and return an fallible iterator of BindingMaps,
    /// starting with the given bindings.
    ///
    /// The iterator may fail (i.e. yield `Err`) if an operation on the graph fails.
    pub fn process_with<'s, G: Graph>(
        &'s mut self,
        graph: &'s G,
        initial_bindings: BindingMap,
    ) -> Box<dyn Iterator<Item = GResult<G, BindingMap>> + 's> {
        self.prepare(graph, &initial_bindings);
        match self {
            Query::Triples(triples) => bindings_for_triples(graph, triples, initial_bindings),
        }
    }
}

/// Iter over the bindings of all triples in `q` for graph `g`, given the binding `b`.
fn bindings_for_triples<'a, G>(
    g: &'a G,
    q: &'a [[ArcTerm; 3]],
    b: BindingMap,
) -> Box<dyn Iterator<Item = GResult<G, BindingMap>> + 'a>
where
    G: Graph,
{
    if q.is_empty() {
        Box::new(once(Ok(b)))
    } else {
        Box::new(
            bindings_for_triple(g, &q[0], b).flat_map(move |res| match res {
                Err(err) => Box::new(once(Err(err))),
                Ok(b2) => bindings_for_triples(g, &q[1..], b2),
            }),
        )
    }
}

/// Iter over the bindings of triple `tq` for graph `g`, given the binding `b`.
fn bindings_for_triple<'a, G>(
    g: &'a G,
    tq: &'a [ArcTerm; 3],
    b: BindingMap,
) -> impl Iterator<Item = GResult<G, BindingMap>> + 'a
where
    G: Graph,
{
    let tm = [
        matcher(tq.s(), &b),
        matcher(tq.p(), &b),
        matcher(tq.o(), &b),
    ];
    // NB: the unsafe code below is used to convince the compiler that &tm has lifetime 'a .
    // We can guarantee that because the closure below takes ownership of tm,
    // and it will live as long as the returned iterator.
    triples_matching(g, unsafe { &*(&tm[..] as *const [Binding]) }).map_ok(move |tr| {
        let mut b2 = b.clone();
        if tm[0].is_free() {
            b2.insert(tq.s().value(), ArcTerm::from_term(tr.s()));
        }
        if tm[1].is_free() {
            b2.insert(tq.p().value(), ArcTerm::from_term(tr.p()));
        }
        if tm[2].is_free() {
            b2.insert(tq.o().value(), ArcTerm::from_term(tr.o()));
        }
        b2
    })
}

/// Make a matcher corresponding to term `t`, given binding `b`.
fn matcher(t: &ArcTerm, b: &BindingMap) -> Binding {
    if let ArcTerm::Variable(var) = t {
        let vname: &str = var.as_str();
        b.get(vname).cloned().into()
    } else {
        Binding::Exactly(t.clone())
    }
}

/// A wrapper around Graph::triples_matchings, with more convenient parameters.
fn triples_matching<'a, G>(g: &'a G, tm: &'a [Binding]) -> GTripleSource<'a, G>
where
    G: Graph,
{
    debug_assert_eq!(tm.len(), 3, "tm.len() = {}", tm.len());
    let s = &tm[0];
    let p = &tm[1];
    let o = &tm[2];
    g.triples_matching(s, p, o)
}

enum Binding {
    Any,
    Exactly(ArcTerm),
}

impl TermMatcher for Binding {
    type Term = ArcTerm;

    fn matches<T2: Term + ?Sized>(&self, term: &T2) -> bool {
        match self {
            Binding::Any => true,
            Binding::Exactly(t) => Term::eq(t, term.borrow_term()),
        }
    }
}

impl TermMatcher for &Binding {
    type Term = ArcTerm;

    fn matches<T2: Term + ?Sized>(&self, term: &T2) -> bool {
        Binding::matches(self, term)
    }
}
impl From<Option<ArcTerm>> for Binding {
    fn from(other: Option<ArcTerm>) -> Binding {
        match other {
            None => Binding::Any,
            Some(t) => Binding::Exactly(t),
        }
    }
}

trait BindingExt {
    fn is_free(&self) -> bool;
}

impl BindingExt for Binding {
    fn is_free(&self) -> bool {
        matches!(self, Binding::Any)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use sophia_api::{
        ns::{rdf, Namespace},
        prelude::{Iri, MutableGraph},
        term::VarName,
    };

    use super::*;

    #[test]
    fn test_bindings_for_triple_0var_0() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_event = schema.get("Event").unwrap();
        let x_alice = Iri::new_unchecked("http://example.org/alice").into_term();

        let tq: [ArcTerm; 3] = [x_alice, rdf::type_.into_term(), s_event.into_term()];

        let results: Result<Vec<_>, _> = bindings_for_triple(&g, &tq, BindingMap::new()).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_bindings_for_triple_0var_1() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_person = schema.get("Person").unwrap();
        let x_alice = Iri::new_unchecked("http://example.org/alice").into_term();

        let tq: [ArcTerm; 3] = [x_alice, rdf::type_.into_term(), s_person.into_term()];

        let results: Result<Vec<BindingMap>, _> =
            bindings_for_triple(&g, &tq, BindingMap::new()).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        for r in results.iter() {
            assert_eq!(r.len(), 0);
        }
    }

    #[test]
    fn test_bindings_for_triple_1var_0() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_event = schema.get("Event").unwrap();

        let v1 = VarName::new_unchecked("v1").into_term();

        let tq: [ArcTerm; 3] = [v1, rdf::type_.into_term(), s_event.into_term()];

        let results: Result<Vec<BindingMap>, _> =
            bindings_for_triple(&g, &tq, BindingMap::new()).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_bindings_for_triple_1var() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_person = schema.get("Person").unwrap();

        let v1 = VarName::new_unchecked("v1").into_term();

        let tq: [ArcTerm; 3] = [v1, rdf::type_.into_term(), s_person.into_term()];

        let results: Result<Vec<BindingMap>, _> =
            bindings_for_triple(&g, &tq, BindingMap::new()).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        for r in results.iter() {
            assert_eq!(r.len(), 1);
            assert!(r.contains_key("v1"));
        }
        let mut results: Vec<_> = results
            .into_iter()
            .map(|b| b.get("v1").unwrap().value())
            .collect();
        results.sort();
        assert_eq!(results[0], "http://example.org/alice");
        assert_eq!(results[1], "http://example.org/bob");
        assert_eq!(results[2], "http://example.org/charlie");
    }

    #[test]
    fn test_bindings_for_triple_2var() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_name = schema.get("name").unwrap();

        let v1 = VarName::new_unchecked("v1").into_term();
        let v2 = VarName::new_unchecked("v2").into_term();

        let tq: [ArcTerm; 3] = [v1, s_name.into_term(), v2];

        let results: Result<Vec<BindingMap>, _> =
            bindings_for_triple(&g, &tq, BindingMap::new()).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 5);
        for r in results.iter() {
            assert_eq!(r.len(), 2);
            assert!(r.contains_key("v1"));
            assert!(r.contains_key("v2"));
        }
        let mut results: Vec<_> = results
            .into_iter()
            .map(|b| {
                format!(
                    "{} {}",
                    b.get("v1").unwrap().value(),
                    b.get("v2").unwrap().value(),
                )
            })
            .collect();
        results.sort();
        assert_eq!(results[0], "http://example.org/alice Alice");
        assert_eq!(results[1], "http://example.org/alice_n_bob Alice & Bob");
        assert_eq!(results[2], "http://example.org/bob Bob");
        assert_eq!(results[3], "http://example.org/charlie Charlie");
        assert_eq!(results[4], "http://example.org/dan Dan");
    }

    #[test]
    fn test_query_triples() {
        let g = data();

        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_person = schema.get("Person").unwrap();
        let s_name = schema.get("name").unwrap();

        let v1: ArcTerm = VarName::new_unchecked("v1").into_term();
        let v2: ArcTerm = VarName::new_unchecked("v2").into_term();

        #[allow(clippy::redundant_clone)]
        let mut q = Query::Triples(vec![
            [v1.clone(), s_name.into_term(), v2.clone()],
            [v1.clone(), rdf::type_.into_term(), s_person.into_term()],
        ]);

        let results: Result<Vec<BindingMap>, _> = q.process(&g).collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        for r in results.iter() {
            assert_eq!(r.len(), 2);
            assert!(r.contains_key("v1"));
            assert!(r.contains_key("v2"));
        }
        let mut results: Vec<_> = results
            .into_iter()
            .map(|b| {
                format!(
                    "{} {}",
                    b.get("v1").unwrap().value(),
                    b.get("v2").unwrap().value(),
                )
            })
            .collect();
        results.sort();
        assert_eq!(results[0], "http://example.org/alice Alice");
        assert_eq!(results[1], "http://example.org/bob Bob");
        assert_eq!(results[2], "http://example.org/charlie Charlie");
    }

    fn data() -> HashSet<[ArcTerm; 3]> {
        let schema = Namespace::new("http://schema.org/").unwrap();
        let s_person = schema.get("Person").unwrap();
        let s_organization = schema.get("Organization").unwrap();
        let s_name = schema.get("name").unwrap();
        let s_member = schema.get("member").unwrap();

        let example = Namespace::new("http://example.org/").unwrap();
        let x_alice = example.get("alice").unwrap();
        let x_bob = example.get("bob").unwrap();
        let x_charlie = example.get("charlie").unwrap();
        let x_dan = example.get("dan").unwrap();
        let x_alice_n_bob = example.get("alice_n_bob").unwrap();

        let mut g = HashSet::new();

        MutableGraph::insert(&mut g, x_alice, rdf::type_, s_person).unwrap();
        MutableGraph::insert(&mut g, x_alice, s_name, "Alice").unwrap();
        MutableGraph::insert(&mut g, x_bob, rdf::type_, s_person).unwrap();
        MutableGraph::insert(&mut g, x_bob, s_name, "Bob").unwrap();
        MutableGraph::insert(&mut g, x_charlie, rdf::type_, s_person).unwrap();
        MutableGraph::insert(&mut g, x_charlie, s_name, "Charlie").unwrap();
        MutableGraph::insert(&mut g, x_dan, s_name, "Dan").unwrap();
        MutableGraph::insert(&mut g, x_alice_n_bob, rdf::type_, s_organization).unwrap();
        MutableGraph::insert(&mut g, x_alice_n_bob, s_name, "Alice & Bob").unwrap();
        MutableGraph::insert(&mut g, x_alice_n_bob, s_member, x_alice).unwrap();
        MutableGraph::insert(&mut g, x_alice_n_bob, s_member, x_bob).unwrap();

        g
    }
}

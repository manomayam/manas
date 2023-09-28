//! I define models for rdf graphs.
//!

use std::{collections::HashSet, convert::Infallible, fmt::Debug, hash::Hash, ops::Deref};

use sophia_api::{
    graph::GTerm,
    prelude::{Graph, MutableGraph},
    term::{FromTerm, Term},
};
use unwrap_infallible::UnwrapInfallible;

/// A trait for infallible graphs.
pub trait InfallibleGraph: Graph<Error = Infallible> + Debug {
    /// Get set of all variables in this graph.
    fn var_set<T: FromTerm + PartialEq + Eq + Hash>(&self) -> HashSet<T> {
        self.variables()
            .map(|vr| vr.unwrap_infallible().into_term())
            .collect()
    }
}

impl<G> InfallibleGraph for G where G: Graph<Error = Infallible> + Debug {}

/// A trait for infallible mutable graphs.
pub trait InfallibleMutableGraph:
    MutableGraph<Error = Infallible, MutationError = Infallible> + Debug
{
}

impl<G> InfallibleMutableGraph for G where
    G: MutableGraph<Error = Infallible, MutationError = Infallible> + Debug
{
}

/// Type of static graph terms.
pub type GSTerm<G> = GTerm<'static, G>;

/// A graph type to wrap around few common rust types, providing
/// graph implementations for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CompatGraph<D>(pub D);

impl<T> Deref for CompatGraph<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "compat-ecow")]
pub use ecow_graph_impl::*;

#[cfg(feature = "compat-ecow")]
mod ecow_graph_impl {
    use ecow::EcoVec;
    use sophia_api::{
        graph::{CollectibleGraph, GTripleSource, MgResult},
        source::{StreamError::SourceError, StreamResult, TripleSource},
        term::FromTerm,
        triple::{TBorrowTerm, Triple},
    };

    use super::*;

    impl<T: Triple> Graph for CompatGraph<EcoVec<T>> {
        type Triple<'x> = [TBorrowTerm<'x, T>; 3] where Self: 'x;

        type Error = Infallible;

        #[inline]
        fn triples(&self) -> GTripleSource<Self> {
            self.0[..].triples()
        }
    }

    impl<T: Term + FromTerm + Clone> MutableGraph for CompatGraph<EcoVec<[T; 3]>> {
        type MutationError = Infallible;

        fn insert<TS, TP, TO>(
            &mut self,
            s: TS,
            p: TP,
            o: TO,
        ) -> Result<bool, <Self as MutableGraph>::MutationError>
        where
            TS: Term,
            TP: Term,
            TO: Term,
        {
            self.0.push([s.into_term(), p.into_term(), o.into_term()]);
            Ok(true)
        }

        fn remove<TS, TP, TO>(&mut self, s: TS, p: TP, o: TO) -> MgResult<Self, bool>
        where
            TS: Term,
            TP: Term,
            TO: Term,
        {
            let s = s.borrow_term();
            let p = p.borrow_term();
            let o = o.borrow_term();
            match self.0.iter().position(|t| t.matched_by([s], [p], [o])) {
                None => Ok(false),
                Some(i) => {
                    self.0.remove(i);
                    Ok(true)
                }
            }
        }
    }

    impl<T: Term + FromTerm + Clone> CollectibleGraph for CompatGraph<EcoVec<[T; 3]>> {
        fn from_triple_source<TS: TripleSource>(
            mut triples: TS,
        ) -> StreamResult<Self, TS::Error, Self::Error> {
            let min_cap = triples.size_hint_triples().0;
            let mut v = EcoVec::with_capacity(min_cap);
            triples
                .for_each_triple(|t| {
                    v.push([t.s().into_term(), t.p().into_term(), t.o().into_term()])
                })
                .map_err(SourceError)?;
            Ok(Self(v))
        }
    }

    /// Type of graph backed by an ecovec.
    pub type EcoGraph<T> = CompatGraph<EcoVec<T>>;
}

//! I define traits and types for subject handles.
//!

use std::{fmt::Debug, marker::PhantomData};

use sophia_api::{
    graph::{GTerm, GTripleSource},
    term::{matcher::Any, FromTerm, Term},
    triple::Triple,
};
use unwrap_infallible::UnwrapInfallible;

use super::graph::{InfallibleGraph, InfallibleMutableGraph};

/// A trait for rdf subject handles.
pub trait Handle: Debug + Sized {
    /// Type of inner term.
    type Term: Term;

    /// Type of error for converting from term to handle.
    type ConvertErr;

    /// Try to create a new handle from given term.
    fn try_new(term: Self::Term) -> Result<Self, Self::ConvertErr>;

    /// Get a reference to inner term.
    fn as_term(&self) -> &Self::Term;

    /// Convert into inner term.
    fn into_term(self) -> Self::Term;
}

/// An iterator that unwraps infallible items of inner triple source.
pub struct UnwrappingIterator<'g, G>
where
    G: InfallibleGraph + 'g,
{
    inner: GTripleSource<'g, G>,
}

impl<'g, G> Iterator for UnwrappingIterator<'g, G>
where
    G: InfallibleGraph + 'g,
{
    type Item = G::Triple<'g>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|r| r.unwrap_infallible())
    }
}

/// An iterator that maps inner triple source items to their
/// object terms.
pub struct ObjectIterator<'g, G: InfallibleGraph + 'g> {
    inner: GTripleSource<'g, G>,
}

impl<'g, G> Iterator for ObjectIterator<'g, G>
where
    G: InfallibleGraph + 'g,
{
    type Item = GTerm<'g, G>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|r| r.unwrap_infallible().to_o())
    }
}

impl<'g, G> ObjectIterator<'g, G>
where
    G: InfallibleGraph + 'g,
{
    /// Convert into term owning iterator.
    pub fn into_term_owning<T: FromTerm>(self) -> OwnedObjectIterator<'g, G, T> {
        OwnedObjectIterator {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }
}

/// An iterator that maps inner triple source items to their
/// owned object terms.
pub struct OwnedObjectIterator<'g, G: InfallibleGraph + 'g, T: FromTerm> {
    inner: GTripleSource<'g, G>,
    _phantom: PhantomData<fn() -> T>,
}

impl<'g, G, T> Iterator for OwnedObjectIterator<'g, G, T>
where
    G: InfallibleGraph + 'g,
    T: FromTerm,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|r| r.unwrap_infallible().to_o().into_term())
    }
}

/// An iterator that maps inner triple source items to their
/// object handles.
pub struct ObjectHandleIterator<'g, G, H>
where
    G: InfallibleGraph + 'g,
    H: Handle<Term = GTerm<'g, G>>,
{
    inner: GTripleSource<'g, G>,
    _phantom: PhantomData<fn() -> H>,
}

impl<'g, G, H: Handle<Term = GTerm<'g, G>>> Iterator for ObjectHandleIterator<'g, G, H>
where
    G: InfallibleGraph + 'g,
{
    type Item = H;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find_map(|r| H::try_new(r.unwrap_infallible().to_o()).ok())
    }
}

impl<'g, G, H> ObjectHandleIterator<'g, G, H>
where
    G: InfallibleGraph + 'g,
    H: Handle<Term = GTerm<'g, G>>,
{
    /// Convert into handle owning iterator.
    #[inline]
    pub fn into_handle_owning<H2>(self) -> OwnedObjectHandleIterator<'g, G, H2>
    where
        H2: Handle,
        H2::Term: FromTerm,
    {
        OwnedObjectHandleIterator {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }
}

/// An iterator that maps inner triple source items to their
/// owned object handles.
pub struct OwnedObjectHandleIterator<'g, G, H>
where
    G: InfallibleGraph + 'g,
    H: Handle,
    H::Term: FromTerm,
{
    inner: GTripleSource<'g, G>,
    _phantom: PhantomData<fn() -> H>,
}

impl<'g, G, H> Iterator for OwnedObjectHandleIterator<'g, G, H>
where
    G: InfallibleGraph + 'g,
    H: Handle,
    H::Term: FromTerm,
{
    type Item = H;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find_map(|r| H::try_new(r.unwrap_infallible().to_o().into_term()).ok())
    }
}

/// An extension trait for [`Handle`].
pub trait HandleExt: Handle {
    /// Get all triples about the subject with given predicate in given graph.
    #[inline]
    fn get_all_triples<'g, G, TP>(&'g self, graph: &'g G, p: &'g TP) -> UnwrappingIterator<'g, G>
    where
        G: InfallibleGraph,
        TP: Term,
    {
        UnwrappingIterator {
            inner: graph.triples_matching([self.as_term().borrow_term()], [p.borrow_term()], Any),
        }
    }

    /// Get all objects of triples about the subject with given
    /// predicate in given graph.
    #[inline]
    fn get_all<'g, G, TP>(&'g self, graph: &'g G, p: &'g TP) -> ObjectIterator<'g, G>
    where
        G: InfallibleGraph,
        TP: Term,
    {
        ObjectIterator {
            inner: graph.triples_matching([self.as_term().borrow_term()], [p.borrow_term()], Any),
        }
    }

    /// Get handles to all objects of triples about the subject
    /// with given predicate in given graph.
    #[inline]
    fn get_all_handles<'g, G, H, TP>(
        &'g self,
        graph: &'g G,
        p: &'g TP,
    ) -> OwnedObjectHandleIterator<'g, G, H>
    where
        G: InfallibleGraph,
        H: Handle,
        H::Term: FromTerm,
        TP: Term,
    {
        OwnedObjectHandleIterator {
            inner: graph.triples_matching([self.as_term().borrow_term()], [p.borrow_term()], Any),
            _phantom: PhantomData,
        }
    }

    /// Get object of first triple about the subject with given
    /// predicate in given graph.
    #[inline]
    fn get_first<'g, G, TP>(&'g self, graph: &'g G, p: &'g TP) -> Option<GTerm<'g, G>>
    where
        G: InfallibleGraph,
        TP: Term,
    {
        self.get_all(graph, p).next()
    }

    /// Get handle to object of first triple about the subject
    /// with given predicate in given graph.
    #[inline]
    fn get_first_handle<'g, G, H, TP>(&'g self, graph: &'g G, p: &'g TP) -> Option<H>
    where
        G: InfallibleGraph,
        H: Handle,
        H::Term: FromTerm,
        TP: Term,
    {
        self.get_all_handles(graph, p).next()
    }

    /// Check if there exist a triple about the subject with
    /// given predicate in given graph.
    #[inline]
    fn has_any<'g, G, TP>(&'g self, graph: &'g G, p: &'g TP) -> bool
    where
        G: InfallibleGraph,
        TP: Term,
    {
        self.get_all_triples(graph, p).next().is_some()
    }

    /// Check if there exist a triple about the subject with
    /// given predicate and given object in given graph.
    #[inline]
    fn has_any_with<'g, G, TP, TO>(&'g self, graph: &'g G, p: &'g TP, o: &'g TO) -> bool
    where
        G: InfallibleGraph,
        TP: Term,
        TO: Term,
    {
        graph
            .triples_matching(
                [self.as_term().borrow_term()],
                [p.borrow_term()],
                [o.borrow_term()],
            )
            .next()
            .is_some()
    }

    /// Check if there are triples about the subject with given predicates with common object in given graph.
    #[inline]
    fn has_common<'g, G, TP1, TP2>(&'g self, graph: &'g G, p1: &'g TP1, p2: &'g TP2) -> bool
    where
        G: InfallibleGraph,
        TP1: Term,
        TP2: Term,
    {
        self.get_all_triples(graph, p1)
            .any(|t| self.has_any_with(graph, p2, &t.o()))
    }

    /// Add a statement about the subject with given predicate and object to the graph.
    /// Returns if the addition actually changed the graph.
    /// Return value is significant only if `G` is also a `SetGraph`.
    #[inline]
    fn add<'s, G, TP, TO>(&'s self, graph: &'s mut G, p: &'s TP, o: &'s TO) -> bool
    where
        G: InfallibleMutableGraph,
        TP: Term,
        TO: Term,
    {
        graph
            .insert(
                self.as_term().borrow_term(),
                p.borrow_term(),
                o.borrow_term(),
            )
            .unwrap_infallible()
    }

    /// Add statements about subject with given predicate, object pairs ro given graph.
    ///
    /// The usize value returned in case of success is not significant unless this graph also implements SetGraph.
    /// If it does, the number of triples that were actually inserted (i.e. that were not already present in this SetGraph) is returned.
    #[inline]
    fn add_all<'s, G, TP, TO>(
        &'s self,
        graph: &'s mut G,
        po_iter: impl Iterator<Item = (&'s TP, &'s TO)>,
    ) -> usize
    where
        G: InfallibleMutableGraph,
        TP: Term + 's,
        TO: Term + 's,
    {
        po_iter
            .filter_map(|(p, o)| self.add(graph, p, o).then_some(true))
            .count()
    }

    /// Add statements about resource with given predicate, and each of object.
    #[inline]
    fn add_all_with<'s, G, TP, TO>(
        &'s self,
        graph: &'s mut G,
        p: &TP,
        o_iter: impl Iterator<Item = &'s TO>,
    ) -> usize
    where
        G: InfallibleMutableGraph,
        TP: Term + 's,
        TO: Term + 's,
    {
        o_iter
            .filter_map(|o| self.add(graph, p, o).then_some(true))
            .count()
    }

    /// Remove any statement about the subject with given predicate and object from given graph.
    /// Returns if the removal actually changed the graph.
    /// Return value is significant only if `G` is also a `SetGraph`.
    #[inline]
    fn remove<'s, G, TP, TO>(&'s self, graph: &'s mut G, p: &'s TP, o: &'s TO) -> bool
    where
        G: InfallibleMutableGraph,
        TP: Term,
        TO: Term,
    {
        graph
            .remove(
                self.as_term().borrow_term(),
                p.borrow_term(),
                o.borrow_term(),
            )
            .unwrap_infallible()
    }

    /// Remove statements about the subject with given predicate, object pairs from given graph.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this graph also implements [`SetGraph`].
    ///
    /// If it does,
    /// the number of triples that were *actually* removed
    /// (i.e. that were not already absent from this [`SetGraph`])
    /// is returned.
    ///
    /// [`SetGraph`]: trait.SetGraph.html
    #[inline]
    fn remove_all<'s, G, TP, TO>(
        &'s self,
        graph: &'s mut G,
        po_iter: impl Iterator<Item = (&'s TP, &'s TO)>,
    ) -> usize
    where
        G: InfallibleMutableGraph,
        TP: Term + 's,
        TO: Term + 's,
    {
        po_iter
            .filter_map(|(p, o)| self.remove(graph, p, o).then_some(true))
            .count()
    }

    /// Remove any statements about the resource with given predicate from given graph.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this graph also implements [`SetGraph`].
    ///
    /// If it does,
    /// the number of triples that were *actually* removed
    /// (i.e. that were not already absent from this [`SetGraph`])
    /// is returned.
    ///
    /// [`SetGraph`]: trait.SetGraph.html
    #[inline]
    fn remove_all_with<'s, G, TP>(&'s self, graph: &'s mut G, p: &'s TP) -> usize
    where
        G: InfallibleMutableGraph,
        TP: Term,
    {
        graph
            .remove_matching([self.as_term().borrow_term()], [p.borrow_term()], Any)
            .unwrap_infallible()
    }

    /// Add statement about the subject with given predicate and object,
    /// and remove any other statements about the subject with given predicate.
    #[inline]
    fn set<'s, G, TP, TO>(&'s self, graph: &'s mut G, p: &'s TP, o: &'s TO)
    where
        G: InfallibleMutableGraph,
        TP: Term,
        TO: Term,
    {
        self.remove_all_with(graph, p);
        self.add(graph, p, o);
    }
}

impl<H: Handle> HandleExt for H {}

/// An error type for errors of invalid subject terms.
#[derive(Debug, thiserror::Error)]
#[error("Invalid subject term.")]
pub struct InvalidSubjectTerm;

/// A macro for quickly defining new subject handle types.
///
/// NOTE: generated code depends on `paste` and `sophia_api`
/// crates.
#[macro_export]
macro_rules! define_handle_type {
    (
        $(#[$outer:meta])*
        $H:ident;
        [
            $(
                $(#[$pouter:meta])*
                ($pm:ident, $p:expr, $PH:ident);
            )*
        ]
    ) => {
        paste::paste! {
            pub use [<handle_impl_$H:snake>]::$H;

            #[allow(unused_imports)]
            mod [<handle_impl_$H:snake>] {
                use $crate::model::{
                    graph::InfallibleGraph,
                    handle::{InvalidSubjectTerm, Handle, HandleExt, OwnedObjectHandleIterator},
                };
                use sophia_api::{
                    term::{FromTerm, Term, TermKind},
                };

                use super::*;

                $(#[$outer])*
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub struct $H<T>(T);

                impl<T: Term> Handle for $H<T> {
                    type Term = T;

                    type ConvertErr = InvalidSubjectTerm;

                    fn try_new(term: Self::Term) -> Result<Self, Self::ConvertErr> {
                        let kind = term.kind();
                        // Reject if term is not an iri or a blank node.
                        if !(kind == TermKind::Iri || kind == TermKind::BlankNode) {
                            return Err(InvalidSubjectTerm);
                        }
                        Ok(Self(term))
                    }

                    #[inline]
                    fn as_term(&self) -> &Self::Term {
                        &self.0
                    }

                    #[inline]
                    fn into_term(self) -> Self::Term {
                        self.0
                    }
                }

                impl<T: Term> $H<T> {
                    /// Get a new handle with mapped inner term.
                    #[inline]
                    pub fn map_term<T2: FromTerm>(self) -> $H<T2> {
                        $H(self.0.into_term())
                    }

                    /// Get a new handle for given term, without any checks.
                    ///
                    /// # Safety
                    ///
                    /// term must be either iri or a blank node.
                    #[inline]
                    pub const unsafe fn new_unchecked(term: T) -> Self {
                        $H(term)
                    }
                }

                impl<T: Term> $H<T> {
                    $(
                        $(#[$pouter])*
                        pub fn $pm<'g, G, T2>(
                            &'g self,
                            graph: &'g G,
                        ) -> OwnedObjectHandleIterator<'g, G, $PH<T2>>
                        where
                            G: InfallibleGraph,
                            T2: FromTerm + Term,
                        {
                            HandleExt::get_all_handles(self, graph, $p)
                        }
                    )*
                }
            }
        }
    };
}

pub use super::description::HAny;

#[cfg(test)]
#[allow(dead_code)]
mod test_define_handle_macro {
    use sophia_api::{ns, term::Term};

    use crate::model::handle::HAny;

    define_handle_type!(
        /// Test definition of a new subject handle type.
        HTest;

        [
            /// About.
            (about, &ns::rdf::about, HAny);

            /// Subject.
            (subject, &ns::rdf::subject, HAny);
        ]
    );

    impl<T: Term> HTest<T> {}
}

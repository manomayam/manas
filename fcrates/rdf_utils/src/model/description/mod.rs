//! I define models for rdf descriptions about a subject resource.
//!  

use std::{borrow::Borrow, fmt::Debug};

use sophia_api::{
    graph::GTerm,
    term::{FromTerm, Term},
};

use super::{
    graph::{InfallibleGraph, InfallibleMutableGraph},
    handle::{Handle, HandleExt, ObjectIterator, OwnedObjectHandleIterator, UnwrappingIterator},
};

/// A trait for representing a subject's description.
pub trait Description: Debug {
    /// Type of description's subject handle.
    type Handle: Handle;

    /// Type of description graph.
    type Graph: InfallibleGraph;

    /// Get the handle to subject of the description.
    fn handle(&self) -> &Self::Handle;

    /// Get a reference to graph of the description.
    fn graph(&self) -> &Self::Graph;

    /// Get references to subject and graph parts of the description.
    #[inline]
    fn as_parts(&self) -> (&Self::Handle, &Self::Graph) {
        (self.handle(), self.graph())
    }
}

/// A trait for mutable descriptions.
pub trait MutableDescription: Description<Graph = Self::MutGraph> {
    /// Type of mutable graph.
    type MutGraph: InfallibleMutableGraph;

    /// Get a mutable reference to graph of the description.
    fn graph_mut(&mut self) -> &mut Self::Graph;

    /// Get references to handle and mut-graph parts of the description.
    fn as_parts_mut(&mut self) -> (&Self::Handle, &mut Self::Graph);
}

/// A trait for simple descriptions that can be constructed
/// from their subject handle and graph without
/// any further validation.
///
pub trait SimpleDescription: Description {
    /// Type of wrapped graph.
    type WGraph: Borrow<Self::Graph>;

    /// Create a new instance with given params.
    fn new(handle: Self::Handle, wgraph: Self::WGraph) -> Self;

    /// Get the wrapped graph backing this description.
    fn wgraph(&self) -> &Self::WGraph;
}

/// An extension trait for [`Description`].
pub trait DescriptionExt: Description {
    /// Get all triples about the subject with given predicate.
    #[inline]
    fn get_all_triples<'g, TP>(&'g self, p: &'g TP) -> UnwrappingIterator<'g, Self::Graph>
    where
        TP: Term,
    {
        self.handle().get_all_triples(self.graph(), p)
    }

    /// Get all objects of triples about the subject with given
    /// predicate.
    #[inline]
    fn get_all<'g, TP>(&'g self, p: &'g TP) -> ObjectIterator<'g, Self::Graph>
    where
        TP: Term,
    {
        self.handle().get_all(self.graph(), p)
    }

    /// Get handles to all objects of triples about the subject
    /// with given predicate.
    #[inline]
    fn get_all_handles<'g, H, TP>(
        &'g self,
        p: &'g TP,
    ) -> OwnedObjectHandleIterator<'g, Self::Graph, H>
    where
        H: Handle,
        H::Term: FromTerm,
        TP: Term,
    {
        self.handle().get_all_handles(self.graph(), p)
    }

    /// Get object of first triple about the subject with given
    /// predicate.
    #[inline]
    fn get_first<'g, TP>(&'g self, p: &'g TP) -> Option<GTerm<Self::Graph>>
    where
        TP: Term,
    {
        self.handle().get_first(self.graph(), p)
    }

    /// Get handle to object of first triple about the subject
    /// with given predicate.
    #[inline]
    fn get_first_handle<'g, H, TP>(&'g self, p: &'g TP) -> Option<H>
    where
        H: Handle,
        H::Term: FromTerm,
        TP: Term,
    {
        self.handle().get_first_handle(self.graph(), p)
    }

    /// Check if there exist a triple about the subject with
    /// given predicate.
    #[inline]
    fn has_any<'g, TP>(&'g self, p: &'g TP) -> bool
    where
        TP: Term,
    {
        self.handle().has_any(self.graph(), p)
    }

    /// Check if there exist a triple about the subject with
    /// given predicate and given object.
    #[inline]
    fn has_any_with<'g, TP, TO>(&'g self, p: &'g TP, o: &'g TO) -> bool
    where
        TP: Term,
        TO: Term,
    {
        self.handle().has_any_with(self.graph(), p, o)
    }

    /// Check if there are triples about the subject with given
    /// predicates with common object.
    #[inline]
    fn has_common<'g, TP1, TP2>(&'g self, p1: &'g TP1, p2: &'g TP2) -> bool
    where
        TP1: Term,
        TP2: Term,
    {
        self.handle().has_common(self.graph(), p1, p2)
    }

    /// Add a statement about the subject with given predicate
    /// and object to the description.
    /// Returns if the addition actually changed the graph.
    /// Return value is significant only if backing graph is
    /// also a `SetGraph`.
    #[inline]
    fn add<'s, TP, TO>(&'s mut self, p: &'s TP, o: &'s TO) -> bool
    where
        Self: MutableDescription,
        TP: Term,
        TO: Term,
    {
        let (h, g) = self.as_parts_mut();
        h.add(g, p, o)
    }

    /// Add statements about subject with given predicate,
    /// object pairs ro given graph.
    ///
    /// The usize value returned in case of success is not
    /// significant unless this graph also implements SetGraph.
    /// If it does, the number of triples that were actually
    /// inserted (i.e. that were not already present in this
    /// SetGraph) is returned.
    #[inline]
    fn add_all<'s, G, TP, TO>(
        &'s mut self,
        po_iter: impl Iterator<Item = (&'s TP, &'s TO)>,
    ) -> usize
    where
        Self: MutableDescription,
        TP: Term + 's,
        TO: Term + 's,
    {
        let (h, g) = self.as_parts_mut();
        h.add_all(g, po_iter)
    }

    /// Add statements about resource with given predicate, and
    /// each of object.
    #[inline]
    fn add_all_with<'s, TP, TO>(&'s mut self, p: &TP, o_iter: impl Iterator<Item = &'s TO>) -> usize
    where
        Self: MutableDescription,
        TP: Term + 's,
        TO: Term + 's,
    {
        let (h, g) = self.as_parts_mut();
        h.add_all_with(g, p, o_iter)
    }

    /// Remove any statement about the subject with given predicate and object from given graph.
    /// Returns if the removal actually changed the graph.
    /// Return value is significant only if `G` is also a
    /// `SetGraph`.
    #[inline]
    fn remove<'s, TP, TO>(&'s mut self, p: &'s TP, o: &'s TO) -> bool
    where
        Self: MutableDescription,
        TP: Term,
        TO: Term,
    {
        let (h, g) = self.as_parts_mut();
        h.remove(g, p, o)
    }

    /// Remove statements about the subject with given
    /// predicate, object pairs from given graph.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this graph also implements [`SetGraph`](sophia_api::graph::SetGraph).
    ///
    /// If it does,
    /// the number of triples that were *actually* removed
    /// (i.e. that were not already absent from this
    /// [`SetGraph`](sophia_api::graph::SetGraph) is returned.
    ///
    /// [`SetGraph`]: trait.SetGraph.html
    #[inline]
    fn remove_all<'s, TP, TO>(
        &'s mut self,
        po_iter: impl Iterator<Item = (&'s TP, &'s TO)>,
    ) -> usize
    where
        Self: MutableDescription,
        TP: Term + 's,
        TO: Term + 's,
    {
        let (h, g) = self.as_parts_mut();
        h.remove_all(g, po_iter)
    }

    /// Remove any statements about the resource with given
    /// predicate from given graph.
    ///
    /// # Return value
    /// The `usize` value returned in case of success is
    /// **not significant unless** this graph also implements [`SetGraph`](sophia_api::graph::SetGraph).
    ///
    /// If it does,
    /// the number of triples that were *actually* removed
    /// (i.e. that were not already absent from this
    /// [`SetGraph`](sophia_api::graph::SetGraph)) is returned.
    ///
    #[inline]
    fn remove_all_with<'s, TP>(&'s mut self, p: &'s TP) -> usize
    where
        Self: MutableDescription,
        TP: Term,
    {
        let (h, g) = self.as_parts_mut();
        h.remove_all_with(g, p)
    }

    /// Add statement about the subject with given predicate and
    /// object,and remove any other statements about the subject
    /// with given predicate.
    #[inline]
    fn set<'s, TP, TO>(&'s mut self, p: &'s TP, o: &'s TO)
    where
        Self: MutableDescription,
        TP: Term,
        TO: Term,
    {
        let (h, g) = self.as_parts_mut();
        h.set(g, p, o)
    }
}

impl<D: Description> DescriptionExt for D {}

/// A macro to define custom[`Handle`], and [`Description`]
/// types quickly.
///
/// NOTE: generated code depends on `paste` and `sophia_api`
/// crates.
#[macro_export]
macro_rules! define_handle_and_description_types {
    (
        $(#[$houter:meta])*
        $H:ident;
        $(#[$douter:meta])*
        $D:ident;
        [
            $(
                $(#[$pouter:meta])*
                ($pm:ident, $p:expr, $PH:ident $(,$PD:ident)?);
            )*
        ]
    ) => {
        $crate::define_handle_type!(
            $(#[$houter])*
            $H;
            [
                $(
                    $(#[$pouter])*
                    ($pm, $p, $PH);
                )*
            ]
        );


        paste::paste! {
#[allow(unused_imports)]
pub use [<descr_impl_$D:snake>]::$D;

#[allow(unused_imports)]
mod [<descr_impl_$D:snake>] {
    use std::{
        borrow::{Borrow, BorrowMut},
        fmt::Debug,
        marker::PhantomData,
    };

    use $crate::model::{
        description::{Description, MutableDescription, SimpleDescription},
        graph::{InfallibleGraph, InfallibleMutableGraph},
        handle::{Handle, HandleExt, OwnedObjectHandleIterator},
        term::ArcTerm,
    };
    use sophia_api::{term::{FromTerm, Term}};

    use super::*;

    $(#[$douter])*
    #[derive(Debug)]
    pub struct $D<G, WG>
    where
        G: InfallibleGraph,
        WG: Borrow<G>,
    {
        handle: $H<ArcTerm>,
        wgraph: WG,
        _phantom: PhantomData<fn() -> G>
    }

    impl<G, WG> Clone for $D<G, WG>
    where
        G: InfallibleGraph,
        WG: Borrow<G> + Clone,
    {
        fn clone(&self) -> Self {
            Self {
                handle: self.handle.clone(),
                wgraph: self.wgraph.clone(),
                _phantom: PhantomData
            }
        }
    }

    impl<G, WG> Description for $D<G, WG>
    where
        G: InfallibleGraph,
        WG: Borrow<G> + Debug,
    {
        type Handle = $H<ArcTerm>;

        type Graph = G;

        #[inline]
        fn handle(&self) -> &Self::Handle {
            &self.handle
        }

        #[inline]
        fn graph(&self) -> &Self::Graph {
            self.wgraph.borrow()
        }
    }

    impl<G, WG> SimpleDescription for $D<G, WG>
    where
        G: InfallibleGraph,
        WG: Borrow<G> + Debug,
    {
        type WGraph = WG;

        #[inline]
        fn new(handle: Self::Handle, wgraph: Self::WGraph) -> Self {
            Self {
                handle,
                wgraph,
                _phantom: PhantomData
            }
        }

        #[inline]
        fn wgraph(&self) -> &Self::WGraph {
            &self.wgraph
        }
    }

    impl<G, WG> MutableDescription for $D<G, WG>
    where
        G: InfallibleMutableGraph,
        WG: BorrowMut<G> + Debug,
    {
        type MutGraph = G;

        #[inline]
        fn graph_mut(&mut self) -> &mut Self::Graph {
            self.wgraph.borrow_mut()
        }

        #[inline]
        fn as_parts_mut(&mut self) -> (&Self::Handle, &mut Self::Graph) {
            (&self.handle, self.wgraph.borrow_mut())
        }
    }

    impl<G, WG> $D<G, WG>
    where
        G: InfallibleGraph,
        WG: Borrow<G>,
    {
        /// Convert into parts.
        #[inline]
        pub fn into_parts(self) -> ($H<ArcTerm>, WG) {
            (self.handle, self.wgraph)
        }

        $(
            $(#[$pouter])*
            pub fn [<h_$pm>]<T2: Term + FromTerm>(&self) -> OwnedObjectHandleIterator<G, $PH<T2>> {
                HandleExt::get_all_handles(&self.handle, self.wgraph.borrow(), $p)
            }

            $(
                #[doc = "Get descriptions of `" $pm "`  objects."]
                pub fn $pm<'s>(&'s self) -> impl Iterator<Item = $PD<G, WG>> + 's
                where
                    WG: Clone + Debug,
                {
                    self.[<h_$pm>]::<ArcTerm>().map(|h: $PH<ArcTerm>| {
                        SimpleDescription::new(
                            h,
                            self.wgraph.clone(),
                        )
                    })
                }
            )?
        )*
    }
}

        }
    };
}

define_handle_and_description_types!(
    /// A struct for representing handle for any resource.
    HAny;
    /// A struct for representing description of any resource.
    DAny;
    []
);

#[cfg(test)]
#[allow(dead_code)]
mod test_define_descr_macro {
    use sophia_api::ns;

    use super::HAny;

    define_handle_and_description_types!(
        /// Test definition of a new subject handle type.
        HTest;
        DTest;
        [
            /// About.
            (about, &ns::rdf::about, HAny);

            /// Subject.
            (subject, &ns::rdf::subject, HAny);
        ]
    );
}

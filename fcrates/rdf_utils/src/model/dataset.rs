//! I define models for rdf datasets.
//!

use std::{convert::Infallible, fmt::Debug, ops::Deref};

use sophia_api::{
    dataset::DTerm,
    graph::adapter::DatasetGraph,
    prelude::{Dataset, MutableDataset},
    term::{GraphName, Term},
};

/// A trait for infallible datasets.
pub trait InfallibleDataset: Dataset<Error = Infallible> + Debug {
    /// Borrows one of the graphs of this dataset
    fn graph_view<T: Term>(&self, graph_name: GraphName<T>) -> DatasetGraph<&Self, T> {
        DatasetGraph::new(self, graph_name)
    }
}

impl<D> InfallibleDataset for D where D: Dataset<Error = Infallible> + Debug {}

/// A trait for infallible mutable datasets.
pub trait InfallibleMutableDataset:
    MutableDataset<Error = Infallible, MutationError = Infallible> + Debug
{
}

impl<D> InfallibleMutableDataset for D where
    D: MutableDataset<Error = Infallible, MutationError = Infallible> + Debug
{
}

/// Type of static dataset terms.
pub type DSTerm<D> = DTerm<'static, D>;

/// A dataset type to wrap around few common rust types, providing
/// dataset implementations for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CompatDataset<D>(pub D);

impl<D> Deref for CompatDataset<D> {
    type Target = D;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "compat-ecow")]
pub use ecow_dataset_impl::*;

#[cfg(feature = "compat-ecow")]
mod ecow_dataset_impl {
    use ecow::EcoVec;
    use sophia_api::{
        dataset::{CollectibleDataset, DQuadSource, MdResult},
        quad::{QBorrowTerm, Quad, Spog},
        source::{QuadSource, StreamError::SourceError, StreamResult},
        term::FromTerm,
    };

    use super::*;

    impl<Q: Quad> Dataset for CompatDataset<EcoVec<Q>> {
        type Quad<'x> = Spog<QBorrowTerm<'x, Q>> where Self: 'x;

        type Error = Infallible;

        #[inline]
        fn quads(&self) -> DQuadSource<Self> {
            self.0[..].quads()
        }
    }

    impl<T: Term + FromTerm + Clone> MutableDataset for CompatDataset<EcoVec<Spog<T>>> {
        type MutationError = Infallible;

        fn insert<TS, TP, TO, TG>(
            &mut self,
            s: TS,
            p: TP,
            o: TO,
            g: GraphName<TG>,
        ) -> std::result::Result<bool, <Self as MutableDataset>::MutationError>
        where
            TS: Term,
            TP: Term,
            TO: Term,
            TG: Term,
        {
            self.0.push((
                [s.into_term(), p.into_term(), o.into_term()],
                g.map(Term::into_term),
            ));
            Ok(true)
        }

        fn remove<TS, TP, TO, TG>(
            &mut self,
            s: TS,
            p: TP,
            o: TO,
            g: GraphName<TG>,
        ) -> MdResult<Self, bool>
        where
            TS: Term,
            TP: Term,
            TO: Term,
            TG: Term,
        {
            let s = s.borrow_term();
            let p = p.borrow_term();
            let o = o.borrow_term();
            let g = g.as_ref().map(|gn| gn.borrow_term());
            match self.0.iter().position(|q| q.matched_by([s], [p], [o], [g])) {
                None => Ok(false),
                Some(i) => {
                    self.0.remove(i);
                    Ok(true)
                }
            }
        }
    }

    impl<T: Term + FromTerm + Clone> CollectibleDataset for CompatDataset<EcoVec<Spog<T>>> {
        fn from_quad_source<QS: QuadSource>(
            mut quads: QS,
        ) -> StreamResult<Self, QS::Error, Self::Error> {
            let min_cap = quads.size_hint_quads().0;
            let mut v = EcoVec::with_capacity(min_cap);
            quads
                .for_each_quad(|q| {
                    v.push((
                        [q.s().into_term(), q.p().into_term(), q.o().into_term()],
                        q.g().map(Term::into_term),
                    ))
                })
                .map_err(SourceError)?;
            Ok(Self(v))
        }
    }

    /// Type of dataset backed by an ecovec.
    pub type EcoDataset<Q> = CompatDataset<EcoVec<Q>>;
}

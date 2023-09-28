//! I define definitions and implementations for attribute matcher services.
//!

use std::{borrow::Borrow, fmt::Debug};

use dyn_clone::{clone_trait_object, DynClone};
use dyn_problem::{ProbFuture, Problem};
use rdf_utils::model::graph::InfallibleGraph;
use sophia_api::term::Term;
use tower::Service;

use crate::model::context::DContext;

pub mod impl_;

/// Struct representing an attribute match request.
#[derive(Debug, Clone)]
pub struct AttributeMatchRequest<T, G, WG>
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
    /// Value of the attribute to be matched.
    pub value: T,

    /// Resource access context description,
    /// against which attribute match have to be resolved.
    pub context: DContext<G, WG>,
}

/// A trait for services that resolve attribute match against resource access context.
pub trait AttributeMatchService<T, G, WG>:
    Service<
        AttributeMatchRequest<T, G, WG>,
        Response = bool,
        Error = Problem,
        Future = ProbFuture<'static, bool>,
    > + DynClone
    + Send
    + Sync
    + 'static
where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
}

impl<T, G, WG, S> AttributeMatchService<T, G, WG> for S
where
    S: Service<
            AttributeMatchRequest<T, G, WG>,
            Response = bool,
            Error = Problem,
            Future = ProbFuture<'static, bool>,
        > + DynClone
        + Send
        + Sync
        + 'static,
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
{
}

/// Type alias for type erased attribute match services.
pub type BoxedAttributeMatchService<T, G, WG> = Box<dyn AttributeMatchService<T, G, WG>>;

clone_trait_object!(<T, G, WG> AttributeMatchService<T, G, WG> where
    T: Term,
    G: InfallibleGraph,
    WG: Borrow<G> + Debug,
);

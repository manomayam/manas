//! I define an dynsyn models for rdf concepts.
//!

//TODO bring to sanity.

use rio_api::model::{Quad as RioQuad, Term as RioTerm, Triple as RioTriple};
use sophia_api::{
    quad::{QBorrowTerm, Quad, Spog},
    term::{BnodeId, CmpTerm, GraphName, IriRef, LanguageTag, Term, TermKind, VarName},
    triple::{TBorrowTerm, Triple},
    MownStr,
};
#[cfg(feature = "jsonld")]
use sophia_jsonld::RdfTerm;
use sophia_rio::model::Trusted;

/// An enum of different variants of borrow-terms produced by different parsers.
#[derive(Debug, Clone, Copy)]
pub(crate) enum InnerBorrowTerm<'a> {
    /// Trusted rio terms variant.
    Rio(CmpTerm<Trusted<RioTerm<'a>>>),
    #[cfg(feature = "jsonld")]
    RdfTerm(&'a RdfTerm),
}

impl<'a> From<Trusted<RioTerm<'a>>> for InnerBorrowTerm<'a> {
    #[inline]
    fn from(value: Trusted<RioTerm<'a>>) -> Self {
        Self::Rio(CmpTerm(value))
    }
}

impl<'a> From<CmpTerm<Trusted<RioTerm<'a>>>> for InnerBorrowTerm<'a> {
    #[inline]
    fn from(value: CmpTerm<Trusted<RioTerm<'a>>>) -> Self {
        Self::Rio(value)
    }
}

#[cfg(feature = "jsonld")]
impl<'a> From<&'a RdfTerm> for InnerBorrowTerm<'a> {
    #[inline]
    fn from(value: &'a RdfTerm) -> Self {
        Self::RdfTerm(value)
    }
}

/// Type of borrow-terms produced by dynsyn parsers.
#[derive(Debug, Clone, Copy)]
pub struct DynSynBorrowTerm<'a>(pub(crate) InnerBorrowTerm<'a>);

impl<'a> Term for DynSynBorrowTerm<'a> {
    type BorrowTerm<'x> = Self where Self: 'x;

    #[inline]
    fn kind(&self) -> TermKind {
        match self.0 {
            InnerBorrowTerm::Rio(v) => v.kind(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.kind(),
        }
    }

    #[inline]
    fn borrow_term(&self) -> Self::BorrowTerm<'_> {
        *self
    }

    #[inline]
    fn iri(&self) -> Option<IriRef<MownStr>> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.iri(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.iri(),
        }
    }

    #[inline]
    fn bnode_id(&self) -> Option<BnodeId<MownStr>> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.bnode_id(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.bnode_id(),
        }
    }

    #[inline]
    fn lexical_form(&self) -> Option<MownStr> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.lexical_form(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.lexical_form(),
        }
    }

    #[inline]
    fn datatype(&self) -> Option<IriRef<MownStr>> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.datatype(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.datatype(),
        }
    }

    #[inline]
    fn language_tag(&self) -> Option<LanguageTag<MownStr>> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.language_tag(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.language_tag(),
        }
    }

    #[inline]
    fn variable(&self) -> Option<VarName<MownStr>> {
        match &self.0 {
            InnerBorrowTerm::Rio(v) => v.variable(),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v.variable(),
        }
    }

    #[inline]
    fn triple(&self) -> Option<[Self::BorrowTerm<'_>; 3]> {
        match self.0 {
            InnerBorrowTerm::Rio(v) => v
                .triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v
                .triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
        }
    }

    #[inline]
    fn to_triple(self) -> Option<[Self; 3]>
    where
        Self: Sized,
    {
        match self.0 {
            InnerBorrowTerm::Rio(v) => v
                .to_triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
            #[cfg(feature = "jsonld")]
            InnerBorrowTerm::RdfTerm(v) => v
                .to_triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
        }
    }
}

/// An enum of different variants of terms produced by different parsers.
#[derive(Debug, Clone)]
pub(crate) enum InnerTerm<'a> {
    /// Trusted rio terms variant.
    Rio(CmpTerm<Trusted<RioTerm<'a>>>),
    #[cfg(feature = "jsonld")]
    RdfTerm(RdfTerm),
}

impl<'a> From<Trusted<RioTerm<'a>>> for InnerTerm<'a> {
    #[inline]
    fn from(value: Trusted<RioTerm<'a>>) -> Self {
        Self::Rio(CmpTerm(value))
    }
}

impl<'a> From<CmpTerm<Trusted<RioTerm<'a>>>> for InnerTerm<'a> {
    #[inline]
    fn from(value: CmpTerm<Trusted<RioTerm<'a>>>) -> Self {
        Self::Rio(value)
    }
}

#[cfg(feature = "jsonld")]
impl<'a> From<RdfTerm> for InnerTerm<'a> {
    #[inline]
    fn from(value: RdfTerm) -> Self {
        Self::RdfTerm(value)
    }
}

/// Type of terms produced by dynsyn parsers.
#[derive(Debug, Clone)]
pub struct DynSynTerm<'a>(pub(crate) InnerTerm<'a>);

impl<'a> Term for DynSynTerm<'a> {
    type BorrowTerm<'x> = DynSynBorrowTerm<'x> where Self: 'x;

    #[inline]
    fn kind(&self) -> TermKind {
        match &self.0 {
            InnerTerm::Rio(v) => v.kind(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.kind(),
        }
    }

    #[inline]
    fn borrow_term(&self) -> Self::BorrowTerm<'_> {
        match &self.0 {
            InnerTerm::Rio(v) => DynSynBorrowTerm((*v).into()),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => DynSynBorrowTerm(v.into()),
        }
    }

    #[inline]
    fn iri(&self) -> Option<IriRef<MownStr>> {
        match &self.0 {
            InnerTerm::Rio(v) => v.iri(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.iri(),
        }
    }

    #[inline]
    fn bnode_id(&self) -> Option<BnodeId<MownStr>> {
        match &self.0 {
            InnerTerm::Rio(v) => v.bnode_id(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.bnode_id(),
        }
    }

    #[inline]
    fn lexical_form(&self) -> Option<MownStr> {
        match &self.0 {
            InnerTerm::Rio(v) => v.lexical_form(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.lexical_form(),
        }
    }

    #[inline]
    fn datatype(&self) -> Option<IriRef<MownStr>> {
        match &self.0 {
            InnerTerm::Rio(v) => v.datatype(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.datatype(),
        }
    }

    #[inline]
    fn language_tag(&self) -> Option<LanguageTag<MownStr>> {
        match &self.0 {
            InnerTerm::Rio(v) => v.language_tag(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.language_tag(),
        }
    }

    #[inline]
    fn variable(&self) -> Option<VarName<MownStr>> {
        match &self.0 {
            InnerTerm::Rio(v) => v.variable(),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v.variable(),
        }
    }

    #[inline]
    fn triple(&self) -> Option<[Self::BorrowTerm<'_>; 3]> {
        match &self.0 {
            InnerTerm::Rio(v) => v
                .triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v
                .triple()
                .map(|triple| triple.map(|term| DynSynBorrowTerm(term.into()))),
        }
    }

    #[inline]
    fn to_triple(self) -> Option<[Self; 3]>
    where
        Self: Sized,
    {
        match self.0 {
            InnerTerm::Rio(v) => v
                .to_triple()
                .map(|triple| triple.map(|term| DynSynTerm(term.into()))),
            #[cfg(feature = "jsonld")]
            InnerTerm::RdfTerm(v) => v
                .to_triple()
                .map(|triple| triple.map(|term| DynSynTerm(term.into()))),
        }
    }
}

/// An enum of different variants of triples produced by different parsers.
#[derive(Debug, Clone)]
pub(crate) enum InnerTriple<'a> {
    /// Trusted rio triple variant.
    Rio(Trusted<RioTriple<'a>>),
}

impl<'a> From<Trusted<RioTriple<'a>>> for InnerTriple<'a> {
    #[inline]
    fn from(value: Trusted<RioTriple<'a>>) -> Self {
        Self::Rio(value)
    }
}

/// Type of triples produced by dynsyn parsers.
#[derive(Debug, Clone)]
pub struct DynSynTriple<'a>(pub(crate) InnerTriple<'a>);

impl<'a> Triple for DynSynTriple<'a> {
    type Term = DynSynTerm<'a>;

    #[inline]
    fn s(&self) -> TBorrowTerm<Self> {
        match self.0 {
            InnerTriple::Rio(v) => DynSynBorrowTerm(v.s().into()),
        }
    }

    #[inline]
    fn p(&self) -> TBorrowTerm<Self> {
        match self.0 {
            InnerTriple::Rio(v) => DynSynBorrowTerm(v.p().into()),
        }
    }

    #[inline]
    fn o(&self) -> TBorrowTerm<Self> {
        match self.0 {
            InnerTriple::Rio(v) => DynSynBorrowTerm(v.o().into()),
        }
    }

    #[inline]
    fn to_spo(self) -> [Self::Term; 3] {
        match self.0 {
            InnerTriple::Rio(v) => v.to_spo().map(|term| DynSynTerm(term.into())),
        }
    }
}

/// An enum of different variants of quads produced by different parsers.
#[derive(Debug, Clone)]
pub(crate) enum InnerQuad<'a> {
    /// Trusted rio quad variant.
    Rio(Trusted<rio_api::model::Quad<'a>>),
    #[cfg(feature = "jsonld")]
    Jsonld(Spog<RdfTerm>),
}

impl<'a> From<Trusted<RioQuad<'a>>> for InnerQuad<'a> {
    #[inline]
    fn from(value: Trusted<RioQuad<'a>>) -> Self {
        Self::Rio(value)
    }
}

#[cfg(feature = "jsonld")]
impl<'a> From<Spog<RdfTerm>> for InnerQuad<'a> {
    #[inline]
    fn from(value: Spog<RdfTerm>) -> Self {
        Self::Jsonld(value)
    }
}

/// Type of quads produced by dynsyn parsers.
#[derive(Debug, Clone)]
pub struct DynSynQuad<'a>(pub(crate) InnerQuad<'a>);

impl<'a> Quad for DynSynQuad<'a> {
    type Term = DynSynTerm<'a>;

    #[inline]
    fn s(&self) -> QBorrowTerm<Self> {
        match &self.0 {
            InnerQuad::Rio(v) => DynSynBorrowTerm(v.s().into()),
            #[cfg(feature = "jsonld")]
            InnerQuad::Jsonld(v) => DynSynBorrowTerm(v.s().into()),
        }
    }

    #[inline]
    fn p(&self) -> QBorrowTerm<Self> {
        match &self.0 {
            InnerQuad::Rio(v) => DynSynBorrowTerm(v.p().into()),
            #[cfg(feature = "jsonld")]
            InnerQuad::Jsonld(v) => DynSynBorrowTerm(v.p().into()),
        }
    }

    #[inline]
    fn o(&self) -> QBorrowTerm<Self> {
        match &self.0 {
            InnerQuad::Rio(v) => DynSynBorrowTerm(v.o().into()),
            #[cfg(feature = "jsonld")]
            InnerQuad::Jsonld(v) => DynSynBorrowTerm(v.o().into()),
        }
    }

    #[inline]
    fn g(&self) -> GraphName<QBorrowTerm<Self>> {
        match &self.0 {
            InnerQuad::Rio(v) => v.g().map(|gn| DynSynBorrowTerm(gn.into())),
            #[cfg(feature = "jsonld")]
            InnerQuad::Jsonld(v) => v.g().map(|gn| DynSynBorrowTerm(gn.into())),
        }
    }

    #[inline]
    fn to_spog(self) -> Spog<Self::Term> {
        match self.0 {
            InnerQuad::Rio(v) => {
                let spog = v.to_spog();
                (
                    spog.0.map(|term| DynSynTerm(term.into())),
                    spog.1.map(|term| DynSynTerm(term.into())),
                )
            }
            #[cfg(feature = "jsonld")]
            InnerQuad::Jsonld(v) => {
                let spog = v.to_spog();
                (
                    spog.0.map(|term| DynSynTerm(term.into())),
                    spog.1.map(|term| DynSynTerm(term.into())),
                )
            }
        }
    }
}

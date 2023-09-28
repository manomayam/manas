//! I define models for rdf terms.
//!

use std::{borrow::Borrow, fmt::Debug, rc::Rc, sync::Arc};

use once_cell::sync::Lazy;
use sophia_api::{
    ns::{rdf, xsd},
    prelude::IriRef,
    term::{BnodeId, FromTerm, LanguageTag, Term, TermKind, TryFromTerm, VarName},
    triple::Triple,
    MownStr,
};

static RDF_LANG_STRING: Lazy<Box<str>> =
    Lazy::new(|| rdf::langString.iri().unwrap().unwrap().into());

static XSD_INTEGER: Lazy<Box<str>> = Lazy::new(|| xsd::integer.iri().unwrap().unwrap().into());

/// A basic implementation of [`Term`].
#[derive(Debug, Clone)]
pub enum BasicTerm<S: Borrow<str>> {
    /// An [RDF IRI](https://www.w3.org/TR/rdf11-concepts/#section-IRIs)
    Iri(IriRef<S>),
    /// An RDF [blank node](https://www.w3.org/TR/rdf11-concepts/#section-blank-nodes)
    BlankNode(BnodeId<S>),
    /// An RDF [literal](https://www.w3.org/TR/rdf11-concepts/#section-Graph-Literal)
    LiteralDatatype(S, IriRef<S>),
    /// An RDF [language-tagged string](https://www.w3.org/TR/rdf11-concepts/#dfn-language-tagged-string)
    LiteralLanguage(S, LanguageTag<S>),
    /// An RDF-star [quoted triple](https://www.w3.org/2021/12/rdf-star.html#dfn-quoted)
    Triple(Box<[Self; 3]>),
    /// A SPARQL or Notation3 variable
    Variable(VarName<S>),
}

use BasicTerm::*;

impl<S: Borrow<str> + Debug> Term for BasicTerm<S> {
    type BorrowTerm<'x> = &'x Self where Self: 'x;

    fn kind(&self) -> TermKind {
        match self {
            Iri(_) => TermKind::Iri,
            BlankNode(_) => TermKind::BlankNode,
            LiteralDatatype(..) | LiteralLanguage(..) => TermKind::Literal,
            Triple(_) => TermKind::Triple,
            Variable(_) => TermKind::Variable,
        }
    }
    fn iri(&self) -> Option<IriRef<MownStr>> {
        if let Iri(iri) = self {
            Some(IriRef::new_unchecked(MownStr::from_str(iri.as_str())))
        } else {
            None
        }
    }
    fn bnode_id(&self) -> Option<BnodeId<MownStr>> {
        if let BlankNode(bnid) = self {
            Some(BnodeId::new_unchecked(MownStr::from_str(bnid.as_str())))
        } else {
            None
        }
    }
    fn lexical_form(&self) -> Option<MownStr> {
        match self {
            LiteralDatatype(val, _) | LiteralLanguage(val, _) => Some(MownStr::from(val.borrow())),
            _ => None,
        }
    }
    fn datatype(&self) -> Option<IriRef<MownStr>> {
        match self {
            LiteralDatatype(_, iri) => Some(IriRef::new_unchecked(MownStr::from_str(iri.as_str()))),
            LiteralLanguage(..) => Some(IriRef::new_unchecked(MownStr::from_str(&RDF_LANG_STRING))),
            _ => None,
        }
    }
    fn language_tag(&self) -> Option<LanguageTag<MownStr>> {
        if let LiteralLanguage(_, tag) = self {
            Some(LanguageTag::new_unchecked(MownStr::from_str(tag.as_str())))
        } else {
            None
        }
    }
    fn variable(&self) -> Option<VarName<MownStr>> {
        if let Variable(name) = self {
            Some(VarName::new_unchecked(MownStr::from_str(name.as_str())))
        } else {
            None
        }
    }
    fn triple(&self) -> Option<[Self::BorrowTerm<'_>; 3]> {
        if let Triple(triple) = self {
            let [s, p, o] = triple.as_ref();
            Some([s, p, o])
        } else {
            None
        }
    }
    fn to_triple(self) -> Option<[Self; 3]> {
        if let Triple(triple) = self {
            Some(*triple)
        } else {
            None
        }
    }
    fn borrow_term(&self) -> Self::BorrowTerm<'_> {
        self
    }
}

#[inline]
fn into_other_str<S: for<'a> From<&'a str>>(m: MownStr) -> S {
    m.as_ref().into()
}

impl<S> FromTerm for BasicTerm<S>
where
    S: Borrow<str> + for<'a> From<&'a str>,
{
    fn from_term<T: Term>(term: T) -> Self {
        match term.kind() {
            TermKind::Iri => BasicTerm::Iri(term.iri().unwrap().map_unchecked(into_other_str)),
            TermKind::BlankNode => {
                BasicTerm::BlankNode(term.bnode_id().unwrap().map_unchecked(into_other_str))
            }
            TermKind::Literal => {
                let lex = into_other_str(term.lexical_form().unwrap());
                if let Some(tag) = term.language_tag() {
                    let tag = tag.map_unchecked(into_other_str);
                    BasicTerm::LiteralLanguage(lex, tag)
                } else {
                    let dt = term.datatype().unwrap().map_unchecked(into_other_str);
                    BasicTerm::LiteralDatatype(lex, dt)
                }
            }
            TermKind::Triple => {
                let t = term.triple().unwrap();
                BasicTerm::Triple(Box::new([
                    Self::from_term(t.s()),
                    Self::from_term(t.p()),
                    Self::from_term(t.o()),
                ]))
            }
            TermKind::Variable => {
                BasicTerm::Variable(term.variable().unwrap().map_unchecked(into_other_str))
            }
        }
    }
}

impl<S: Borrow<str> + for<'a> From<&'a str>> TryFromTerm for BasicTerm<S> {
    type Error = std::convert::Infallible;

    fn try_from_term<T: Term>(term: T) -> Result<Self, Self::Error> {
        Ok(Self::from_term(term))
    }
}

impl<S: Borrow<str> + Debug, T: Term> PartialEq<T> for BasicTerm<S> {
    fn eq(&self, other: &T) -> bool {
        Term::eq(self, other.borrow_term())
    }
}

impl<S: Borrow<str> + Debug> Eq for BasicTerm<S> {}

impl<S: Borrow<str> + Debug> std::hash::Hash for BasicTerm<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Term::hash(self, state)
    }
}

/// A [`BasicTerm`] with [`&str`] as inner data.
pub type RefTerm<'a> = BasicTerm<&'a str>;

/// A [`BasicTerm`] with [`MownStr`] as inner data.
pub type MownTerm<'a> = BasicTerm<MownStr<'a>>;

/// A [`BasicTerm`] with [`String`] as inner data.
pub type OwnedTerm = BasicTerm<String>;

/// A [`BasicTerm`] with [`Rc`] 'ed str as inner data.
pub type RcTerm = BasicTerm<Rc<str>>;

/// A [`BasicTerm`] with [`Arc`] 'ed str as inner data.
pub type ArcTerm = BasicTerm<Arc<str>>;

/// Iri type with arc-str content.
pub type ArcIri = sophia_api::prelude::Iri<Arc<str>>;

/// IriRef type with arc-str content.
pub type ArcIriRef = sophia_api::prelude::IriRef<Arc<str>>;

/// BnodeId type with arc-str content.
pub type ArcBlankNode = BnodeId<Arc<str>>;

/// A term type to wrap around few common rust types, providing
/// term implementations for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompatTerm<T>(pub T);

impl Term for CompatTerm<u64> {
    type BorrowTerm<'x> = Self
    where
        Self: 'x;

    #[inline]
    fn kind(&self) -> TermKind {
        TermKind::Literal
    }

    #[inline]
    fn borrow_term(&self) -> Self::BorrowTerm<'_> {
        *self
    }

    #[inline]
    fn lexical_form(&self) -> Option<MownStr> {
        Some(MownStr::from(format!("{}", self.0)))
    }

    #[inline]
    fn datatype(&self) -> Option<IriRef<MownStr>> {
        Some(IriRef::new_unchecked(MownStr::from_str(&XSD_INTEGER)))
    }

    #[inline]
    fn language_tag(&self) -> Option<LanguageTag<MownStr>> {
        None
    }
}

#[cfg(feature = "compat-chrono")]
mod chrono_terms_impl {
    use chrono::{DateTime, Utc};

    use super::*;

    static XSD_DATE_TIME: Lazy<Box<str>> =
        Lazy::new(|| xsd::dateTime.iri().unwrap().unwrap().into());

    impl Term for CompatTerm<DateTime<Utc>> {
        type BorrowTerm<'x> = &'x Self
            where
            Self: 'x;

        #[inline]
        fn kind(&self) -> TermKind {
            TermKind::Literal
        }

        #[inline]
        fn borrow_term(&self) -> Self::BorrowTerm<'_> {
            self
        }

        #[inline]
        fn lexical_form(&self) -> Option<MownStr> {
            Some(MownStr::from(self.0.to_rfc3339()))
        }

        #[inline]
        fn datatype(&self) -> Option<IriRef<MownStr>> {
            Some(IriRef::new_unchecked(MownStr::from_str(&XSD_DATE_TIME)))
        }

        #[inline]
        fn language_tag(&self) -> Option<LanguageTag<MownStr>> {
            None
        }
    }
}

#[cfg(feature = "compat-iri-string")]
mod iri_string_terms_impl {
    use iri_string::types::{IriReferenceStr, UriReferenceStr};
    use sophia_api::term::IriRef;

    use super::*;
    impl<'a> CompatTerm<&'a UriReferenceStr> {
        /// Get [`CompatTerm`] for uri references.
        #[inline]
        pub fn from_uri_ref<U: AsRef<UriReferenceStr>>(uri: &'a U) -> Self {
            CompatTerm(uri.as_ref())
        }
    }

    impl<'a> Term for CompatTerm<&'a UriReferenceStr> {
        type BorrowTerm<'x> = Self
        where
            Self: 'x;

        #[inline]
        fn kind(&self) -> TermKind {
            TermKind::Iri
        }

        #[inline]
        fn borrow_term(&self) -> Self::BorrowTerm<'_> {
            *self
        }

        #[inline]
        fn iri(&self) -> Option<IriRef<MownStr>> {
            Some(IriRef::new_unchecked(MownStr::from_str(self.0.as_str())))
        }
    }

    impl<'a> Term for CompatTerm<&'a IriReferenceStr> {
        type BorrowTerm<'x> = Self
        where
            Self: 'x;

        #[inline]
        fn kind(&self) -> TermKind {
            TermKind::Iri
        }

        #[inline]
        fn borrow_term(&self) -> Self::BorrowTerm<'_> {
            *self
        }

        #[inline]
        fn iri(&self) -> Option<IriRef<MownStr>> {
            Some(IriRef::new_unchecked(MownStr::from_str(self.0.as_str())))
        }
    }
}

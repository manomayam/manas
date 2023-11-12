//! This crate provides types and functionality for
//! representing and handling web ids.
//!

#![warn(missing_docs)]
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![deny(unused_qualifications)]

use std::{borrow::Borrow, fmt::Display, ops::Deref, str::FromStr};

use http_uri::{HttpUri, InvalidHttpUri};
use iri_string::types::UriStr;

#[cfg(feature = "profile-req-agent")]
pub mod profile_req_agent;

#[cfg(feature = "invariants")]
pub mod invariant;

/// A struct for representing valid webids.
///
/// A WebID is an HTTP URI which refers to an Agent (Person, Organization, Group, Device, etc.).
/// See: <https://www.w3.org/2005/Incubator/webid/spec/identity/>
///
/// This struct is cheaply clonable.   
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct WebId(HttpUri);

impl std::fmt::Debug for WebId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for WebId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebId({})", self.0.as_str())
    }
}

impl Deref for WebId {
    type Target = HttpUri;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<HttpUri> for WebId {
    #[inline]
    fn borrow(&self) -> &HttpUri {
        &self.0
    }
}

impl From<WebId> for HttpUri {
    #[inline]
    fn from(value: WebId) -> Self {
        value.0
    }
}

impl From<HttpUri> for WebId {
    #[inline]
    fn from(value: HttpUri) -> Self {
        Self(value)
    }
}

impl AsRef<UriStr> for WebId {
    #[inline]
    fn as_ref(&self) -> &UriStr {
        self.0.as_ref()
    }
}

impl AsRef<str> for WebId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<&str> for WebId {
    type Error = InvalidWebId;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl FromStr for WebId {
    type Err = InvalidWebId;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.try_into()?))
    }
}

/// An error type for invalid webids.
#[derive(Debug, thiserror::Error)]
#[error("Invalid webid.")]
pub struct InvalidWebId(#[from] InvalidHttpUri);

#[cfg(feature = "sophia")]
mod term_impl {
    use sophia_api::term::{IriRef, Term, TermKind};

    use crate::WebId;

    impl Term for WebId {
        type BorrowTerm<'x> = &'x Self
        where
            Self: 'x;

        #[inline]
        fn kind(&self) -> TermKind {
            self.0.kind()
        }

        #[inline]
        fn borrow_term(&self) -> Self::BorrowTerm<'_> {
            self
        }

        #[inline]
        fn iri(&self) -> Option<IriRef<sophia_api::MownStr>> {
            self.0.iri()
        }
    }
}

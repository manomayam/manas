//! I define statics for recommonded file extensions for different concrete rdf syntaxes.
//!

use std::{borrow::Cow, ffi::OsStr, fmt::Display, ops::Deref, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct that denotes a file extension
pub struct FileExtension(pub Cow<'static, str>);

impl Deref for FileExtension {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for FileExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<Cow<'static, str>>> From<T> for FileExtension {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

impl FileExtension {
    /// Get extension from given path.
    pub fn from_path(path: &Path) -> Option<Self> {
        Some(Self::from(
            path.extension().and_then(OsStr::to_str)?.to_string(),
        ))
    }

    /// Get extension from given path str.
    pub fn from_path_str(path_str: &str) -> Option<Self> {
        Self::from_path(Path::new(path_str))
    }

    /// Get extension from given static path str
    pub const fn from_static(v: &'static str) -> Self {
        Self(Cow::Borrowed(v))
    }
}

/// .html
pub const HTML: FileExtension = FileExtension::from_static("html");

/// .json
pub const JSON: FileExtension = FileExtension::from_static("json");

/// .jsonld
pub const JSONLD: FileExtension = FileExtension::from_static("jsonld");

/// .n3
pub const N3: FileExtension = FileExtension::from_static("n3");

/// .nq
pub const NQ: FileExtension = FileExtension::from_static("nq");

/// .nquads
pub const NQUADS: FileExtension = FileExtension::from_static("nquads");

/// .nt
pub const NT: FileExtension = FileExtension::from_static("nt");

/// .ntriples
pub const NTRIPLES: FileExtension = FileExtension::from_static("ttl");

/// .omn
pub const OMN: FileExtension = FileExtension::from_static("omn");

/// .owl
pub const OWL: FileExtension = FileExtension::from_static("owl");

/// .owx
pub const OWX: FileExtension = FileExtension::from_static("owx");

/// .rdf
pub const RDF: FileExtension = FileExtension::from_static("rdf");

/// .rdfxml
pub const RDFXML: FileExtension = FileExtension::from_static("rdfxml");

/// .trig
pub const TRIG: FileExtension = FileExtension::from_static("trig");

/// .ttl
pub const TTL: FileExtension = FileExtension::from_static("ttl");

/// .turtle
pub const TURTLE: FileExtension = FileExtension::from_static("turtle");

/// .xhtml
pub const XHTML: FileExtension = FileExtension::from_static("xhtml");

//! I define rust models for concepts and entities involved
//! in access control over resources in solid storages.
//! I also provide few default implementations.
//!

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use acp::model::access_mode::{HAccessMode, H_APPEND, H_CONTROL, H_READ, H_WRITE};
use http_headers::wac_allow::{AccessMode, AccessParam, PermissionGroup, WacAllow};
use manas_authentication::common::credentials::RequestCredentials;
use manas_space::resource::{operation::SolidResourceOperation, uri::SolidResourceUri};
use once_cell::sync::Lazy;
use rdf_utils::model::{handle::Handle, term::ArcTerm};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use sophia_api::term::Term;
use typed_record::TypedRecordKey;
use vec1::Vec1;

pub mod pdp;
pub mod pep;
pub mod prp;

/// A struct to represent an operation to perform, and
/// justification.
#[derive(Debug, Clone, Serialize)]
pub struct JustifiedOperation {
    /// Resource operation.
    #[serde(rename = "operation", serialize_with = "serialize_op")]
    pub op: SolidResourceOperation,

    /// Justification.
    pub why: Cow<'static, str>,
}

fn serialize_op<S: Serializer>(op: &SolidResourceOperation, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(op.label)
}

/// A struct to represent list of justified operations over a resource for an action.
#[derive(Debug, Clone)]
pub struct ActionOpList {
    /// Target resource uri.
    pub on: SolidResourceUri,

    /// List of justified operations.
    pub ops: Vec<JustifiedOperation>,
}

/// Type of access grant set.
pub type AccessGrantSet = HashSet<HAccessMode<ArcTerm>>;

/// A struct to represent authorization.
#[derive(Debug, Clone, Serialize)]
pub struct Authorization<C: RequestCredentials> {
    /// Action target resource.
    pub target: SolidResourceUri,

    /// Credentials to which this authorization apply.
    pub credentials: C,

    #[serde(serialize_with = "serialize_grants")]
    /// Granted access modes.
    pub grants: AccessGrantSet,
}

fn serialize_grants<S: Serializer>(grants: &AccessGrantSet, s: S) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(grants.len()))?;
    for mode in grants {
        if let Some(mode_iri) = mode.as_term().iri() {
            seq.serialize_element(mode_iri.as_str())?;
        }
    }
    seq.end()
}

static WAC_ALLOW_ACCESS_MODE_MAP: Lazy<HashMap<HAccessMode<ArcTerm>, AccessMode>> =
    Lazy::new(|| {
        [
            (&H_READ, AccessMode::READ),
            (&H_APPEND, AccessMode::APPEND),
            (&H_WRITE, AccessMode::WRITE),
            (&H_CONTROL, AccessMode::CONTROL),
        ]
        .into_iter()
        .map(|(h, m)| (h.clone().map_term(), m))
        .collect()
    });

impl<C: RequestCredentials> Authorization<C> {
    /// Get equivalent wac allow.
    pub fn to_wac_allow(&self) -> WacAllow {
        // TODO seek clarity in context of acp.
        let permission_group = if self.credentials.of_agent().is_some() {
            PermissionGroup::USER
        } else {
            PermissionGroup::PUBLIC
        };

        let access_modes = self
            .grants
            .iter()
            // TODO allow custom modes too.
            .filter_map(|h| WAC_ALLOW_ACCESS_MODE_MAP.get(h).cloned())
            .collect();

        WacAllow {
            access_params: vec![AccessParam {
                access_modes,
                permission_group,
            }],
        }
    }
}

/// A type to represent resolved access control.
#[derive(Debug, Clone, Serialize)]
pub enum ResolvedAccessControl<C: RequestCredentials> {
    /// Allow the action
    Allow {
        /// Authorization.
        authorization: Authorization<C>,
    },

    /// Deny the action.
    Deny {
        /// Authorization.
        authorization: Authorization<C>,

        /// Denied op list.
        denied_ops: Vec1<JustifiedOperation>,
    },
}

impl<C: RequestCredentials> ResolvedAccessControl<C> {
    /// Get if action is allowed.
    #[inline]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    /// Get the resolved authorization.
    #[inline]
    pub fn authorization(&self) -> &Authorization<C> {
        match self {
            ResolvedAccessControl::Allow { authorization: why } => why,
            ResolvedAccessControl::Deny {
                authorization: why, ..
            } => why,
        }
    }
}

/// A typed record key for resolved access control.
#[derive(Debug, Clone)]
pub struct KResolvedAccessControl<C: RequestCredentials> {
    _phantom: PhantomData<C>,
}

impl<C: RequestCredentials> TypedRecordKey for KResolvedAccessControl<C> {
    type Value = ResolvedAccessControl<C>;
}

/// A typed record key for resolved access control over host
/// resource.
#[derive(Debug, Clone)]
pub struct KResolvedHostAccessControl<C: RequestCredentials> {
    _phantom: PhantomData<C>,
}

impl<C: RequestCredentials> TypedRecordKey for KResolvedHostAccessControl<C> {
    type Value = ResolvedAccessControl<C>;
}

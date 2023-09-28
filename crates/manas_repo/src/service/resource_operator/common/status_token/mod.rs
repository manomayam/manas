//! I define traits for types for resource status tokens.
//! These tokens can be used for defining operations with type
//! driven contracts over resource status.
//!

use std::{fmt::Debug, sync::Arc};

use manas_http::representation::metadata::RepresentationMetadata;
use manas_space::resource::{slot::SolidResourceSlot, uri::SolidResourceUri};

use crate::Repo;

pub mod impl_;

/// A base trait for repo resource status token base.
pub trait RepoResourceStatusTokenBase: Debug {
    /// Type of the repo.
    type Repo: Repo;

    /// Get the repo context.
    fn repo_context(&self) -> &Arc<<Self::Repo as Repo>::Context>;
}

/// A trait to represent existing, but non-represented
/// resource status tokens.
pub trait ExistingNonRepresentedResourceToken: RepoResourceStatusTokenBase {
    /// Get slot of the existing resource.
    fn slot(&self) -> &SolidResourceSlot<<Self::Repo as Repo>::StSpace>;
}

/// A trait to represent existing represented resource status
/// tokens.
pub trait ExistingRepresentedResourceToken: RepoResourceStatusTokenBase {
    /// Get slot of the represented resource.
    fn slot(&self) -> &SolidResourceSlot<<Self::Repo as Repo>::StSpace>;

    /// Get representation validators.
    fn rep_validators(&self) -> RepresentationMetadata;
}

/// A trait to represent non-existing, but mutex existing
/// resource status tokens.
pub trait NonExistingMutexExistingResourceToken: RepoResourceStatusTokenBase {
    /// Get uri of the non existing target resource.
    fn uri(&self) -> &SolidResourceUri;

    /// Get slot of the existing mutex resource.
    fn mutex_slot(&self) -> &SolidResourceSlot<<Self::Repo as Repo>::StSpace>;
}

/// A trait to represent non-existing, mutex non-existing
/// resource status token.
pub trait NonExistingMutexNonExistingResourceToken: RepoResourceStatusTokenBase {
    /// Get uri of the non existing target resource.
    fn uri(&self) -> &SolidResourceUri;

    /// If resource is known to exist in the past.
    fn was_existing(&self) -> bool;
}

/// A trait to represent resource status token types set.
pub trait ResourceStatusTokenTypes: Debug + Send + 'static {
    /// Type of the repo.
    type Repo: Repo;

    /// Type of variant for existing non represented
    /// resource status tokens.
    type ExistingNonRepresented: ExistingNonRepresentedResourceToken<Repo = Self::Repo>
        + Send
        + 'static;

    /// Type of variant for existing represented
    /// resource status tokens.
    type ExistingRepresented: ExistingRepresentedResourceToken<Repo = Self::Repo> + Send + 'static;

    /// Type of variant for non-existing but
    /// mutex-existing resource status tokens.
    type NonExistingMutexExisting: NonExistingMutexExistingResourceToken<Repo = Self::Repo>
        + Send
        + 'static;

    /// Type of variant for non-existing and
    /// mutex non-existing resource status tokens.
    type NonExistingMutexNonExisting: NonExistingMutexNonExistingResourceToken<Repo = Self::Repo>
        + Send
        + 'static;
}

/// A type to represent existing resource status tokens.
#[derive(Debug, Clone)]
pub enum ExistingResourceToken<RSTypes: ResourceStatusTokenTypes> {
    /// Variant for existing non-represented resource status
    /// tokens.
    NonRepresented(RSTypes::ExistingNonRepresented),

    /// Variant for existing represented resource status tokens.
    Represented(RSTypes::ExistingRepresented),
}

impl<RSTypes: ResourceStatusTokenTypes> ExistingResourceToken<RSTypes> {
    /// Get existing resource slot.
    pub fn slot(&self) -> &SolidResourceSlot<<RSTypes::Repo as Repo>::StSpace> {
        match self {
            ExistingResourceToken::NonRepresented(t) => t.slot(),
            ExistingResourceToken::Represented(t) => t.slot(),
        }
    }

    /// Get the repo context.
    pub fn repo_context(&self) -> &Arc<<RSTypes::Repo as Repo>::Context> {
        match self {
            ExistingResourceToken::NonRepresented(t) => t.repo_context(),
            ExistingResourceToken::Represented(t) => t.repo_context(),
        }
    }
}

/// A type to represent non-existing resource status tokens.
#[derive(Debug, Clone)]
pub enum NonExistingResourceToken<RSTypes: ResourceStatusTokenTypes> {
    /// Variant for non-existing, but mutex-existing resource
    /// status tokens.
    MutexExisting(RSTypes::NonExistingMutexExisting),

    /// Variant for non-existing, and mutex non-existing
    /// resource status tokens.
    MutexNonExisting(RSTypes::NonExistingMutexNonExisting),
}

impl<RSTypes: ResourceStatusTokenTypes> NonExistingResourceToken<RSTypes> {
    /// Get if resource was known to exist once.
    pub fn was_existing(&self) -> bool {
        match self {
            NonExistingResourceToken::MutexExisting(_) => false,
            NonExistingResourceToken::MutexNonExisting(t) => t.was_existing(),
        }
    }

    /// Get existing mutex resource slot, if any.
    pub fn existing_mutex_slot(
        &self,
    ) -> Option<&SolidResourceSlot<<RSTypes::Repo as Repo>::StSpace>> {
        match self {
            NonExistingResourceToken::MutexExisting(t) => Some(t.mutex_slot()),
            NonExistingResourceToken::MutexNonExisting(_) => None,
        }
    }

    /// Get the repo context.
    pub fn repo_context(&self) -> &Arc<<RSTypes::Repo as Repo>::Context> {
        match self {
            NonExistingResourceToken::MutexExisting(t) => t.repo_context(),
            NonExistingResourceToken::MutexNonExisting(t) => t.repo_context(),
        }
    }
}

/// A type to represent resource status tokens.
#[derive(Debug)]
pub enum ResourceStatusToken<RSTypes: ResourceStatusTokenTypes> {
    /// Variant for existing resource status tokens.
    Existing(ExistingResourceToken<RSTypes>),

    /// Variant for non-existing resource status tokens.
    NonExisting(NonExistingResourceToken<RSTypes>),
}

impl<RSTypes: ResourceStatusTokenTypes> ResourceStatusToken<RSTypes> {
    /// Try to convert into existing-represented variant.
    pub fn existing_represented(self) -> Option<RSTypes::ExistingRepresented> {
        match self {
            ResourceStatusToken::Existing(e_token) => match e_token {
                ExistingResourceToken::Represented(er_token) => Some(er_token),
                ExistingResourceToken::NonRepresented(_) => None,
            },
            ResourceStatusToken::NonExisting(_) => None,
        }
    }

    /// Try to convert into non-existing--mutex-non-existing
    /// variant.
    pub fn non_existing_mutex_non_existing(self) -> Option<RSTypes::NonExistingMutexNonExisting> {
        match self {
            ResourceStatusToken::Existing(_) => None,
            ResourceStatusToken::NonExisting(ne_token) => match ne_token {
                NonExistingResourceToken::MutexExisting(_) => None,
                NonExistingResourceToken::MutexNonExisting(ne_mne_token) => Some(ne_mne_token),
            },
        }
    }

    /// Get the repo context.
    pub fn repo_context(&self) -> &Arc<<RSTypes::Repo as Repo>::Context> {
        match self {
            ResourceStatusToken::Existing(t) => t.repo_context(),
            ResourceStatusToken::NonExisting(t) => t.repo_context(),
        }
    }
}

/// Inconsistent token set.
#[derive(Debug, thiserror::Error)]
#[error("Inconsistent token set.")]
pub struct InconsistentTokenSet;

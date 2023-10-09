//! I provide layered resource status token implementations.
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use manas_http::representation::metadata::RepresentationMetadata;
use manas_space::resource::{slot::SolidResourceSlot, uri::SolidResourceUri};

use crate::{
    layer::LayeredRepo,
    service::resource_operator::{
        common::status_token::{
            ExistingNonRepresentedResourceToken, ExistingRepresentedResourceToken,
            ExistingResourceToken, NonExistingMutexExistingResourceToken,
            NonExistingMutexNonExistingResourceToken, NonExistingResourceToken,
            RepoResourceStatusTokenBase, ResourceStatusToken, ResourceStatusTokenTypes,
        },
        creator::ResourceCreateTokenSet,
        deleter::ResourceDeleteTokenSet,
        reader::ResourceReadTokenSet,
        updater::ResourceUpdateTokenSet,
    },
    Repo,
};

/// A struct to represent layered resource status tokens.
#[derive(Clone)]
pub struct Layered<T, LR: Repo> {
    /// Inner token.
    pub inner: T,

    /// Layer context.
    pub layer_context: Arc<LR::Context>,
}

impl<T: Debug, LR: Repo> Debug for Layered<T, LR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layered")
            .field("inner", &self.inner)
            .field("layer_context", &self.layer_context)
            .finish()
    }
}

impl<T, LR: Repo> Layered<T, LR> {
    /// Create a new [`Layered`] resource status token.
    #[inline]
    pub fn new(inner: T, layer_context: Arc<LR::Context>) -> Self {
        Self {
            inner,
            layer_context,
        }
    }
}

impl<T, LR> RepoResourceStatusTokenBase for Layered<T, LR>
where
    T: RepoResourceStatusTokenBase,
    LR: Repo,
{
    type Repo = LR;

    #[inline]
    fn repo_context(&self) -> &Arc<LR::Context> {
        &self.layer_context
    }
}

impl<T, LR> ExistingRepresentedResourceToken for Layered<T, LR>
where
    T: ExistingRepresentedResourceToken,
    LR: Repo<StSpace = <T::Repo as Repo>::StSpace>,
{
    #[inline]
    fn slot(&self) -> &SolidResourceSlot<LR::StSpace> {
        self.inner.slot()
    }

    #[inline]
    fn rep_validators(&self) -> RepresentationMetadata {
        self.inner.rep_validators()
    }
}

impl<T, LR> ExistingNonRepresentedResourceToken for Layered<T, LR>
where
    T: ExistingNonRepresentedResourceToken,
    LR: Repo<StSpace = <T::Repo as Repo>::StSpace>,
{
    #[inline]
    fn slot(&self) -> &SolidResourceSlot<LR::StSpace> {
        self.inner.slot()
    }
}

impl<T, LR> NonExistingMutexExistingResourceToken for Layered<T, LR>
where
    T: NonExistingMutexExistingResourceToken,
    LR: Repo<StSpace = <T::Repo as Repo>::StSpace>,
{
    #[inline]
    fn uri(&self) -> &SolidResourceUri {
        self.inner.uri()
    }

    #[inline]
    fn mutex_slot(&self) -> &SolidResourceSlot<LR::StSpace> {
        self.inner.mutex_slot()
    }
}

impl<T, LR> NonExistingMutexNonExistingResourceToken for Layered<T, LR>
where
    T: NonExistingMutexNonExistingResourceToken,
    LR: Repo<StSpace = <T::Repo as Repo>::StSpace>,
{
    #[inline]
    fn uri(&self) -> &SolidResourceUri {
        self.inner.uri()
    }

    #[inline]
    fn was_existing(&self) -> bool {
        self.inner.was_existing()
    }
}

/// An implementation of [`ResourceStatusTokenTypes`] that
/// has it's token types to layered versions of inner.
#[derive(Clone)]
pub struct LayeredResourceStatusTokenTypes<Inner, LR> {
    _phantom: PhantomData<fn(Inner, LR)>,
}

impl<Inner: Debug, LR> Debug for LayeredResourceStatusTokenTypes<Inner, LR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayeredResourceStatusTokenTypes").finish()
    }
}

impl<Inner, LR> ResourceStatusTokenTypes for LayeredResourceStatusTokenTypes<Inner, LR>
where
    Inner: ResourceStatusTokenTypes,
    LR: Repo<StSpace = <Inner::Repo as Repo>::StSpace>,
{
    type Repo = LR;

    type ExistingNonRepresented = Layered<Inner::ExistingNonRepresented, LR>;

    type ExistingRepresented = Layered<Inner::ExistingRepresented, LR>;

    type NonExistingMutexExisting = Layered<Inner::NonExistingMutexExisting, LR>;

    type NonExistingMutexNonExisting = Layered<Inner::NonExistingMutexNonExisting, LR>;
}

impl<RSTypes, LR> From<Layered<ExistingResourceToken<RSTypes>, LR>>
    for ExistingResourceToken<LayeredResourceStatusTokenTypes<RSTypes, LR>>
where
    RSTypes: ResourceStatusTokenTypes,
    LR: Repo<StSpace = <RSTypes::Repo as Repo>::StSpace>,
{
    fn from(value: Layered<ExistingResourceToken<RSTypes>, LR>) -> Self {
        let Layered {
            inner,
            layer_context,
        } = value;
        match inner {
            ExistingResourceToken::NonRepresented(nr) => {
                ExistingResourceToken::NonRepresented(Layered::new(nr, layer_context))
            }
            ExistingResourceToken::Represented(r) => {
                ExistingResourceToken::Represented(Layered::new(r, layer_context))
            }
        }
    }
}

impl<RSTypes, LR> From<ExistingResourceToken<LayeredResourceStatusTokenTypes<RSTypes, LR>>>
    for Layered<ExistingResourceToken<RSTypes>, LR>
where
    RSTypes: ResourceStatusTokenTypes,
    LR: Repo<StSpace = <RSTypes::Repo as Repo>::StSpace>,
{
    fn from(value: ExistingResourceToken<LayeredResourceStatusTokenTypes<RSTypes, LR>>) -> Self {
        match value {
            ExistingResourceToken::NonRepresented(nr) => Layered::new(
                ExistingResourceToken::NonRepresented(nr.inner),
                nr.layer_context,
            ),
            ExistingResourceToken::Represented(r) => {
                Layered::new(ExistingResourceToken::Represented(r.inner), r.layer_context)
            }
        }
    }
}

impl<RSTypes, LR> From<Layered<NonExistingResourceToken<RSTypes>, LR>>
    for NonExistingResourceToken<LayeredResourceStatusTokenTypes<RSTypes, LR>>
where
    RSTypes: ResourceStatusTokenTypes,
    LR: Repo<StSpace = <RSTypes::Repo as Repo>::StSpace>,
{
    fn from(value: Layered<NonExistingResourceToken<RSTypes>, LR>) -> Self {
        let Layered {
            inner,
            layer_context,
        } = value;
        match inner {
            NonExistingResourceToken::MutexExisting(me) => {
                NonExistingResourceToken::MutexExisting(Layered::new(me, layer_context))
            }
            NonExistingResourceToken::MutexNonExisting(mne) => {
                NonExistingResourceToken::MutexNonExisting(Layered::new(mne, layer_context))
            }
        }
    }
}

impl<RSTypes, LR> From<Layered<ResourceStatusToken<RSTypes>, LR>>
    for ResourceStatusToken<LayeredResourceStatusTokenTypes<RSTypes, LR>>
where
    RSTypes: ResourceStatusTokenTypes,
    LR: Repo<StSpace = <RSTypes::Repo as Repo>::StSpace>,
{
    fn from(value: Layered<ResourceStatusToken<RSTypes>, LR>) -> Self {
        let Layered {
            inner,
            layer_context,
        } = value;
        match inner {
            ResourceStatusToken::Existing(e) => {
                ResourceStatusToken::Existing(Layered::new(e, layer_context).into())
            }
            ResourceStatusToken::NonExisting(ne) => {
                ResourceStatusToken::NonExisting(Layered::new(ne, layer_context).into())
            }
        }
    }
}

impl<IR, LR> From<ResourceCreateTokenSet<LR>> for Layered<ResourceCreateTokenSet<IR>, LR>
where
    IR: Repo,
    LR: LayeredRepo<IR>,
{
    fn from(tokens: ResourceCreateTokenSet<LR>) -> Self {
        let layer_context = tokens.repo_context().clone();
        let (res_token, host_token) = tokens.into_parts();
        Layered::new(
            ResourceCreateTokenSet::try_new(res_token.inner, host_token.inner)
                .expect("Must be consistent."),
            layer_context,
        )
    }
}

impl<IR, LR> From<ResourceReadTokenSet<LR>> for Layered<ResourceReadTokenSet<IR>, LR>
where
    IR: Repo,
    LR: LayeredRepo<IR>,
{
    fn from(tokens: ResourceReadTokenSet<LR>) -> Self {
        let layer_context = tokens.res_token.layer_context;
        Self::new(
            ResourceReadTokenSet::new(tokens.res_token.inner),
            layer_context,
        )
    }
}

impl<IR, LR> From<Layered<ResourceReadTokenSet<IR>, LR>> for ResourceReadTokenSet<LR>
where
    IR: Repo,
    LR: LayeredRepo<IR>,
{
    fn from(l_tokens: Layered<ResourceReadTokenSet<IR>, LR>) -> Self {
        ResourceReadTokenSet::new(Layered::new(
            l_tokens.inner.res_token,
            l_tokens.layer_context,
        ))
    }
}

impl<IR, LR> From<ResourceUpdateTokenSet<LR>> for Layered<ResourceUpdateTokenSet<IR>, LR>
where
    IR: Repo,
    LR: LayeredRepo<IR>,
{
    fn from(tokens: ResourceUpdateTokenSet<LR>) -> Self {
        let l_res_token = Layered::from(tokens.res_token);
        Self::new(
            ResourceUpdateTokenSet::new(l_res_token.inner),
            l_res_token.layer_context,
        )
    }
}

impl<IR, LR> From<ResourceDeleteTokenSet<LR>> for Layered<ResourceDeleteTokenSet<IR>, LR>
where
    IR: Repo,
    LR: LayeredRepo<IR>,
{
    fn from(tokens: ResourceDeleteTokenSet<LR>) -> Self {
        let layer_context = tokens.res_token.layer_context;
        Self::new(
            ResourceDeleteTokenSet::new(tokens.res_token.inner),
            layer_context,
        )
    }
}

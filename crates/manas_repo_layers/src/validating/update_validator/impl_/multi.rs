//! I provide an implementation of [`RepUpdateValidator`] that
//! evaluates multiple inner validation services.
//!

use std::{fmt::Debug, marker::PhantomData, sync::Arc, task::Poll};

use dyn_problem::{ProbFuture, Problem};
use frunk_core::hlist::{HCons, HList, HNil};
use manas_repo::Repo;
use tower::{Service, ServiceExt};

use crate::validating::update_validator::{update_context::RepUpdateContext, RepUpdateValidator};

/// A trait for `HList` of [`RepUpdateValidator`]s..
pub trait RepUpdateValidatorList<R>: HList + Debug + Send + 'static
where
    R: Repo,
{
    /// Type of config.
    type Config: Debug + Send + Sync + 'static + Clone;

    /// Create a new validator list with given config.
    fn new(config: Self::Config) -> Self;

    /// Validate the update context.
    fn validate(
        &mut self,
        update_context: RepUpdateContext<R>,
    ) -> ProbFuture<'static, RepUpdateContext<R>>;
}

impl<R> RepUpdateValidatorList<R> for HNil
where
    R: Repo,
{
    #[inline]
    fn validate(
        &mut self,
        update_context: RepUpdateContext<R>,
    ) -> ProbFuture<'static, RepUpdateContext<R>> {
        Box::pin(async { Ok(update_context) })
    }

    type Config = HNil;

    fn new(_config: Self::Config) -> Self {
        Self
    }
}

impl<R, H, T> RepUpdateValidatorList<R> for HCons<H, T>
where
    H: RepUpdateValidator<R> + Clone,
    T: RepUpdateValidatorList<R>,
    R: Repo,
{
    fn validate(
        &mut self,
        update_context: RepUpdateContext<R>,
    ) -> ProbFuture<'static, RepUpdateContext<R>> {
        let tail_fut = self.tail.validate(update_context);
        let mut head = self.head.clone();

        Box::pin(async move {
            let update_context = tail_fut.await?;
            head.ready().await?.call(update_context).await
        })
    }

    type Config = HCons<Arc<H::Config>, T::Config>;

    fn new(config: Self::Config) -> Self {
        Self {
            head: H::new(config.head),
            tail: T::new(config.tail),
        }
    }
}

/// Config for [`MultiRepUpdateValidator`].
#[derive(Debug, Clone)]
pub struct MultiRepUpdateValidatorConfig<R, VL>
where
    R: Repo,
    VL: RepUpdateValidatorList<R>,
{
    list_config: VL::Config,
    _phantom: PhantomData<R>,
}

impl<R, VL> MultiRepUpdateValidatorConfig<R, VL>
where
    R: Repo,
    VL: RepUpdateValidatorList<R>,
{
    /// Create a new [`]MultiRepUpdateValidatorConfig`].
    pub fn new(list_config: VL::Config) -> Self {
        Self {
            list_config,
            _phantom: PhantomData,
        }
    }
}

/// An implementation of [`RepUpdateValidator`] that
/// evaluates multiple inner validation services.
///
/// Validation order is not guaranteed.
#[derive(Debug, Clone)]
pub struct MultiRepUpdateValidator<R: Repo, VL: RepUpdateValidatorList<R>> {
    list: VL,
    _phantom: PhantomData<fn(R)>,
}

impl<R, VL> Service<RepUpdateContext<R>> for MultiRepUpdateValidator<R, VL>
where
    R: Repo,
    VL: RepUpdateValidatorList<R>,
{
    type Response = RepUpdateContext<R>;

    type Error = Problem;

    type Future = ProbFuture<'static, Self::Response>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, update_context: RepUpdateContext<R>) -> Self::Future {
        self.list.validate(update_context)
    }
}

impl<R, VL> RepUpdateValidator<R> for MultiRepUpdateValidator<R, VL>
where
    R: Repo,
    VL: RepUpdateValidatorList<R>,
{
    type Config = MultiRepUpdateValidatorConfig<R, VL>;

    fn new(config: Arc<Self::Config>) -> Self {
        Self {
            list: VL::new(config.as_ref().list_config.clone()),
            _phantom: PhantomData,
        }
    }
}

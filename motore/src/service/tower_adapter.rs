//! This module provides the Adapter trait, which is used to convert a Motore service into a Tower
//! service and vice versa.
//!
//! Take `TowerAdapter` for example: it will be automatically implemented for any type that
//! implements `Motore::Service`. Thus, you can use `.tower(f)` method with a closure parameters
//! passed in to convert a Motore service into a Tower service.
//!
//! # Example
//!
//! ```rust, ignore
//! // Convert a Motore service into a Tower service
//! let tower_service = motore_service.tower(|tower_req| { cx, motore_req });
//!
//! // Convert a Tower service into a Motore service
//! let motore_service = tower_service.motore(|cx, motore_req| { tower_req });
//! ```

use std::{
    fmt,
    marker::PhantomData,
    task::{Context, Poll},
};

#[cfg(feature = "service_send")]
use futures::future::BoxFuture;
#[cfg(not(feature = "service_send"))]
use futures::future::LocalBoxFuture;
use futures::{Future, FutureExt};
use tower::ServiceExt;

use crate::Service;

impl<T: ?Sized, Cx, MotoreReq, TowerReq> TowerAdapter<Cx, MotoreReq, TowerReq> for T where
    T: Service<Cx, MotoreReq>
{
}

#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub trait TowerAdapter<Cx, MotoreReq, TowerReq>: Service<Cx, MotoreReq> {
    fn tower<F>(self, f: F) -> Tower<Self, F, Cx, MotoreReq>
    where
        F: FnOnce(TowerReq) -> (Cx, MotoreReq),
        Self: Sized,
    {
        Tower::new(self, f)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub struct Tower<S, F, Cx, MotoreReq> {
    inner: S,
    f: F,
    _phantom: PhantomData<fn(Cx, MotoreReq)>,
}

impl<S, F, Cx, MotoreReq> Tower<S, F, Cx, MotoreReq> {
    pub const fn new(inner: S, f: F) -> Self {
        Self {
            inner,
            f,
            _phantom: PhantomData,
        }
    }
}

#[cfg(feature = "service_send")]
impl<S, F, Cx, MotoreReq, TowerReq> tower::Service<TowerReq> for Tower<S, F, Cx, MotoreReq>
where
    S: Service<Cx, MotoreReq> + Clone + 'static + Send,
    F: FnOnce(TowerReq) -> (Cx, MotoreReq) + Clone,
    MotoreReq: 'static + Send,
    Cx: 'static + Send,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TowerReq) -> Self::Future {
        let inner = self.inner.clone();
        let (mut cx, r) = (self.f.clone())(req);
        async move { inner.call(&mut cx, r).await }.boxed()
    }
}

#[cfg(not(feature = "service_send"))]
impl<S, F, Cx, MotoreReq, TowerReq> tower::Service<TowerReq> for Tower<S, F, Cx, MotoreReq>
where
    S: Service<Cx, MotoreReq> + Clone + 'static,
    F: FnOnce(TowerReq) -> (Cx, MotoreReq) + Clone,
    MotoreReq: 'static,
    Cx: 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TowerReq) -> Self::Future {
        let inner = self.inner.clone();
        let (mut cx, r) = (self.f.clone())(req);
        async move { inner.call(&mut cx, r).await }.boxed_local()
    }
}

impl<S, F, Cx, MotoreReq> Clone for Tower<S, F, Cx, MotoreReq>
where
    S: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            f: self.f.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<S, F, Cx, MotoreReq> fmt::Debug for Tower<S, F, Cx, MotoreReq>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tower")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<T: ?Sized, Cx, MotoreReq, TowerReq> MotoreAdapter<Cx, MotoreReq, TowerReq> for T where
    T: tower::Service<TowerReq>
{
}

/// Converts a tower [`Service`](tower::Service) into a motore [`Service`].
///
/// The adapter polls the tower service for readiness before dispatching each
/// request.
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub trait MotoreAdapter<Cx, MotoreReq, TowerReq>: tower::Service<TowerReq> {
    fn motore<F>(self, f: F) -> Motore<Self, F>
    where
        F: FnOnce(&mut Cx, MotoreReq) -> TowerReq,
        Self: Sized,
    {
        Motore::new(self, f)
    }
}

/// A motore [`Service`] backed by a tower [`Service`](tower::Service).
///
/// The tower service is cloned for each call and polled for readiness before
/// the request is dispatched.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub struct Motore<S, F> {
    inner: S,
    f: F,
}

impl<S, F> Motore<S, F> {
    pub const fn new(inner: S, f: F) -> Self {
        Self { inner, f }
    }
}

#[cfg(feature = "service_send")]
impl<S, F, Cx, MotoreReq, TowerReq> Service<Cx, MotoreReq> for Motore<S, F>
where
    S: tower::Service<TowerReq> + Clone + Send,
    for<'cx> <S as tower::Service<TowerReq>>::Future: Send + 'cx,
    F: FnOnce(&mut Cx, MotoreReq) -> TowerReq + Clone,
    TowerReq: Send,
{
    type Response = S::Response;

    type Error = S::Error;

    fn call(
        &self,
        cx: &mut Cx,
        req: MotoreReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.inner.clone().oneshot((self.f.clone())(cx, req))
    }
}

#[cfg(not(feature = "service_send"))]
impl<S, F, Cx, MotoreReq, TowerReq> Service<Cx, MotoreReq> for Motore<S, F>
where
    S: tower::Service<TowerReq> + Clone,
    for<'cx> <S as tower::Service<TowerReq>>::Future: Send + 'cx,
    F: FnOnce(&mut Cx, MotoreReq) -> TowerReq + Clone,
{
    type Response = S::Response;

    type Error = S::Error;

    fn call(
        &self,
        cx: &mut Cx,
        req: MotoreReq,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        self.inner.clone().oneshot((self.f.clone())(cx, req))
    }
}

impl<S, F> fmt::Debug for Motore<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Motore")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        convert::Infallible,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        task::{Context, Poll},
    };

    use super::*;

    #[derive(Clone)]
    struct RequiresReady {
        ready: Arc<AtomicBool>,
    }

    impl tower::Service<u64> for RequiresReady {
        type Response = u64;
        type Error = Infallible;
        type Future = std::future::Ready<Result<u64, Infallible>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.ready.store(true, Ordering::SeqCst);
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: u64) -> Self::Future {
            assert!(
                self.ready.swap(false, Ordering::SeqCst),
                "tower service was called before poll_ready"
            );
            std::future::ready(Ok(req))
        }
    }

    #[test]
    fn motore_adapter_polls_ready_before_call() {
        let tower = RequiresReady {
            ready: Arc::new(AtomicBool::new(false)),
        };
        let service = tower.motore(|_cx: &mut (), req: u64| req);
        let mut cx = ();

        let res = futures::executor::block_on(service.call(&mut cx, 42));

        assert_eq!(res.unwrap(), 42);
    }
}

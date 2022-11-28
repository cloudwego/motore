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

use futures::Future;

use crate::Service;

impl<T: ?Sized, Cx, MotoreReq, TowerReq> TowerAdapter<Cx, MotoreReq, TowerReq> for T where
    T: Service<Cx, MotoreReq>
{
}

pub trait TowerAdapter<Cx, MotoreReq, TowerReq>: Service<Cx, MotoreReq> {
    fn tower<F>(self, f: F) -> Tower<Self, F, Cx, MotoreReq>
    where
        F: FnOnce(TowerReq) -> (Cx, MotoreReq),
        Self: Sized,
    {
        Tower::new(self, f)
    }
}

pub struct Tower<S, F, Cx, MotoreReq> {
    inner: S,
    f: F,
    _phantom: PhantomData<fn(Cx, MotoreReq)>,
}

impl<S, F, Cx, MotoreReq> Tower<S, F, Cx, MotoreReq> {
    pub fn new(inner: S, f: F) -> Self {
        Self {
            inner,
            f,
            _phantom: PhantomData,
        }
    }
}

impl<S, F, Cx, MotoreReq, TowerReq> tower::Service<TowerReq> for Tower<S, F, Cx, MotoreReq>
where
    S: Service<Cx, MotoreReq> + Clone,
    F: FnOnce(TowerReq) -> (Cx, MotoreReq) + Clone,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = impl Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: TowerReq) -> Self::Future {
        let inner = self.inner.clone();
        let (mut cx, r) = (self.f.clone())(req);
        async move { inner.call(&mut cx, r).await }
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

pub trait MotoreAdapter<Cx, MotoreReq, TowerReq>: tower::Service<TowerReq> {
    fn motore<F>(self, f: F) -> Motore<Self, F>
    where
        F: FnOnce(&mut Cx, MotoreReq) -> TowerReq,
        Self: Sized,
    {
        Motore::new(self, f)
    }
}

#[derive(Clone)]
pub struct Motore<S, F> {
    inner: S,
    f: F,
}

impl<S, F> Motore<S, F> {
    pub fn new(inner: S, f: F) -> Self {
        Self { inner, f }
    }
}

impl<S, F, Cx, MotoreReq, TowerReq> Service<Cx, MotoreReq> for Motore<S, F>
where
    S: tower::Service<TowerReq> + Clone,
    for<'cx> <S as tower::Service<TowerReq>>::Future: 'cx,
    F: FnOnce(&mut Cx, MotoreReq) -> TowerReq + Clone,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: MotoreReq) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        self.inner.clone().call((self.f.clone())(cx, req))
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

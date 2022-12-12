//! The Adapter layer is used to convert a Motore service into a Tower service and vice versa.
//!
//! # Example
//!
//! ```rust, ignore
//! // Convert a Motore service into a Tower service
//! let tower_service = ServiceBuilder::new()
//!     .layer(tower_layer)
//!     .layer(TowerAdapterLayer::new(|tower_req| (cx, motore_req)))
//!     .service(motore_service);
//!
//! // Convert a Tower service into a Motore service
//! let motore_service = ServiceBuilder::new()
//!     .layer(motore_layer)
//!     .layer(TowerAdapterLayer::new(|cx, motore_req| tower_req))
//!     .service(tower_service);

use std::{fmt, marker::PhantomData};

use super::Layer;
use crate::service::{Motore, Tower};
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub struct TowerAdapterLayer<F, Cx, MotoreReq> {
    f: F,
    _phantom: PhantomData<fn(Cx, MotoreReq)>,
}

impl<F, Cx, MotoreReq> TowerAdapterLayer<F, Cx, MotoreReq> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<S, F, Cx, MotoreReq> tower::Layer<S> for TowerAdapterLayer<F, Cx, MotoreReq>
where
    F: Clone,
{
    type Service = Tower<S, F, Cx, MotoreReq>;

    fn layer(&self, inner: S) -> Self::Service {
        Tower::new(inner, self.f.clone())
    }
}

impl<F, Cx, MotoreReq> Clone for TowerAdapterLayer<F, Cx, MotoreReq>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<F, Cx, MotoreReq> fmt::Debug for TowerAdapterLayer<F, Cx, MotoreReq> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TowerAdapterLayer")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
pub struct MotoreAdapterLayer<F> {
    f: F,
}

impl<S, F> Layer<S> for MotoreAdapterLayer<F> {
    type Service = Motore<S, F>;

    fn layer(self, inner: S) -> Self::Service {
        Motore::new(inner, self.f)
    }
}

impl<F> fmt::Debug for MotoreAdapterLayer<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MotoreAdapterLayer")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

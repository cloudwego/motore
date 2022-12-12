//! Builder types to compose layers and services

use std::fmt;

use crate::layer::{Identity, Layer, Stack};

/// Declaratively construct [`Service`] values.
///
/// [`ServiceBuilder`] provides a builder-like interface for composing
/// layers to be applied to a [`Service`].
///
/// [`Service`]: crate::service::Service
#[derive(Clone)]
pub struct ServiceBuilder<L> {
    layer: L,
}

impl Default for ServiceBuilder<Identity> {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceBuilder<Identity> {
    /// Create a new [`ServiceBuilder`].
    pub fn new() -> Self {
        ServiceBuilder {
            layer: Identity::new(),
        }
    }
}

impl<L> ServiceBuilder<L> {
    /// Add a new layer `T` into the [`ServiceBuilder`].
    ///
    /// This wraps the inner service with the service provided by a user-defined
    /// [`Layer`]. The provided layer must implement the [`Layer`] trait.
    ///
    /// [`Layer`]: crate::layer::Layer
    pub fn layer<T>(self, layer: T) -> ServiceBuilder<Stack<T, L>> {
        ServiceBuilder {
            layer: Stack::new(layer, self.layer),
        }
    }

    /// Optionally add a new layer `T` into the [`ServiceBuilder`].
    pub fn option_layer<T>(
        self,
        layer: Option<T>,
    ) -> ServiceBuilder<Stack<crate::utils::Either<T, Identity>, L>> {
        self.layer(crate::utils::option_layer(layer))
    }

    /// Add a [`Layer`] built from a function that accepts a service and returns another service.
    ///
    /// See the documentation for [`layer_fn`] for more details.
    ///
    /// [`layer_fn`]: crate::layer::layer_fn
    pub fn layer_fn<F>(self, f: F) -> ServiceBuilder<Stack<crate::layer::LayerFn<F>, L>> {
        self.layer(crate::layer::layer_fn(f))
    }

    /// Fail requests that take longer than `timeout`.
    ///
    /// If the next layer takes more than `timeout` to respond to a request,
    /// processing is terminated and an error is returned.
    ///
    /// This wraps the inner service with an instance of the [`timeout`]
    /// middleware.
    ///
    /// [`timeout`]: crate::timeout
    pub fn timeout(
        self,
        timeout: Option<std::time::Duration>,
    ) -> ServiceBuilder<Stack<crate::timeout::TimeoutLayer, L>> {
        self.layer(crate::timeout::TimeoutLayer::new(timeout))
    }

    /// Map one error type to another.
    ///
    /// This wraps the inner service with an instance of the [`MapErr`]
    /// middleware.
    ///
    /// [`MapErr`]: crate::service::MapErr
    pub fn map_err<F>(self, f: F) -> ServiceBuilder<Stack<crate::layer::MapErrLayer<F>, L>> {
        self.layer(crate::layer::MapErrLayer::new(f))
    }

    /// Returns the underlying `Layer` implementation.
    pub fn into_inner(self) -> L {
        self.layer
    }

    /// Wrap the service `S` with the middleware provided by this
    /// [`ServiceBuilder`]'s [`Layer`]'s, returning a new [`Service`].
    ///
    /// [`Layer`]: crate::layer::Layer
    /// [`Service`]: crate::service::Service
    pub fn service<S>(self, service: S) -> L::Service
    where
        L: Layer<S>,
    {
        self.layer.layer(service)
    }

    /// Wrap the async function `F` with the middleware provided by this [`ServiceBuilder`]'s
    /// [`Layer`]s, returning a new [`Service`].
    ///
    /// [`Layer`]: crate::layer::Layer
    /// [`Service`]: crate::service::Service
    pub fn service_fn<F>(self, f: F) -> L::Service
    where
        L: Layer<crate::service::ServiceFn<F>>,
    {
        self.service(crate::service::service_fn(f))
    }
}

impl<L: fmt::Debug> fmt::Debug for ServiceBuilder<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ServiceBuilder").field(&self.layer).finish()
    }
}

impl<S, L> Layer<S> for ServiceBuilder<L>
where
    L: Layer<S>,
{
    type Service = L::Service;

    fn layer(self, inner: S) -> Self::Service {
        self.layer.layer(inner)
    }
}

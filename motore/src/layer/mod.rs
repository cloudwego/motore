//! Layer traits and extensions.
//!
//! A layer decorates an service and provides additional functionality. It
//! allows other services to be composed with the service that implements layer.
//!
//! A middleware implements the [`Layer`] and [`Service`] trait.
//!
//! [`Service`]: https://docs.rs/motore/latest/motore/trait.Service.html

mod ext;
mod identity;
mod layer_fn;
mod layers;
mod stack;

pub use self::{
    ext::{LayerExt, MapErrLayer},
    identity::Identity,
    layer_fn::{layer_fn, LayerFn},
    layers::Layers,
    stack::Stack,
};

/// Decorates a [`Service`], transforming either the request or the response.
///
/// Often, many of the pieces needed for writing network applications can be
/// reused across multiple services. The `Layer` trait can be used to write
/// reusable components that can be applied to very different kinds of services;
/// for example, it can be applied to services operating on different protocols,
/// and to both the client and server side of a network transaction.
///
/// # Example
///
/// ```rust
/// use motore::{layer::Layer, timeout::Timeout, BoxError, Service};
///
/// #[derive(Clone)]
/// pub struct TimeoutLayer {
///     duration: Option<std::time::Duration>,
/// }
///
/// impl TimeoutLayer {
///     pub fn new(duration: Option<std::time::Duration>) -> Self {
///         TimeoutLayer { duration }
///     }
/// }
///
/// impl<S> Layer<S> for TimeoutLayer {
///     type Service = Timeout<S>;
///
///     fn layer(self, inner: S) -> Self::Service {
///         Timeout::new(inner, self.duration)
///     }
/// }
/// ```
pub trait Layer<S> {
    /// The wrapped service
    type Service;

    /// Wrap the given service with the middleware, returning a new service
    /// that has been decorated with the middleware. Consumes self.
    fn layer(self, inner: S) -> Self::Service;
}

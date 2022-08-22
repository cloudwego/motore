use std::fmt;

use super::Layer;

/// Returns a new [`LayerFn`] that implements [`Layer`] by calling the
/// given function.
///
/// The [`Layer::layer`] method takes a type implementing [`Service`] and
/// returns a different type implementing [`Service`]. In many cases, this can
/// be implemented by a function or a closure. The [`LayerFn`] helper allows
/// writing simple [`Layer`] implementations without needing the boilerplate of
/// a new struct implementing [`Layer`].
///
/// # Example
///
/// ```rust
/// # #![feature(generic_associated_types)]
/// # #![feature(type_alias_impl_trait)]
/// #
/// # use futures::Future;
/// # use motore::layer::{Layer, layer_fn};
/// # use motore::service::{service_fn, Service, ServiceFn};
/// # use std::convert::Infallible;
/// # use std::fmt;
/// #
/// // A middleware that logs requests before forwarding them to another service
/// pub struct LogService<S> {
///     target: &'static str,
///     service: S,
/// }
///
/// impl<S, Cx, Request> Service<Cx, Request> for LogService<S>
/// where
///     S: Service<Cx, Request>,
///     Request: fmt::Debug,
/// {
///     type Response = S::Response;
///     type Error = S::Error;
///     type Future<'cx> = S::Future<'cx>
/// where
///     Cx: 'cx,
///     S: 'cx;
///
///     fn call<'cx, 's>(&'s mut self, cx: &'cx mut Cx, req: Request) -> Self::Future<'cx>
///     where
///         's: 'cx,
///     {
///         // Log the request
///         println!("req = {:?}, target = {:?}", req, self.target);
///
///         self.service.call(cx, req)
///     }
/// }
///
/// // A `Layer` that wraps services in `LogService`
/// let log_layer = layer_fn(|service| LogService {
///     service,
///     target: "motore-docs",
/// });
///
/// #[derive(Debug)]
/// pub struct MotoreContext;
/// async fn handle(cx: &mut MotoreContext, req: String) -> Result<String, Infallible> {
///     println!("{:?}, {:?}", cx, req);
///     Ok::<_, Infallible>(req.to_uppercase())
/// }
///
/// // An example service. This one uppercases strings
/// let mut uppercase_service = service_fn(handle);
///
/// // Wrap our service in a `LogService` so requests are logged.
/// let wrapped_service = log_layer.layer(uppercase_service);
/// ```
pub fn layer_fn<F>(f: F) -> LayerFn<F> {
    LayerFn { f }
}

/// A `Layer` implemented by a closure. See the docs for [`layer_fn`] for more details.
#[derive(Clone, Copy)]
pub struct LayerFn<F> {
    f: F,
}

impl<S, F, Out> Layer<S> for LayerFn<F>
where
    F: Fn(S) -> Out,
{
    type Service = Out;

    fn layer(self, inner: S) -> Self::Service {
        (self.f)(inner)
    }
}

impl<F> fmt::Debug for LayerFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerFn")
            .field("f", &format_args!("<{}>", std::any::type_name::<F>()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[test]
    fn debug_impl_ok() {
        struct WrappedService<S> {
            inner: S,
        }
        let layer = layer_fn(|svc| WrappedService { inner: svc });
        let _svc = layer.layer("foo");
        assert_eq!(
            "LayerFn { f: <motore::layer::layer_fn::tests::debug_impl_ok::{{closure}}> }"
                .to_string(),
            format!("{:?}", layer),
        );
    }
}

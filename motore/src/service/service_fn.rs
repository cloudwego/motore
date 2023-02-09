use std::fmt;

use futures::Future;

use crate::service::Service;

/// Returns a new [`ServiceFn`] with the given closure.
///
/// This lets you build a [`Service`] from an async function that returns a [`Result`].
///
/// # Example
///
/// ```rust
/// # #![feature(type_alias_impl_trait)]
/// #
/// # use futures::Future;
/// # use motore::service::{service_fn, Service, ServiceFn};
/// # use motore::BoxError;
/// #
/// # #[derive(Debug)]
/// # struct MotoreContext;
/// # #[derive(Debug)]
/// # struct Request;
/// # struct Response;
///
/// async fn handle(cx: &mut MotoreContext, req: Request) -> Result<Response, BoxError> {
///     println!("{:?}, {:?}", cx, req);
///     Ok(Response)
/// }
///
/// let mut service = service_fn(handle);
///
/// let _ = service.call(&mut MotoreContext, Request);
/// ```
pub fn service_fn<F>(f: F) -> ServiceFn<F> {
    ServiceFn { f }
}

/// A [`Service`] implemented by a closure. See the docs for [`service_fn`] for more details.
#[derive(Copy, Clone)]
pub struct ServiceFn<F> {
    f: F,
}

impl<Cx, F, Request, R, E> Service<Cx, Request> for ServiceFn<F>
where
    F: for<'r> Callback<'r, Cx, Request, Response = R, Error = E>,
    Request: 'static,
    R: 'static,
    E: 'static,
{
    type Response = R;
    type Error = E;
    type Future<'cx> = impl Future<Output= Result<R, E>> + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Request) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        (self.f).call(cx, req)
    }
}

impl<F> fmt::Debug for ServiceFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceFn")
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

/// [`Service`] for binding lifetime to return value while using closure.
/// This is just a temporary workaround for lifetime issues.
///
/// Related issue: https://github.com/rust-lang/rust/issues/70263.
/// Related RFC: https://github.com/rust-lang/rfcs/pull/3216.
pub trait Callback<'r, Cx, Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>> + Send + 'r;

    fn call(&self, cx: &'r mut Cx, req: Request) -> Self::Future;
}

impl<'r, F, Fut, Cx, Request, R, E> Callback<'r, Cx, Request> for F
where
    F: Fn(&'r mut Cx, Request) -> Fut,
    Fut: Future<Output = Result<R, E>> + Send + 'r,
    Cx: 'r,
{
    type Response = R;
    type Error = E;
    type Future = Fut;

    fn call(&self, cx: &'r mut Cx, req: Request) -> Self::Future {
        self(cx, req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_impl_ok() {
        use std::convert::Infallible;

        #[derive(Debug)]
        struct MotoreContext;

        async fn handle(cx: &mut MotoreContext, request: String) -> Result<String, Infallible> {
            println!("{cx:?}, {request:?}");
            Ok::<_, Infallible>(request.to_uppercase())
        }

        let uppercase_service = service_fn(handle);
        let _ = uppercase_service.call(&mut MotoreContext, "req".to_string());
        assert_eq!(
            "ServiceFn { f: motore::service::service_fn::tests::debug_impl_ok::handle }"
                .to_string(),
            format!("{uppercase_service:?}"),
        );
    }
}

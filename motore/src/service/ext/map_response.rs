use std::{fmt, future::Future};

use futures::TryFutureExt;

use crate::Service;
/// Service returned by the [`map_response`] combinator.
///
/// [`map_response`]: crate::service::ServiceExt::map_response
#[derive(Clone)]
pub struct MapResponse<S, F> {
    pub(crate) inner: S,
    pub(crate) f: F,
}

impl<S, F, Cx, Req, Response> Service<Cx, Req> for MapResponse<S, F>
where
    S: Service<Cx, Req>,
    F: FnOnce(S::Response) -> Response + Clone + Send,
{
    type Response = Response;
    type Error = S::Error;

    #[cfg(feature = "service_send")]
    fn call(
        &self,
        cx: &mut Cx,
        req: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.inner.call(cx, req).map_ok(self.f.clone())
    }
    #[cfg(not(feature = "service_send"))]
    fn call(
        &self,
        cx: &mut Cx,
        req: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        self.inner.call(cx, req).map_ok(self.f.clone())
    }
}

impl<S, F> fmt::Debug for MapResponse<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponse")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

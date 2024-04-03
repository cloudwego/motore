use std::future::Future;

use futures::TryFutureExt;

use crate::Service;

/// Service returned by the [`map_err`] combinator.
///
/// [`map_err`]: crate::service::ServiceExt::map_err
#[derive(Clone)]
pub struct MapErr<S, F> {
    pub(crate) inner: S,
    pub(crate) f: F,
}

impl<Cx, Req, S, F, E> Service<Cx, Req> for MapErr<S, F>
where
    S: Service<Cx, Req>,
    F: FnOnce(S::Error) -> E + Clone + Send,
{
    type Response = S::Response;

    type Error = E;

    #[cfg(feature = "service_send")]
    fn call(
        &self,
        cx: &mut Cx,
        req: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.inner.call(cx, req).map_err(self.f.clone())
    }
    #[cfg(not(feature = "service_send"))]
    fn call(
        &self,
        cx: &mut Cx,
        req: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        self.inner.call(cx, req).map_err(self.f.clone())
    }
}

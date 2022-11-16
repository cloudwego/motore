use futures::Future;

use crate::Service;

/// Service returned by the [`map_err`] combinator.
///
/// [`map_err`]: crate::service::ext::ServiceExt::map_err
#[derive(Clone)]
pub struct MapErr<S, F> {
    pub(crate) inner: S,
    pub(crate) f: F,
}

impl<Cx, Req, S, F, E> Service<Cx, Req> for MapErr<S, F>
where
    E: 'static,
    Req: Send + 'static,
    S: Service<Cx, Req> + Send + Sync,
    Cx: Send,
    for<'cx> S::Future<'cx>: Send,
    F: FnOnce(S::Error) -> E + Clone + Send + Sync,
{
    type Response = S::Response;

    type Error = E;

    type Future<'cx> = impl Send + Future<Output = Result<Self::Response, Self::Error>> + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        let f = self.f.clone();
        async move {
            match self.inner.call(cx, req).await {
                Ok(r) => Ok(r),
                Err(e) => Err((f)(e)),
            }
        }
    }
}

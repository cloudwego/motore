use futures::{Future, TryFutureExt};

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
    S: Service<Cx, Req>,
    F: FnOnce(S::Error) -> E + Clone + Send,
{
    type Response = S::Response;

    type Error = E;

    type Future<'cx> = impl   Future<Output = Result<Self::Response, Self::Error>> + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        self.inner.call(cx, req).map_err(self.f.clone())
    }
}

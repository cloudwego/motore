use futures::Future;

use crate::{layer::Layer, service::Service, BoxError};

/// Combine two different service types into a single type.
///
/// Both services must be of the same request, response, and error types.
/// [`Either`] is useful for handling conditional branching in service middleware
/// to different inner service types.
#[derive(Clone, Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<S, A, B> Layer<S> for Either<A, B>
where
    A: Layer<S>,
    B: Layer<S>,
{
    type Service = Either<A::Service, B::Service>;

    fn layer(self, inner: S) -> Self::Service {
        match self {
            Either::A(layer) => Either::A(layer.layer(inner)),
            Either::B(layer) => Either::B(layer.layer(inner)),
        }
    }
}

impl<A, B, Cx, Req> Service<Cx, Req> for Either<A, B>
where
    Req: 'static + Send,
    Cx: Send + 'static,
    A: Service<Cx, Req> + Send + 'static,
    B: Service<Cx, Req, Response = A::Response> + Send + 'static,
    A::Error: Into<BoxError>,
    B::Error: Into<BoxError>,
{
    type Response = A::Response;

    type Error = BoxError;

    type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    fn call<'cx, 's>(&'s mut self, cx: &'cx mut Cx, req: Req) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        async move {
            match self {
                Either::A(s) => s.call(cx, req).await.map_err(|e| e.into()),
                Either::B(s) => s.call(cx, req).await.map_err(|e| e.into()),
            }
        }
    }
}

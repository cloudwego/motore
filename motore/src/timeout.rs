//! Applies a timeout to request
//! if the inner service's call does not complete within specified timeout, the response will be
//! aborted.

use std::time::Duration;

use crate::{layer::Layer, service::Service, BoxError};

#[derive(Clone)]
pub struct Timeout<S> {
    inner: S,
    duration: Option<Duration>,
}

impl<S> Timeout<S> {
    pub fn new(inner: S, duration: Option<Duration>) -> Self {
        Self { inner, duration }
    }
}

impl<Cx, Req, S> Service<Cx, Req> for Timeout<S>
where
    Req: 'static + Send,
    S: Service<Cx, Req> + 'static + Send + Sync,
    Cx: 'static + Send,
    S::Error: Send + Sync + Into<BoxError>,
{
    type Response = S::Response;

    type Error = BoxError;

    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut Cx,
        req: Req,
    ) -> Result<Self::Response, Self::Error> {
        match self.duration {
            Some(duration) => {
                let sleep = tokio::time::sleep(duration);
                tokio::select! {
                    r = self.inner.call(cx, req) => {
                        r.map_err(Into::into)
                    },
                    _ = sleep => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "service time out").into()),
                }
            }
            None => self.inner.call(cx, req).await.map_err(Into::into),
        }
    }
}

#[derive(Clone)]
pub struct TimeoutLayer {
    duration: Option<Duration>,
}

impl TimeoutLayer {
    pub fn new(duration: Option<Duration>) -> Self {
        TimeoutLayer { duration }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = Timeout<S>;

    fn layer(self, inner: S) -> Self::Service {
        Timeout {
            inner,
            duration: self.duration,
        }
    }
}

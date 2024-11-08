use std::fmt;

use super::Layer;

/// Two middlewares chained together.
#[derive(Clone)]
pub struct Stack<Inner, Outer> {
    inner: Inner,
    outer: Outer,
}

impl<Inner, Outer> Stack<Inner, Outer> {
    /// Create a new `Stack`.
    pub const fn new(inner: Inner, outer: Outer) -> Self {
        Stack { inner, outer }
    }
}

impl<S, Inner, Outer> Layer<S> for Stack<Inner, Outer>
where
    Inner: Layer<S>,
    Outer: Layer<Inner::Service>,
{
    type Service = Outer::Service;

    fn layer(self, service: S) -> Self::Service {
        let inner = self.inner.layer(service);

        self.outer.layer(inner)
    }
}

impl<Inner, Outer> fmt::Debug for Stack<Inner, Outer>
where
    Inner: fmt::Debug,
    Outer: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}, {:?}", self.outer, self.inner)
    }
}

use crate::{layer::Layer, service::MapErr};

pub struct MapErrLayer<F> {
    pub(crate) f: F,
}

impl<F> MapErrLayer<F> {
    pub fn new(f: F) -> Self {
        MapErrLayer { f }
    }
}

impl<S, F: Clone> Layer<S> for MapErrLayer<F> {
    type Service = MapErr<S, F>;

    fn layer(self, svc: S) -> Self::Service {
        MapErr {
            inner: svc,
            f: self.f,
        }
    }
}

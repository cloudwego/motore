use super::Layer;

pub trait ServiceLayerExt: Sized {
    fn layer<L>(self, l: L) -> L::Service
    where
        L: Layer<Self>;
}

impl<S> ServiceLayerExt for S {
    fn layer<L>(self, l: L) -> L::Service
    where
        L: Layer<Self>,
    {
        Layer::layer(l, self)
    }
}

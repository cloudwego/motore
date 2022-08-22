use super::Layer;
use crate::Service;

mod map_err;
pub use self::map_err::MapErrLayer;

pub trait LayerExt<Cx, Req, S>: Layer<S> + Sized
where
    S: Service<Cx, Req>,
{
    fn map_err<E, F: FnOnce(S::Error) -> E>(self, f: F) -> MapErrLayer<F>;
}

impl<Cx, Req, T, S> LayerExt<Cx, Req, S> for T
where
    T: Layer<S>,
    S: Service<Cx, Req>,
{
    fn map_err<E, F: FnOnce(S::Error) -> E>(self, f: F) -> MapErrLayer<F> {
        MapErrLayer { f }
    }
}

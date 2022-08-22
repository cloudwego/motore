use crate::Service;

mod map_err;
pub use self::map_err::MapErr;

/// An extension trait for `Service`s that provides a variety of convenient
/// adapters
pub trait ServiceExt<Cx, Req>: Service<Cx, Req> + Sized {
    /// Maps this service's error value to a different value.
    ///
    /// This method can be used to change the [`Error`] type of the service
    /// into a different type. It is similar to the [`Result::map_err`] method.
    fn map_err<E, F: FnOnce(Self::Error) -> E>(self, f: F) -> MapErr<Self, F>;
}

impl<T, Cx, Req> ServiceExt<Cx, Req> for T
where
    T: Service<Cx, Req>,
{
    fn map_err<E, F: FnOnce(Self::Error) -> E>(self, f: F) -> MapErr<Self, F> {
        MapErr { inner: self, f }
    }
}

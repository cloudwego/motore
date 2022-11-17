use crate::Service;

mod map_err;
mod map_response;
pub use self::{map_err::MapErr, map_response::MapResponse};

/// An extension trait for `Service`s that provides a variety of convenient
/// adapters
pub trait ServiceExt<Cx, Req>: Service<Cx, Req> + Sized {
    /// Maps this service's error value to a different value.
    ///
    /// This method can be used to change the [`Error`] type of the service
    /// into a different type. It is similar to the [`Result::map_err`] method.
    fn map_err<E, F: FnOnce(Self::Error) -> E>(self, f: F) -> MapErr<Self, F>;

    /// Maps this service's response value to a different value.
    ///
    /// This method can be used to change the [`Response`] type of the service
    /// into a different type. It is similar to the [`Result::map`]
    /// method. You can use this method to chain along a computation once the
    /// service's response has been resolved.
    fn map_response<F: FnOnce(Self::Response) -> Response, Response>(
        self,
        f: F,
    ) -> MapResponse<Self, F>;
}

impl<T, Cx, Req> ServiceExt<Cx, Req> for T
where
    T: Service<Cx, Req>,
{
    fn map_err<E, F: FnOnce(Self::Error) -> E>(self, f: F) -> MapErr<Self, F> {
        MapErr { inner: self, f }
    }

    fn map_response<F: FnOnce(Self::Response) -> Response, Response>(
        self,
        f: F,
    ) -> MapResponse<Self, F> {
        MapResponse { inner: self, f }
    }
}

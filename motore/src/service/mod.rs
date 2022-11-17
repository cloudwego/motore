//! Definition of the core `Service` trait to Motore.
//!
//! The [`Service`] trait provides the necessary abstractions for defining
//! request / response clients and servers. It is simple but powerful and is
//! used as the foundation for the rest of Motore.

use std::{fmt, future::Future};

use futures::future::BoxFuture;

mod ext;
mod service_fn;

pub use ext::*;
pub use service_fn::{service_fn, ServiceFn};

/// An asynchronous function from a `Request` to a `Response`.
///
/// The `Service` trait is a simplified interface making it easy to write
/// network applications in a modular and reusable way, decoupled from the
/// underlying protocol. It is one of Tower's fundamental abstractions.
///
/// # Functional
///
/// A `Service` is a function of a `Request`. It immediately returns a
/// `Future` representing the eventual completion of processing the
/// request. The actual request processing may happen at any time in the
/// future, on any thread or executor. The processing may depend on calling
/// other services. At some point in the future, the processing will complete,
/// and the `Future` will resolve to a response or error.
///
/// At a high level, the `Service::call` function represents an RPC request. The
/// `Service` value can be a server or a client.
///
/// # Server
///
/// An RPC server *implements* the `Service` trait. Requests received by the
/// server over the network are deserialized and then passed as an argument to the
/// server value. The returned response is sent back over the network.
///
/// As an example, here is how an HTTP request is processed by a server:
///
/// ```rust
/// #![feature(type_alias_impl_trait)]
///
/// use std::future::Future;
///
/// use http::{Request, Response, StatusCode};
/// use motore::Service;
///
/// struct HelloWorld;
///
/// impl<Cx> Service<Cx, Request<Vec<u8>>> for HelloWorld
/// where
///     Cx: 'static + Send,
/// {
///     type Response = Response<Vec<u8>>;
///     type Error = http::Error;
///     type Future<'cx> = impl Future<Output = Result<Self::Response, Self::Error>> + 'cx;
///
///     fn call<'cx, 's>(&'s self, _cx: &'cx mut Cx, _req: Request<Vec<u8>>) -> Self::Future<'cx>
///     where
///         's: 'cx,
///     {
///         // create the body
///         let body: Vec<u8> = "hello, world!\n".as_bytes().to_owned();
///         // Create the HTTP response
///         let resp = Response::builder()
///             .status(StatusCode::OK)
///             .body(body)
///             .expect("Unable to create `http::Response`");
///         // create a response in a future.
///         async { Ok(resp) }
///     }
/// }
/// ```
///
/// # Middleware / Layer
///
/// More often than not, all the pieces needed for writing robust, scalable
/// network applications are the same no matter the underlying protocol. By
/// unifying the API for both clients and servers in a protocol agnostic way,
/// it is possible to write middleware that provide these pieces in a
/// reusable way.
///
/// For example, you can refer to the [`motore::timeout::Timeout`][crate::timeout::Timeout] Service.
pub trait Service<Cx, Request> {
    /// Responses given by the service.
    type Response;
    /// Errors produced by the service.
    type Error;

    /// The future response value.
    type Future<'cx>: Future<Output = Result<Self::Response, Self::Error>> + Send + 'cx
    where
        Cx: 'cx,
        Self: 'cx;

    /// Process the request and return the response asynchronously.
    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: Request) -> Self::Future<'cx>
    where
        's: 'cx;
}

/// [`Service`] without need of Context.
pub trait UnaryService<Request> {
    type Response;
    type Error;

    type Future<'s>: Future<Output = Result<Self::Response, Self::Error>> + Send + 's
    where
        Self: 's;

    fn call(&self, req: Request) -> Self::Future<'_>;
}

/// A [`Clone`] + [`Send`] boxed [`Service`].
///
/// [`BoxCloneService`] turns a service into a trait object, allowing the
/// response future type to be dynamic, and allowing the service to be cloned.
///
/// This is similar to [`BoxService`](super::BoxService) except the resulting
/// service implements [`Clone`].
pub struct BoxCloneService<Cx, T, U, E> {
    raw: *mut (),
    vtable: ServiceVtable<Cx, T, U, E>,
}

unsafe impl<Cx, T, U, E> Sync for BoxCloneService<Cx, T, U, E> {}

impl<Cx, T, U, E> BoxCloneService<Cx, T, U, E> {
    /// Create a new `BoxCloneService`.
    pub fn new<S>(s: S) -> Self
    where
        S: Service<Cx, T, Response = U, Error = E> + Clone + Send + 'static,
        T: 'static,
        for<'cx> S::Future<'cx>: Send,
    {
        let raw = Box::into_raw(Box::new(s)) as *mut ();
        BoxCloneService {
            raw,
            vtable: ServiceVtable {
                call: call::<Cx, T, S>,
                clone: clone::<Cx, T, S>,
                drop: drop::<S>,
            },
        }
    }
}

impl<Cx, T, U, E> Drop for BoxCloneService<Cx, T, U, E> {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.raw) };
    }
}

impl<Cx, T, U, E> Clone for BoxCloneService<Cx, T, U, E> {
    fn clone(&self) -> Self {
        unsafe { (self.vtable.clone)(self.raw) }
    }
}

impl<Cx, T, U, E> fmt::Debug for BoxCloneService<Cx, T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxCloneService").finish()
    }
}

impl<Cx, T, U, E> Service<Cx, T> for BoxCloneService<Cx, T, U, E> {
    type Response = U;

    type Error = E;

    type Future<'cx> = BoxFuture<'cx, Result<U, E>>
    where
        Self: 'cx;

    fn call<'cx, 's>(&'s self, cx: &'cx mut Cx, req: T) -> Self::Future<'cx>
    where
        's: 'cx,
    {
        unsafe { (self.vtable.call)(self.raw, cx, req) }
    }
}

/// # Safety
///
/// The contained `Service` must be `Send` required by the bounds of `new` and `clone`.
unsafe impl<Cx, T, U, E> Send for BoxCloneService<Cx, T, U, E> {}

struct ServiceVtable<Cx, T, U, E> {
    call: unsafe fn(raw: *mut (), cx: &mut Cx, req: T) -> BoxFuture<'_, Result<U, E>>,
    clone: unsafe fn(raw: *mut ()) -> BoxCloneService<Cx, T, U, E>,
    drop: unsafe fn(raw: *mut ()),
}

fn call<Cx, Req, S>(
    raw: *mut (),
    cx: &mut Cx,
    req: Req,
) -> BoxFuture<'_, Result<S::Response, S::Error>>
where
    Req: 'static,
    S: Service<Cx, Req> + 'static,
    for<'cx> S::Future<'cx>: Send,
{
    let fut = S::call(unsafe { (raw as *mut S).as_mut().unwrap() }, cx, req);
    Box::pin(fut)
}

fn clone<Cx, Req, S: Clone + Send + Service<Cx, Req> + 'static>(
    raw: *mut (),
) -> BoxCloneService<Cx, Req, S::Response, S::Error>
where
    Req: 'static,
    for<'cx> S::Future<'cx>: Send,
{
    BoxCloneService::new(S::clone(unsafe { (raw as *mut S).as_ref().unwrap() }))
}

fn drop<S>(raw: *mut ()) {
    unsafe { Box::from_raw(raw as *mut S) };
}

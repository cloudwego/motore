#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/cloudwego/motore/raw/main/.github/assets/logo.png?sanitize=true"
)]

//! # Overview
//!
//! Motore is a library of the basic abstractions of middlewares for building
//! robust networking clients and servers.
//!
//! Motore is greatly inspired by the [`tower`] crate, and we have used / modified
//! some of `Tower`'s code and documentation. We really appreciate the work that
//! the `Tower` team have done.
//!
//! `Tower` is licensed under the MIT license, and a copy can be found under
//! the [`licenses/tower`](https://github.com/cloudwego/motore/tree/main/licenses/tower) directory.
//!
//! Motore provides a simple core abstraction, the [`Service`] trait, which
//! represents an asynchronous function taking a request and returning either a
//! response or an error. This abstraction can be used to model both clients and
//! servers.
//!
//! Generic components, like `timeouts`, `rate limiting`, and `load balancing`,
//! can be modeled as [`Service`]s that wrap some inner service and apply
//! additional behavior before or after the inner service is called. This allows
//! implementing these components in a protocol-agnostic, composable way. Typically,
//! such services are referred to as _middleware_.
//!
//! An additional abstraction, the [`Layer`] trait, is used to compose
//! middleware with [`Service`]s. If a [`Service`] can be thought of as an
//! asynchronous function from a request type to a response type, a [`Layer`] is
//! a function taking a [`Service`] of one type and returning a [`Service`] of a
//! different type. The [`ServiceBuilder`] type is used to add middleware to a
//! service by composing it with multiple [`Layer`]s.
//!
//! [`tower`]: https://crates.io/crates/tower
//! [`Layer`]: crate::layer::Layer
//! [`ServiceBuilder`]: crate::builder::ServiceBuilder

pub mod builder;
pub mod layer;
pub mod make;
pub mod service;
pub mod timeout;
pub mod utils;
pub use motore_macros::service;
pub use service::{BoxCloneService, Service, ServiceExt, UnaryService};

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[allow(unreachable_pub)]
mod sealed {
    pub trait Sealed<T> {}
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_service_macro() {
        pub struct Context;
        pub struct Service<S>(S);

        #[crate::service]
        impl<S, Req> crate::Service<Context, Req> for Service<S>
        where
            Req: 'static,
            S: crate::Service<Context, Req>,
        {
            async fn call(&self, _cx: &mut Context, _req: Req) -> Result<S::Response, S::Error> {
                todo!()
            }
        }
    }
}

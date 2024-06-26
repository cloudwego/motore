![Motore](https://github.com/cloudwego/motore/raw/main/.github/assets/logo.png?sanitize=true)

[![Crates.io](https://img.shields.io/crates/v/motore)](https://crates.io/crates/motore)
[![Documentation](https://docs.rs/motore/badge.svg)](https://docs.rs/motore)
[![License](https://img.shields.io/crates/l/motore)](#license)
[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/cloudwego/motore/actions/workflows/ci.yaml/badge.svg
[actions-url]: https://github.com/cloudwego/motore/actions

Motore is an async middleware abstraction powered by AFIT and RPITIT.

Around Motore, we build modular and reusable components for building robust networking clients and servers.

Motore is greatly inspired by [`Tower`][tower].

[tower]: https://github.com/tower-rs/tower

## Overview

Motore uses AFIT and RPITIT to reduce the mental burden of writing asynchronous code, especially to avoid the overhead of `Box` to make people less anxious.

The core abstraciton of Motore is the `Service` trait:

```rust
pub trait Service<Cx, Request> {
    /// Responses given by the service.
    type Response;
    /// Errors produced by the service.
    type Error;

    /// Process the request and return the response asynchronously.
    async fn call(&self, cx: &mut Cx, req: Request) -> Result<Self::Response, Self::Error>;
}
```

## Getting Started

Combing AFIT and RPITIT together, we can write asynchronous code in a very concise and readable way.

```rust
pub struct Timeout<S> {
    inner: S,
    duration: Duration,
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

    async fn call(&self, cx: &mut Cx, req: Req) -> Result<Self::Response, Self::Error> {
        let sleep = tokio::time::sleep(self.duration);
        tokio::select! {
            r = self.inner.call(cx, req) => {
                r.map_err(Into::into)
            },
            _ = sleep => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "service time out").into()),
        }
    }
}
```

We also provided the `#[motore::service]` macro to make writing a `Serivce` more async-native:

```rust
use motore::service;

pub struct S<I> {
    inner: I,
}

#[service]
impl<Cx, Req, I> Service<Cx, Req> for S<I>
where
   Req: Send + 'static,
   I: Service<Cx, Req> + Send + 'static + Sync,
   Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<I::Response, I::Error> {
        self.inner.call(cx, req).await
    }
}
```

## FAQ

### Where's the `poll_ready`(a.k.a. backpressure)?

https://www.cloudwego.io/docs/volo/faq/#where-did-poll_ready-backpressure-go

## License

Motore is dual-licensed under the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](https://github.com/cloudwego/motore/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/cloudwego/motore/blob/main/LICENSE-APACHE) for details.

## Credits

We have used some third party components, and we thank them for their work.

For the full list, you may refer to the [CREDITS.md](https://github.com/cloudwego/motore/blob/main/CREDITS.md) file.

## Community

- Email: [motore@cloudwego.io](mailto:motore@cloudwego.io)
- How to become a member: [COMMUNITY MEMBERSHIP](https://github.com/cloudwego/community/blob/main/COMMUNITY_MEMBERSHIP.md)
- Issues: [Issues](https://github.com/cloudwego/motore/issues)
- Feishu: Scan the QR code below with [Feishu](https://www.feishu.cn/) or [click this link](https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=a17m50a7-79cd-4ece-b14c-c1586e1aa636) to join our CloudWeGo Motore user group.

  <img src="https://github.com/cloudwego/motore/raw/main/.github/assets/motore-feishu-user-group.png" alt="Motore user group" width="50%" height="50%" />

use futures::Future;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{sealed::Sealed, UnaryService};

/// This trait is used to create a connection.
///
/// The connection can either be a real connection or a virtual connection,
/// which means that we only ask for something that is `AsyncRead + AsyncWrite`.
/// A typical example of a virtual connection is a HTTP/2 stream.
pub trait MakeConnection<Address>: Sealed<(Address,)> {
    type Connection: AsyncRead + AsyncWrite + Unpin + Send;
    type Error;
    type Future<'s>: Future<Output = Result<Self::Connection, Self::Error>> + Send + 's
    where
        Self: 's,
        Address: 's;

    fn make_connection(&mut self, req: Address) -> Self::Future<'_>;
}

impl<S, Address> Sealed<(Address,)> for S where S: UnaryService<Address> {}

impl<S, Address> MakeConnection<Address> for S
where
    S: UnaryService<Address>,
    S::Response: AsyncRead + AsyncWrite + Unpin + Send,
{
    type Connection = S::Response;
    type Error = S::Error;
    type Future<'s> = impl Future<Output = Result<Self::Connection, Self::Error>> + Send + 's where Self: 's, Address:'s;

    fn make_connection(&mut self, addr: Address) -> Self::Future<'_> {
        self.call(addr)
    }
}

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

    #[cfg(feature = "service_send")]
    fn make_connection(
        &self,
        req: Address,
    ) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send;
    #[cfg(not(feature = "service_send"))]
    fn make_connection(
        &self,
        req: Address,
    ) -> impl Future<Output = Result<Self::Connection, Self::Error>>;
}

impl<S, Address> Sealed<(Address,)> for S where S: UnaryService<Address> {}

impl<S, Address> MakeConnection<Address> for S
where
    S: UnaryService<Address>,
    S::Response: AsyncRead + AsyncWrite + Unpin + Send,
{
    type Connection = S::Response;
    type Error = S::Error;

    #[cfg(feature = "service_send")]
    fn make_connection(
        &self,
        addr: Address,
    ) -> impl Future<Output = Result<Self::Connection, Self::Error>> + Send {
        self.call(addr)
    }
    #[cfg(not(feature = "service_send"))]
    fn make_connection(
        &self,
        addr: Address,
    ) -> impl Future<Output = Result<Self::Connection, Self::Error>> {
        self.call(addr)
    }
}

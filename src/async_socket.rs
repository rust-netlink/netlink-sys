// SPDX-License-Identifier: MIT

use std::{
    io,
    task::{Context, Poll},
};

use crate::{Socket, SocketAddr};

/// Trait to support different async backends
pub trait AsyncSocket: Sized + Unpin {
    /// Access underyling [`Socket`]
    fn socket_ref(&self) -> &Socket;

    /// Mutable access to the underlying [`Socket`].
    ///
    /// # Safety
    ///
    /// The returned socket must not be dropped or replaced, and its file
    /// descriptor must not be closed or changed. Async backends keep the
    /// original descriptor registered with their reactor, so doing any of
    /// these through the returned reference would leave the registration
    /// dangling.
    unsafe fn socket_mut(&mut self) -> &mut Socket;

    /// Wrapper for [`Socket::new`]
    fn new(protocol: isize) -> io::Result<Self>;

    /// Polling wrapper for [`Socket::send`]
    fn poll_send(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;

    /// Polling wrapper for [`Socket::send_to`]
    fn poll_send_to(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> Poll<io::Result<usize>>;

    /// Polling wrapper for [`Socket::recv`]
    ///
    /// Passes 0 for flags and returns the number of bytes read.
    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;

    /// Polling wrapper for [`Socket::recv_from`]
    ///
    /// Passes 0 for flags and returns the number of bytes read along with the
    /// sender address.
    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>>;

    /// Polling wrapper for [`Socket::recv_from_full`]
    ///
    /// Passes 0 for flags and returns the received data along with the sender
    /// address.
    fn poll_recv_from_full(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(Vec<u8>, SocketAddr)>>;
}

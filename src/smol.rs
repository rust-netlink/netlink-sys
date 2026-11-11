// SPDX-License-Identifier: MIT

use std::{
    io,
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, RawFd},
    task::{Context, Poll, ready},
};

use async_io::Async;

use log::trace;

use crate::{AsyncSocket, Socket, SocketAddr};

/// An I/O object representing a Netlink socket.
pub struct SmolSocket(Async<Socket>);

impl FromRawFd for SmolSocket {
    /// # Safety
    ///
    /// `fd` must be an open netlink socket file descriptor. Ownership of `fd`
    /// is transferred to the returned `SmolSocket`, which will close it on
    /// drop. The caller must not use or close `fd` afterwards.
    ///
    /// # Panics
    ///
    /// Panics if the socket cannot be set non-blocking or registered with the
    /// async-io reactor, because `FromRawFd` cannot return an error.
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        // SAFETY: The caller of `from_raw_fd` guarantees that `fd` is a
        // valid, open file descriptor and transfers ownership of it to us.
        let socket = unsafe { Socket::from_raw_fd(fd) };
        // `Async::new` puts the socket in non-blocking mode and registers it
        // with the reactor.
        SmolSocket(Async::new(socket).expect(
            "failed to set non-blocking mode or register netlink socket \
                 with async-io",
        ))
    }
}

impl AsRawFd for SmolSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.get_ref().as_raw_fd()
    }
}

impl AsFd for SmolSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.get_ref().as_fd()
    }
}

// async_io::Async<..>::{read,write}_with[_mut] functions try IO first,
// and only register context if it would block.
// replicate this in these poll functions:
impl SmolSocket {
    fn poll_write_with<F, R>(
        &self,
        cx: &mut Context<'_>,
        mut op: F,
    ) -> Poll<io::Result<R>>
    where
        F: FnMut(&Self) -> io::Result<R>,
    {
        loop {
            match op(self) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
            // try again if writable now, otherwise come back later:
            ready!(self.0.poll_writable(cx))?;
        }
    }

    fn poll_read_with<F, R>(
        &self,
        cx: &mut Context<'_>,
        mut op: F,
    ) -> Poll<io::Result<R>>
    where
        F: FnMut(&Self) -> io::Result<R>,
    {
        loop {
            match op(self) {
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                res => return Poll::Ready(res),
            }
            // try again if readable now, otherwise come back later:
            ready!(self.0.poll_readable(cx))?;
        }
    }
}

impl AsyncSocket for SmolSocket {
    fn socket_ref(&self) -> &Socket {
        self.0.get_ref()
    }

    /// Mutable access to the underlying [`Socket`].
    ///
    /// # Safety
    ///
    /// The socket is registered with the async-io reactor. Do not drop or
    /// replace it through the returned reference
    /// (`async_io::Async::get_mut` requires that the inner I/O source is not
    /// dropped through the reference).
    unsafe fn socket_mut(&mut self) -> &mut Socket {
        // SAFETY: The caller of this `unsafe fn` guarantees that the inner
        // I/O source is not dropped or replaced through the returned
        // reference, as required by `Async::get_mut`.
        unsafe { self.0.get_mut() }
    }

    fn new(protocol: isize) -> io::Result<Self> {
        let socket = Socket::new(protocol)?;
        Ok(Self(Async::new(socket)?))
    }

    fn poll_send(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.poll_write_with(cx, |this| this.socket_ref().send(buf, 0))
    }

    fn poll_send_to(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> Poll<io::Result<usize>> {
        self.poll_write_with(cx, |this| this.socket_ref().send_to(buf, addr, 0))
    }

    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.poll_read_with(cx, |this| this.socket_ref().recv(buf, 0))
    }

    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        self.poll_read_with(cx, |this| {
            let x = this.socket_ref().recv_from(buf, 0);
            trace!("poll_recv_from: {:?}", x);
            x
        })
    }

    fn poll_recv_from_full(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(Vec<u8>, SocketAddr)>> {
        self.poll_read_with(cx, |this| this.socket_ref().recv_from_full())
    }
}

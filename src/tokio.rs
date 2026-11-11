// SPDX-License-Identifier: MIT

use std::{
    io,
    os::unix::io::{AsRawFd, FromRawFd, RawFd},
    task::{Context, Poll},
};

use futures_util::ready;
use log::trace;
use tokio::io::unix::AsyncFd;

use crate::{AsyncSocket, Socket, SocketAddr};

/// An I/O object representing a Netlink socket.
pub struct TokioSocket(AsyncFd<Socket>);

impl FromRawFd for TokioSocket {
    /// # Safety
    ///
    /// `fd` must be an open netlink socket file descriptor. Ownership of `fd`
    /// is transferred to the returned `TokioSocket`, which will close it on
    /// drop. The caller must not use or close `fd` afterwards.
    ///
    /// # Panics
    ///
    /// Panics if the socket cannot be set non-blocking or registered with the
    /// current Tokio runtime, because `FromRawFd` cannot return an error.
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        // SAFETY: The caller of `from_raw_fd` guarantees that `fd` is a
        // valid, open file descriptor and transfers ownership of it to us.
        let socket = unsafe { Socket::from_raw_fd(fd) };
        socket
            .set_nonblocking(true)
            .expect("failed to set netlink socket non-blocking");
        // SAFETY: `socket` owns a valid open netlink fd and keeps it open and
        // unchanged for the lifetime of the `AsyncFd`.
        TokioSocket(
            unsafe { AsyncFd::register(socket) }
                .expect("failed to register netlink socket with tokio"),
        )
    }
}

impl AsRawFd for TokioSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.get_ref().as_raw_fd()
    }
}

impl AsyncSocket for TokioSocket {
    fn socket_ref(&self) -> &Socket {
        self.0.get_ref()
    }

    /// Mutable access to the underlying [`Socket`].
    ///
    /// Do not replace the socket through the returned reference; the fd must
    /// remain the same for the lifetime of the Tokio registration.
    fn socket_mut(&mut self) -> &mut Socket {
        self.0.get_mut()
    }

    fn new(protocol: isize) -> io::Result<Self> {
        let socket = Socket::new(protocol)?;
        socket.set_nonblocking(true)?;
        // SAFETY: `socket` owns a valid open netlink fd and keeps it open and
        // unchanged for the lifetime of the `AsyncFd`.
        Ok(Self(unsafe { AsyncFd::register(socket) }?))
    }

    fn poll_send(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            // Check if the socket it writable. If
            // AsyncFd::poll_write_ready returns NotReady, it will
            // already have arranged for the current task to be
            // notified when the socket becomes writable, so we can
            // just return Pending
            let mut guard = ready!(self.0.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().send(buf, 0)) {
                Ok(x) => return Poll::Ready(x),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_send_to(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> Poll<io::Result<usize>> {
        loop {
            let mut guard = ready!(self.0.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().send_to(buf, addr, 0)) {
                Ok(x) => return Poll::Ready(x),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_recv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            // Check if the socket is readable. If not,
            // AsyncFd::poll_read_ready would have arranged for the
            // current task to be polled again when the socket becomes
            // readable, so we can just return Pending
            let mut guard = ready!(self.0.poll_read_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().recv(buf, 0)) {
                Ok(x) => return Poll::Ready(x),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        loop {
            trace!("poll_recv_from called");
            let mut guard = ready!(self.0.poll_read_ready(cx))?;
            trace!("poll_recv_from socket is ready for reading");

            match guard.try_io(|inner| inner.get_ref().recv_from(buf, 0)) {
                Ok(x) => {
                    trace!("poll_recv_from {:?} bytes read", x);
                    return Poll::Ready(x);
                }
                Err(_would_block) => {
                    trace!("poll_recv_from socket would block");
                    continue;
                }
            }
        }
    }

    fn poll_recv_from_full(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<(Vec<u8>, SocketAddr)>> {
        loop {
            trace!("poll_recv_from_full called");
            let mut guard = ready!(self.0.poll_read_ready(cx))?;
            trace!("poll_recv_from_full socket is ready for reading");

            match guard.try_io(|inner| inner.get_ref().recv_from_full()) {
                Ok(x) => {
                    trace!("poll_recv_from_full {:?} bytes read", x);
                    return Poll::Ready(x);
                }
                Err(_would_block) => {
                    trace!("poll_recv_from_full socket would block");
                    continue;
                }
            }
        }
    }
}

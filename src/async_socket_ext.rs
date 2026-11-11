// SPDX-License-Identifier: MIT

//! Awaitable extensions for [`AsyncSocket`].
//!
//! The [`AsyncSocketExt`] trait provides futures built on top of the polling
//! methods from [`AsyncSocket`].

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{AsyncSocket, SocketAddr};

/// Support trait for [`AsyncSocket`]
///
/// Provides awaitable variants of the poll functions from [`AsyncSocket`].
pub trait AsyncSocketExt: AsyncSocket {
    /// `async fn send(&self, buf: &[u8]) -> io::Result<usize>`
    fn send<'a, 'b>(&'a self, buf: &'b [u8]) -> PollSend<'a, 'b, Self> {
        PollSend { socket: self, buf }
    }

    /// `async fn send(&self, buf: &[u8]) -> io::Result<usize>`
    fn send_to<'a, 'b>(
        &'a self,
        buf: &'b [u8],
        addr: &'b SocketAddr,
    ) -> PollSendTo<'a, 'b, Self> {
        PollSendTo {
            socket: self,
            buf,
            addr,
        }
    }

    /// `async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>`
    fn recv<'a, 'b>(&'a self, buf: &'b mut [u8]) -> PollRecv<'a, 'b, Self> {
        PollRecv { socket: self, buf }
    }

    /// `async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize,
    /// SocketAddr)>`
    fn recv_from<'a, 'b>(
        &'a self,
        buf: &'b mut [u8],
    ) -> PollRecvFrom<'a, 'b, Self> {
        PollRecvFrom { socket: self, buf }
    }

    /// `async fn recv_from_full(&self) -> io::Result<(Vec<u8>, SocketAddr)>`
    fn recv_from_full(&self) -> PollRecvFromFull<'_, Self> {
        PollRecvFromFull { socket: self }
    }
}

impl<S: AsyncSocket> AsyncSocketExt for S {}

/// Future returned by [`AsyncSocketExt::send`].
pub struct PollSend<'a, 'b, S> {
    socket: &'a S,
    buf: &'b [u8],
}

impl<S> Future for PollSend<'_, '_, S>
where
    S: AsyncSocket,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        this.socket.poll_send(cx, this.buf)
    }
}

/// Future returned by [`AsyncSocketExt::send_to`].
pub struct PollSendTo<'a, 'b, S> {
    socket: &'a S,
    buf: &'b [u8],
    addr: &'b SocketAddr,
}

impl<S> Future for PollSendTo<'_, '_, S>
where
    S: AsyncSocket,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        this.socket.poll_send_to(cx, this.buf, this.addr)
    }
}

/// Future returned by [`AsyncSocketExt::recv`].
pub struct PollRecv<'a, 'b, S> {
    socket: &'a S,
    buf: &'b mut [u8],
}

impl<S> Future for PollRecv<'_, '_, S>
where
    S: AsyncSocket,
{
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        this.socket.poll_recv(cx, this.buf)
    }
}

/// Future returned by [`AsyncSocketExt::recv_from`].
pub struct PollRecvFrom<'a, 'b, S> {
    socket: &'a S,
    buf: &'b mut [u8],
}

impl<S> Future for PollRecvFrom<'_, '_, S>
where
    S: AsyncSocket,
{
    type Output = io::Result<(usize, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        this.socket.poll_recv_from(cx, this.buf)
    }
}

/// Future returned by [`AsyncSocketExt::recv_from_full`].
pub struct PollRecvFromFull<'a, S> {
    socket: &'a S,
}

impl<S> Future for PollRecvFromFull<'_, S>
where
    S: AsyncSocket,
{
    type Output = io::Result<(Vec<u8>, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = Pin::into_inner(self);
        this.socket.poll_recv_from_full(cx)
    }
}

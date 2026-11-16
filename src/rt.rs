// SPDX-License-Identifier: MIT

use std::{
    io,
    os::unix::io::{AsFd, AsRawFd, BorrowedFd, RawFd},
    task::{Context, Poll, ready},
};

use async_io::Async;

use log::trace;

use crate::{AsyncSocket, Socket, SocketAddr};

/// An I/O object representing a Netlink socket. Any runtime that
/// drives the `async-io` reactor can be used; `compio`, `glommio` &
/// `monoio` are currently tested.
pub struct RtAsyncSocket(Async<Socket>);

impl AsRawFd for RtAsyncSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.0.get_ref().as_raw_fd()
    }
}

impl AsFd for RtAsyncSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.get_ref().as_fd()
    }
}

// async_io::Async<..>::{read,write}_with[_mut] functions try IO first,
// and only register context if it would block.
// replicate this in these poll functions:
impl RtAsyncSocket {
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

impl AsyncSocket for RtAsyncSocket {
    fn socket_ref(&self) -> &Socket {
        self.0.get_ref()
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

#[cfg(test)]
mod tests {
    use libc::NETLINK_ROUTE;

    use netlink_packet_core::{
        NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
    };
    use netlink_packet_route::{link::LinkMessage, RouteNetlinkMessage};
    use crate::{
        AsyncSocket, AsyncSocketExt, RtAsyncSocket, SocketAddr,
    };

    async fn dump_links() -> Option<LinkMessage> {
        let socket =
            RtAsyncSocket::new(NETLINK_ROUTE as isize).expect("failed to create SmolSocket");

        let local_addr = socket
            .socket_ref()
            .bind_auto()
            .expect("failed to bind SmolSocket");

        assert_ne!(local_addr.port_number(), 0);

        let mut packet = NetlinkMessage::new(
            NetlinkHeader::default(),
            NetlinkPayload::from(RouteNetlinkMessage::GetLink(LinkMessage::default())),
        );
        packet.header.flags = NLM_F_DUMP | NLM_F_REQUEST;
        packet.header.sequence_number = 1;
        packet.finalize();

        let mut buf = vec![0u8; packet.buffer_len()];
        packet.serialize(&mut buf[..]);

        // Send

        let kernel_addr = SocketAddr::new(0, 0);
        let sent = socket
            .send_to(&buf, &kernel_addr)
            .await
            .expect("failed to send packet");

        assert_eq!(sent, buf.len());

        // Recv

        let mut links = Vec::new();
        let mut done = false;

        while !done {
            let (response_bytes, sender) = socket
                .recv_from_full()
                .await
                .expect("failed to receive response from kernel");

            assert_eq!(sender.port_number(), 0);

            let mut offset = 0;
            while offset < response_bytes.len() {
                let rx_packet: NetlinkMessage<RouteNetlinkMessage> =
                    NetlinkMessage::deserialize(&response_bytes[offset..])
                        .expect("failed to deserialize NetlinkMessage");

                let packet_len = rx_packet.header.length as usize;
                assert!(packet_len > 0);

                match rx_packet.payload {
                    NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(link)) => {
                        links.push(link);
                    }
                    NetlinkPayload::Done(_) => {
                        done = true;
                        break;
                    }
                    NetlinkPayload::Error(err) => {
                        panic!("Received netlink error: {:?}", err);
                    }
                    _ => {}
                }

                offset += packet_len;
            }
        }

        assert!(!links.is_empty(), "expected at least one link");

        let lo = links.into_iter()
            .filter(|link| link.header.index == 1)
            .next()
            .expect("Expect loopback link with index 1");
        Some(lo)
    }

    #[test]
    fn test_smol() {
        assert!(smol::block_on(dump_links()).is_some());
    }

    #[compio::test]
    async fn test_compio() {
        assert!(dump_links().await.is_some());
    }

    #[test]
    fn test_glommio() {
        let ex = glommio::LocalExecutorBuilder::new(glommio::Placement::Fixed(0)).make().unwrap();
        ex.run(async move {
            assert!(dump_links().await.is_some());
        });
    }

    #[monoio::test]
    async fn test_monoio() {
        assert!(dump_links().await.is_some());
    }
}

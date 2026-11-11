// SPDX-License-Identifier: MIT

use std::{
    io::{Error, ErrorKind, Result},
    mem,
    os::{
        fd::{AsFd, BorrowedFd, FromRawFd, IntoRawFd},
        unix::io::{AsRawFd, RawFd},
    },
};

use crate::SocketAddr;

/// A netlink socket.
///
/// # Example
///
/// In this example we:
///
/// 1. open a new socket
/// 2. send a message to the kernel
/// 3. read the reponse
///
/// ```rust
/// use netlink_sys::{Socket, SocketAddr};
/// use std::process;
///
/// // open a new socket for the NETLINK_ROUTE subsystem (see "man 7 rtnetlink")
/// let socket = Socket::new(libc::NETLINK_ROUTE as isize).unwrap();
/// // address of the remote peer we'll send a message to. This particular address is for the kernel
/// let kernel_addr = SocketAddr::new(0, 0);
/// // this is a valid message for listing the network links on the system
/// let pkt = vec![
///     0x14, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x03, 0xfd, 0xfe, 0x38, 0x5c, 0x00, 0x00, 0x00,
///     0x00, 0x00, 0x00, 0x00, 0x00,
/// ];
/// // send the message to the kernel
/// let n_sent = socket.send_to(&pkt[..], &kernel_addr, 0).unwrap();
/// assert_eq!(n_sent, pkt.len());
/// // buffer for receiving the response
/// let mut buf = vec![0; 4096];
/// loop {
///     // receive a datagram
///     let (n_received, sender_addr) = socket.recv_from(&mut buf, 0).unwrap();
///     assert_eq!(sender_addr, kernel_addr);
///     println!("received datagram {:?}", &buf[..n_received]);
///     if buf[4] == 2 && buf[5] == 0 {
///         println!("the kernel responded with an error");
///         return;
///     }
///     if buf[4] == 3 && buf[5] == 0 {
///         println!("end of dump");
///         return;
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Socket(RawFd);

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl AsFd for Socket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: `Socket` exclusively owns a valid open file descriptor for
        // the duration of the borrow, and `BorrowedFd` is tied to `&self` so
        // the descriptor cannot be closed while it is borrowed.
        unsafe { BorrowedFd::borrow_raw(self.0) }
    }
}

impl IntoRawFd for Socket {
    /// Consumes the `Socket` and returns its raw file descriptor.
    ///
    /// Ownership of the descriptor is transferred to the caller, who is
    /// responsible for closing it. This method does not close the descriptor.
    fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        std::mem::forget(self);
        fd
    }
}

impl FromRawFd for Socket {
    /// # Safety
    ///
    /// `fd` must be an open netlink socket file descriptor. Ownership of `fd`
    /// is transferred to the returned `Socket`, which will close it on drop.
    /// The caller must not use or close `fd` afterwards.
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Socket(fd)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        // SAFETY: `Socket` owns a valid open file descriptor, and this is its
        // only owner, so closing it exactly once here cannot race with another
        // close or leave the descriptor usable elsewhere.
        unsafe { libc::close(self.as_raw_fd()) };
    }
}

impl Socket {
    /// Open a new socket for the given netlink subsystem. `protocol` is the
    /// netlink protocol number, for example `libc::NETLINK_ROUTE`.
    pub fn new(protocol: isize) -> Result<Self> {
        // SAFETY: `socket` takes only plain integer arguments and no pointer
        // arguments, so every argument value is valid to pass; it returns a
        // new descriptor or -1.
        let res = unsafe {
            libc::socket(
                libc::PF_NETLINK,
                libc::SOCK_DGRAM | libc::SOCK_CLOEXEC,
                protocol as libc::c_int,
            )
        };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(Socket(res))
    }

    /// Bind the socket to the given address
    pub fn bind(&self, addr: &SocketAddr) -> Result<()> {
        let (addr_ptr, addr_len) = addr.as_raw();
        // SAFETY: `addr_ptr` points to a live `sockaddr_nl` inside `addr` and
        // `addr_len` is its size. The socket descriptor is valid for the
        // duration of the call.
        let res = unsafe { libc::bind(self.as_raw_fd(), addr_ptr, addr_len) };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    /// Bind the socket to an address assigned by the kernel, and return that
    /// address.
    pub fn bind_auto(&self) -> Result<SocketAddr> {
        let mut addr = SocketAddr::new(0, 0);
        self.bind(&addr)?;
        self.get_address(&mut addr)?;
        Ok(addr)
    }

    /// Get the socket address
    pub fn get_address(&self, addr: &mut SocketAddr) -> Result<()> {
        let (addr_ptr, mut addr_len) = addr.as_raw_mut();
        let addr_len_copy = addr_len;
        let addr_len_ptr = &mut addr_len as *mut libc::socklen_t;
        // SAFETY: `addr_ptr` points to a live `sockaddr_nl` that is valid for
        // writes of `addr_len` bytes, and `addr_len_ptr` points to a live
        // length variable. The socket descriptor is valid for the call.
        let res = unsafe {
            libc::getsockname(self.as_raw_fd(), addr_ptr, addr_len_ptr)
        };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        validate_len(addr_len as usize, addr_len_copy as usize, "getsockname")?;
        Ok(())
    }

    // when building with --features smol we don't need this
    #[allow(dead_code)]
    /// Make this socket non-blocking
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        let mut nonblocking = nonblocking as libc::c_int;
        // SAFETY: `FIONBIO` expects a pointer to a `c_int`, which
        // `&mut nonblocking` provides for the duration of the call. The socket
        // descriptor is valid for the call.
        let res = unsafe {
            libc::ioctl(self.as_raw_fd(), libc::FIONBIO, &mut nonblocking)
        };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    /// Connect the socket to the given address. Netlink is a connection-less
    /// protocol, so a socket can communicate with multiple peers with the
    /// [`Socket::send_to`] and [`Socket::recv_from`] methods. However, if the
    /// socket only needs to communicate with one peer, it is convenient not
    /// to have to bother with the peer address. This is what `connect` is
    /// for. After calling `connect`, [`Socket::send`] and [`Socket::recv`]
    /// respectively send and receive datagrams to and from `remote_addr`.
    ///
    /// # Examples
    ///
    /// In this example we:
    ///
    /// 1. open a socket
    /// 2. connect it to the kernel with [`Socket::connect`]
    /// 3. send a request to the kernel with [`Socket::send`]
    /// 4. read the response (which can span over several messages)
    ///    [`Socket::recv`]
    ///
    /// ```rust
    /// use netlink_sys::{Socket, SocketAddr};
    /// use std::process;
    ///
    /// let socket = Socket::new(libc::NETLINK_ROUTE as isize).unwrap();
    /// let _ = socket.bind_auto().unwrap();
    /// let kernel_addr = SocketAddr::new(0, 0);
    /// socket.connect(&kernel_addr).unwrap();
    /// // This is a valid message for listing the network links on the system
    /// let msg = vec![
    ///     0x14, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x03, 0xfd, 0xfe, 0x38, 0x5c, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x00, 0x00,
    /// ];
    /// let n_sent = socket.send(&msg[..], 0).unwrap();
    /// assert_eq!(n_sent, msg.len());
    /// // buffer for receiving the response
    /// let mut buf = vec![0; 4096];
    /// loop {
    ///     let mut n_received = socket.recv(&mut buf, 0).unwrap();
    ///     println!("received {:?}", &buf[..n_received]);
    ///     if buf[4] == 2 && buf[5] == 0 {
    ///         println!("the kernel responded with an error");
    ///         return;
    ///     }
    ///     if buf[4] == 3 && buf[5] == 0 {
    ///         println!("end of dump");
    ///         return;
    ///     }
    /// }
    /// ```
    pub fn connect(&self, remote_addr: &SocketAddr) -> Result<()> {
        // FIXME:
        //
        // Event though for SOCK_DGRAM sockets there's no IO, if our socket is
        // non-blocking, connect() might return EINPROGRESS. In theory,
        // the right way to treat EINPROGRESS would be to ignore the
        // error, and let the user poll the socket to check when it becomes
        // writable, indicating that the connection succeeded. The code already
        // exists in mio for TcpStream:
        //
        // > pub fn connect(stream: net::TcpStream, addr: &SocketAddr) ->
        // > io::Result<TcpStream> {
        // > set_non_block(stream.as_raw_fd())?;
        // > match stream.connect(addr) {
        // > Ok(..) => {}
        // > Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {}
        // > Err(e) => return Err(e),
        // > }
        // > Ok(TcpStream {  inner: stream })
        // > }
        //
        // In practice, since the connection does not require any IO for
        // SOCK_DGRAM sockets, it almost never returns EINPROGRESS and
        // so for now, we just return whatever libc::connect returns. If
        // it returns EINPROGRESS, the caller will have to handle the error
        // themself
        //
        // Refs:
        //
        // - https://stackoverflow.com/a/14046386/1836144
        // - https://lists.isc.org/pipermail/bind-users/2009-August/077527.html
        let (addr, addr_len) = remote_addr.as_raw();
        // SAFETY: `addr` points to a live `sockaddr_nl` inside `remote_addr`
        // and `addr_len` is its size. The socket descriptor is valid for the
        // duration of the call.
        let res = unsafe { libc::connect(self.as_raw_fd(), addr, addr_len) };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    // Most of the comments in this method come from a discussion on rust users
    // forum. [thread]: https://users.rust-lang.org/t/help-understanding-libc-call/17308/9
    //
    /// Read a datagram from the socket and return the number of bytes that have
    /// been read and the address of the sender. The data being read is
    /// copied into `buf`. If `buf` is too small, the datagram is truncated. The
    /// supported flags are the `MSG_*` described in `man 2 recvmsg`.
    ///
    /// The returned length is the length reported by `recvfrom`; when
    /// `MSG_TRUNC` is used it can exceed `buf.len()` even though no more than
    /// `buf.len()` bytes were copied.
    ///
    /// # Warning
    ///
    /// In datagram oriented protocols, `recv` and `recvfrom` receive normally
    /// only ONE datagram, but this seems not to be always true for netlink
    /// sockets: with some protocols like `NETLINK_AUDIT`, multiple netlink
    /// packets can be read with a single call.
    pub fn recv_from(
        &self,
        buf: &mut [u8],
        flags: libc::c_int,
    ) -> Result<(usize, SocketAddr)> {
        // SAFETY: `buf` is valid for writes of `buf.len()` bytes and
        // `recvfrom` writes at most that many bytes.
        self.recv_from_raw(buf.as_mut_ptr(), buf.len(), flags)
    }

    // Shared implementation for `recv_from` and `recv_from_full`. Taking a raw
    // pointer lets `recv_from_full` write directly into a `Vec`'s spare
    // capacity without zero-initializing it first.
    fn recv_from_raw(
        &self,
        buf: *mut u8,
        buf_len: usize,
        flags: libc::c_int,
    ) -> Result<(usize, SocketAddr)> {
        // Create an empty storage for the address. Note that Rust standard
        // library create a sockaddr_storage so that it works for any
        // address family, but here, we already know that we'll have a
        // Netlink address, so we can create the appropriate storage.
        // SAFETY: `sockaddr_nl` is a plain C struct made only of integers, so
        // an all-zero bit pattern is a valid value. `recvfrom` fills it in.
        let mut addr = unsafe { mem::zeroed::<libc::sockaddr_nl>() };

        // recvfrom takes a *sockaddr as parameter so that it can accept any
        // kind of address storage, so we need to create such a pointer
        // for the sockaddr_nl we just initialized.
        //
        //                     Create a raw pointer to        Cast our raw
        // pointer to a                     our storage. We cannot
        // generic pointer to *sockaddr                     pass it to
        // recvfrom yet.       that recvfrom can use
        // ^                              ^
        // |                              |
        // +--------------+---------------+    +---------+--------+
        //                 /                                \  /
        // \
        let addr_ptr =
            &mut addr as *mut libc::sockaddr_nl as *mut libc::sockaddr;

        // Why do we need to pass the address length? We're passing a generic
        // *sockaddr to recvfrom. Somehow recvfrom needs to make sure
        // that the address of the received packet would fit into the
        // actual type that is behind *sockaddr: it could be a sockaddr_nl but
        // also a sockaddr_in, a sockaddr_in6, or even the generic
        // sockaddr_storage that can store any address.
        let mut addrlen = mem::size_of_val(&addr);
        // recvfrom does not take the address length by value (see [thread]), so
        // we need to create a pointer to it.
        let addrlen_ptr = &mut addrlen as *mut usize as *mut libc::socklen_t;

        // Cast the *mut u8 into *mut c_void. This is equivalent to casting a
        // *char into *void.
        let buf_ptr = buf as *mut libc::c_void;
        let buf_len = buf_len as libc::size_t;

        // SAFETY: the caller guarantees that `buf_ptr` is valid for writes of
        // `buf_len` bytes. `addr_ptr` and `addrlen_ptr` point to live local
        // storage whose length matches `sockaddr_nl`, and `recvfrom` writes at
        // most `*addrlen_ptr` bytes to the address. The socket descriptor is
        // valid for the call.
        let res = unsafe {
            libc::recvfrom(
                self.as_raw_fd(),
                buf_ptr,
                buf_len,
                flags,
                addr_ptr,
                addrlen_ptr,
            )
        };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok((res as usize, SocketAddr(addr)))
    }

    /// For a connected socket, `recv` reads a datagram from the socket. The
    /// sender is the remote peer the socket is connected to (see
    /// [`Socket::connect`]). See also [`Socket::recv_from`].
    ///
    /// The returned length is the length reported by `recv`; when `MSG_TRUNC`
    /// is used it can exceed `buf.len()` even though no more than `buf.len()`
    /// bytes were copied.
    pub fn recv(&self, buf: &mut [u8], flags: libc::c_int) -> Result<usize> {
        let buf_ptr = buf.as_mut_ptr() as *mut libc::c_void;
        let buf_len = buf.len() as libc::size_t;

        // SAFETY: `buf_ptr` points to `buf`, which is valid for writes of
        // `buf_len` bytes, and `recv` writes at most that many bytes. The
        // socket descriptor is valid for the call.
        let res =
            unsafe { libc::recv(self.as_raw_fd(), buf_ptr, buf_len, flags) };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(res as usize)
    }

    /// Receive a full message. Unlike [`Socket::recv_from`], which truncates
    /// messages that exceed the length of the buffer passed as argument,
    /// this method always reads a whole message, no matter its size.
    pub fn recv_from_full(&self) -> Result<(Vec<u8>, SocketAddr)> {
        // Peek. A zero-length buffer combined with `MSG_TRUNC` makes the
        // kernel return the full datagram length without copying any data.
        let mut buf: Vec<u8> = Vec::new();
        let (peek_len, _) = self.recv_from_raw(
            buf.as_mut_ptr(),
            buf.len(),
            libc::MSG_PEEK | libc::MSG_TRUNC,
        )?;

        // Receive directly into the spare capacity so the buffer does not
        // need to be zero-initialized first.
        buf.reserve(peek_len);
        let spare = buf.spare_capacity_mut();
        let (rlen, addr) = self.recv_from_raw(
            spare.as_mut_ptr().cast::<u8>(),
            spare.len(),
            0,
        )?;
        validate_len(rlen, peek_len, "recv_from_full")?;
        // SAFETY: `recv_from_raw` wrote `rlen` bytes into the spare capacity
        // and the length check above guarantees `rlen == peek_len`, which is
        // at most `spare.len()`.
        unsafe {
            buf.set_len(rlen);
        }
        Ok((buf, addr))
    }

    /// Send the given buffer `buf` to the remote peer with address `addr`. The
    /// supported flags are the `MSG_*` values documented in `man 2 send`.
    pub fn send_to(
        &self,
        buf: &[u8],
        addr: &SocketAddr,
        flags: libc::c_int,
    ) -> Result<usize> {
        let (addr_ptr, addr_len) = addr.as_raw();
        let buf_ptr = buf.as_ptr() as *const libc::c_void;
        let buf_len = buf.len() as libc::size_t;

        // SAFETY: `buf_ptr` points to `buf`, which is valid for reads of
        // `buf_len` bytes. `addr_ptr` points to a live `sockaddr_nl` inside
        // `addr` and `addr_len` is its size. The socket descriptor is valid for
        // the call.
        let res = unsafe {
            libc::sendto(
                self.as_raw_fd(),
                buf_ptr,
                buf_len,
                flags,
                addr_ptr,
                addr_len,
            )
        };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(res as usize)
    }

    /// For a connected socket, `send` sends the given buffer `buf` to the
    /// remote peer the socket is connected to. See also [`Socket::connect`]
    /// and [`Socket::send_to`].
    pub fn send(&self, buf: &[u8], flags: libc::c_int) -> Result<usize> {
        let buf_ptr = buf.as_ptr() as *const libc::c_void;
        let buf_len = buf.len() as libc::size_t;

        // SAFETY: `buf_ptr` points to `buf`, which is valid for reads of
        // `buf_len` bytes. The socket descriptor is valid for the call.
        let res =
            unsafe { libc::send(self.as_raw_fd(), buf_ptr, buf_len, flags) };
        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(res as usize)
    }

    /// `NETLINK_PKTINFO`. When set, received messages include packet
    /// information describing their origin.
    pub fn set_pktinfo(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_PKTINFO,
            value,
        )
    }

    /// Return whether `NETLINK_PKTINFO` is enabled.
    pub fn get_pktinfo(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_PKTINFO,
        )?;
        Ok(res != 0)
    }

    /// Join the netlink multicast group identified by `group`.
    pub fn add_membership(&self, group: u32) -> Result<()> {
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_ADD_MEMBERSHIP,
            group,
        )
    }

    /// Leave the netlink multicast group identified by `group`.
    pub fn drop_membership(&self, group: u32) -> Result<()> {
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_DROP_MEMBERSHIP,
            group,
        )
    }

    // pub fn list_membership(&self) -> Vec<u32> {
    //     unimplemented!();
    //     // getsockopt won't be enough here, because we may need to perform 2
    // calls, and because the     // length of the list returned by
    // libc::getsockopt is returned by mutating the length     // argument,
    // which our implementation of getsockopt forbids. }

    /// `NETLINK_BROADCAST_ERROR` (since Linux 2.6.30). When not set,
    /// `netlink_broadcast()` only reports `ESRCH` errors and silently
    /// ignore `NOBUFS` errors.
    pub fn set_broadcast_error(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_BROADCAST_ERROR,
            value,
        )
    }

    /// Return whether `NETLINK_BROADCAST_ERROR` is enabled.
    pub fn get_broadcast_error(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_BROADCAST_ERROR,
        )?;
        Ok(res != 0)
    }

    /// `NETLINK_NO_ENOBUFS` (since Linux 2.6.30). This flag can be used by
    /// unicast and broadcast listeners to avoid receiving `ENOBUFS` errors.
    pub fn set_no_enobufs(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_NO_ENOBUFS,
            value,
        )
    }

    /// Return whether `NETLINK_NO_ENOBUFS` is enabled.
    pub fn get_no_enobufs(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_NO_ENOBUFS,
        )?;
        Ok(res != 0)
    }

    /// `NETLINK_LISTEN_ALL_NSID` (since Linux 4.2). When set, this socket will
    /// receive netlink notifications from  all  network  namespaces that
    /// have an nsid assigned into the network namespace where the socket
    /// has been opened. The nsid is sent to user space via an ancillary
    /// data.
    pub fn set_listen_all_namespaces(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_LISTEN_ALL_NSID,
            value,
        )
    }

    /// Return whether `NETLINK_LISTEN_ALL_NSID` is enabled.
    pub fn get_listen_all_namespaces(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_LISTEN_ALL_NSID,
        )?;
        Ok(res != 0)
    }

    /// `NETLINK_CAP_ACK` (since Linux 4.2). The kernel may fail to allocate the
    /// necessary room for the acknowledgment message back to user space.
    /// This option trims off the payload of the original netlink message.
    /// The netlink message header is still included, so the user can
    /// guess from the sequence  number which message triggered the
    /// acknowledgment.
    pub fn set_cap_ack(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_CAP_ACK,
            value,
        )
    }

    /// Return whether `NETLINK_CAP_ACK` is enabled.
    pub fn get_cap_ack(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_CAP_ACK,
        )?;
        Ok(res != 0)
    }

    /// `NETLINK_EXT_ACK`
    /// Extended ACK controls reporting of additional error/warning TLVs in
    /// NLMSG_ERROR and NLMSG_DONE messages.
    pub fn set_ext_ack(&self, value: bool) -> Result<()> {
        let value: libc::c_int = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_EXT_ACK,
            value,
        )
    }

    /// Return whether `NETLINK_EXT_ACK` is enabled.
    pub fn get_ext_ack(&self) -> Result<bool> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_EXT_ACK,
        )?;
        Ok(res != 0)
    }

    /// Sets socket receive buffer in bytes.
    ///
    /// The kernel doubles this value (to allow space for bookkeeping overhead),
    /// and this doubled value is returned by [`Self::get_rx_buf_sz`] (see
    /// socket(7)). The default value is set by the
    /// `/proc/sys/net/core/rmem_default` file, and the maximum allowed value is
    /// set by the `/proc/sys/net/core/rmem_max` file.
    pub fn set_rx_buf_sz(&self, size: usize) -> Result<()> {
        setsockopt(self.as_raw_fd(), libc::SOL_SOCKET, libc::SO_RCVBUF, size)
    }

    /// Gets socket receive buffer in bytes
    pub fn get_rx_buf_sz(&self) -> Result<usize> {
        let res = getsockopt::<libc::c_int>(
            self.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
        )?;
        Ok(res as usize)
    }

    /// Set strict input checking(`NETLINK_GET_STRICT_CHK`) in netlink route
    /// protocol. By default, `NETLINK_GET_STRICT_CHK` is not enabled.
    pub fn set_netlink_get_strict_chk(&self, value: bool) -> Result<()> {
        let value: u32 = value.into();
        setsockopt(
            self.as_raw_fd(),
            libc::SOL_NETLINK,
            libc::NETLINK_GET_STRICT_CHK,
            value,
        )
    }
}

/// Wrapper around `getsockopt`:
///
/// ```no_rust
/// int getsockopt(int socket, int level, int option_name, void *restrict option_value, socklen_t *restrict option_len);
/// ```
pub(crate) fn getsockopt<T: Copy>(
    fd: RawFd,
    level: libc::c_int,
    option: libc::c_int,
) -> Result<T> {
    // Create storage for the options we're fetching
    // SAFETY: all call sites instantiate `T` with integer socket-option types
    // for which an all-zero bit pattern is valid.
    let mut slot: T = unsafe { mem::zeroed() };

    // Create a mutable raw pointer to the storage so that getsockopt can fill
    // the value
    let slot_ptr = &mut slot as *mut T as *mut libc::c_void;

    // Let getsockopt know how big our storage is
    let mut slot_len = mem::size_of::<T>() as libc::socklen_t;

    // getsockopt takes a mutable pointer to the length, because for some
    // options like NETLINK_LIST_MEMBERSHIP where the option value is a list
    // with arbitrary length, getsockopt uses this parameter to signal how
    // big the storage needs to be.
    let slot_len_ptr = &mut slot_len as *mut libc::socklen_t;

    // SAFETY: `slot_ptr` points to `slot`, which is valid for writes of
    // `slot_len` bytes, and `slot_len_ptr` points to a live length variable.
    // `fd` is valid for the duration of the call.
    let res =
        unsafe { libc::getsockopt(fd, level, option, slot_ptr, slot_len_ptr) };
    if res < 0 {
        return Err(Error::last_os_error());
    }

    validate_len(slot_len as usize, mem::size_of::<T>(), "getsockopt")?;

    Ok(slot)
}

/// Validate a length reported by a syscall against the length we expected.
fn validate_len(actual: usize, expected: usize, operation: &str) -> Result<()> {
    if actual != expected {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "{operation} returned an unexpected length: got {actual} \
                 bytes, expected {expected} bytes"
            ),
        ));
    }
    Ok(())
}

// adapted from rust standard library
fn setsockopt<T>(
    fd: RawFd,
    level: libc::c_int,
    option: libc::c_int,
    payload: T,
) -> Result<()> {
    let payload = &payload as *const T as *const libc::c_void;
    let payload_len = mem::size_of::<T>() as libc::socklen_t;

    // SAFETY: `payload` points to a live `T` and `payload_len` is exactly the
    // size of `T`, so `setsockopt` reads no more than the object it points to.
    // `fd` is valid for the duration of the call.
    let res =
        unsafe { libc::setsockopt(fd, level, option, payload, payload_len) };
    if res < 0 {
        return Err(Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const NETLINK_ROUTE: isize = libc::NETLINK_ROUTE as isize;

    #[test]
    fn new() {
        Socket::new(NETLINK_ROUTE).unwrap();
    }

    #[test]
    fn connect() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        sock.connect(&SocketAddr::new(0, 0)).unwrap();
    }

    #[test]
    fn bind() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        sock.bind(&SocketAddr::new(4321, 0)).unwrap();
    }

    #[test]
    fn bind_auto() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        let addr = sock.bind_auto().unwrap();
        // make sure that the address we got from the kernel is there
        assert!(addr.port_number() != 0);
    }

    #[test]
    fn set_nonblocking() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        sock.set_nonblocking(true).unwrap();
        sock.set_nonblocking(false).unwrap();
    }

    #[test]
    fn options() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();

        sock.set_cap_ack(true).unwrap();
        assert!(sock.get_cap_ack().unwrap());
        sock.set_cap_ack(false).unwrap();
        assert!(!sock.get_cap_ack().unwrap());

        sock.set_no_enobufs(true).unwrap();
        assert!(sock.get_no_enobufs().unwrap());
        sock.set_no_enobufs(false).unwrap();
        assert!(!sock.get_no_enobufs().unwrap());

        sock.set_broadcast_error(true).unwrap();
        assert!(sock.get_broadcast_error().unwrap());
        sock.set_broadcast_error(false).unwrap();
        assert!(!sock.get_broadcast_error().unwrap());

        // FIXME: these require root permissions
        // sock.set_listen_all_namespaces(true).unwrap();
        // assert!(sock.get_listen_all_namespaces().unwrap());
        // sock.set_listen_all_namespaces(false).unwrap();
        // assert!(!sock.get_listen_all_namespaces().unwrap());
    }

    #[test]
    fn set_rx_buf_sz_accepts_valid_value() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        sock.set_rx_buf_sz(4096).unwrap();
        assert!(sock.get_rx_buf_sz().unwrap() >= 4096);
    }

    #[test]
    fn validate_len_accepts_matching_lengths() {
        validate_len(4, 4, "test").unwrap();
    }

    #[test]
    fn validate_len_rejects_mismatched_lengths() {
        let err = validate_len(1, 2, "test").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn getsockopt_error_is_propagated() {
        let sock = Socket::new(NETLINK_ROUTE).unwrap();
        let res = getsockopt::<libc::c_int>(
            sock.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::c_int::MAX,
        );
        assert!(res.is_err());
    }
}

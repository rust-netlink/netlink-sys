// SPDX-License-Identifier: MIT

//! Netlink sockets for Linux and FreeBSD.
//!
//! This crate provides a low-level [`Socket`] type for communicating over
//! netlink, along with optional async integrations:
//!
//! - `TokioSocket` behind the `tokio_socket` feature.
//! - `SmolSocket` behind the `smol_socket` feature.
//! - [`mio::event::Source`](https://docs.rs/mio/latest/mio/event/trait.Source.html)
//!   support for [`Socket`] behind the `mio_socket` feature.
//!
//! Netlink protocol numbers are not re-exported by this crate. Pass the
//! appropriate number (for example `libc::NETLINK_ROUTE`) to
//! [`Socket::new`](crate::Socket::new).

#![deny(missing_docs)]

mod socket;
pub use self::socket::Socket;

mod addr;
pub use self::addr::SocketAddr;

mod async_socket;
pub use self::async_socket::AsyncSocket;

pub mod async_socket_ext;
pub use self::async_socket_ext::AsyncSocketExt;

#[cfg(feature = "tokio_socket")]
mod tokio;
#[cfg(feature = "tokio_socket")]
pub use self::tokio::TokioSocket;

#[cfg(feature = "rt_socket")]
mod rt;
#[cfg(feature = "rt_socket")]
pub use self::rt::RtAsyncSocket;

#[deprecated(since = "1.0.0", note = "Use RtAsyncSocket instead")]
pub use self::rt::RtAsyncSocket as SmolSocket;


#[cfg(feature = "mio_socket")]
mod mio;

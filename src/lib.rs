// SPDX-License-Identifier: MIT

//! Netlink sockets for Linux.
//!
//! This crate provides a low-level [`Socket`] type for communicating over
//! netlink, along with optional async integrations:
//!
//! - `TokioSocket` behind the `tokio_socket` feature.
//! - `SmolSocket` behind the `smol_socket` feature.
//! - [`mio::event::Source`](https://docs.rs/mio/latest/mio/event/trait.Source.html)
//!   support for [`Socket`] behind the `mio_socket` feature.
//!
//! Netlink protocol constants are available in [`protocols`], and additional
//! kernel constants are available in [`constants`].

#![deny(missing_docs)]

pub mod constants;

/// Netlink protocol numbers.
///
/// These can be passed to [`Socket::new`](crate::Socket::new) to select the
/// netlink subsystem to communicate with.
pub mod protocols {
    pub use super::constants::{
        NETLINK_AUDIT, NETLINK_CONNECTOR, NETLINK_CRYPTO, NETLINK_DNRTMSG,
        NETLINK_ECRYPTFS, NETLINK_FIB_LOOKUP, NETLINK_FIREWALL,
        NETLINK_GENERIC, NETLINK_IP6_FW, NETLINK_ISCSI, NETLINK_KOBJECT_UEVENT,
        NETLINK_NETFILTER, NETLINK_NFLOG, NETLINK_RDMA, NETLINK_ROUTE,
        NETLINK_SCSITRANSPORT, NETLINK_SELINUX, NETLINK_SOCK_DIAG,
        NETLINK_UNUSED, NETLINK_USERSOCK, NETLINK_XFRM,
    };
}

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

#[cfg(feature = "smol_socket")]
mod smol;
#[cfg(feature = "smol_socket")]
pub use self::smol::SmolSocket;

#[cfg(feature = "mio_socket")]
mod mio;

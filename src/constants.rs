// SPDX-License-Identifier: MIT

#[cfg(not(target_os = "freebsd"))]
pub use libc::{
    NETLINK_ADD_MEMBERSHIP, NETLINK_BROADCAST_ERROR, NETLINK_CAP_ACK,
    NETLINK_DROP_MEMBERSHIP, NETLINK_EXT_ACK, NETLINK_GET_STRICT_CHK,
    NETLINK_LISTEN_ALL_NSID, NETLINK_NO_ENOBUFS, NETLINK_PKTINFO, PF_NETLINK,
    SOL_NETLINK,
};

#[cfg(target_os = "freebsd")]
pub use self::freebsd::*;

#[cfg(target_os = "freebsd")]
mod freebsd {
    use libc::c_int;

    pub const AF_NETLINK: c_int = 38;
    pub const PF_NETLINK: c_int = AF_NETLINK;
    pub const SOL_NETLINK: c_int = 270;

    pub const NETLINK_ADD_MEMBERSHIP: c_int = 1;
    pub const NETLINK_DROP_MEMBERSHIP: c_int = 2;
    pub const NETLINK_LISTEN_ALL_NSID: c_int = 8;
    pub const NETLINK_CAP_ACK: c_int = 10;
    pub const NETLINK_EXT_ACK: c_int = 11;
    pub const NETLINK_GET_STRICT_CHK: c_int = 12;
}

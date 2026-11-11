// SPDX-License-Identifier: MIT

//! This module provides a lot of netlink constants for various protocol. As we
//! add support for the various protocols, these constants will be moved to
//! their own crate.

use libc::c_int as int;

/// Receives routing and link updates and may be used to modify the routing
/// tables (both IPv4 and IPv6), IP addresses, link parameters, neighbor setups,
/// queueing disciplines, traffic classes  and  packet  classifiers  (see
/// rtnetlink(7)).
pub const NETLINK_ROUTE: isize = 0;
/// Unused protocol number.
pub const NETLINK_UNUSED: isize = 1;
/// Reserved for user-mode socket protocols.
pub const NETLINK_USERSOCK: isize = 2;
/// Transport  IPv4  packets  from  netfilter  to  user  space.  Used by
/// ip_queue kernel module.  After a long period of being declared obsolete (in
/// favor of the more advanced nfnetlink_queue feature), it was  removed in
/// Linux 3.5.
pub const NETLINK_FIREWALL: isize = 3;
/// Query information about sockets of various protocol families from the kernel
/// (see sock_diag(7)).
pub const NETLINK_SOCK_DIAG: isize = 4;
/// Netfilter/iptables ULOG.
pub const NETLINK_NFLOG: isize = 5;
/// IPsec.
pub const NETLINK_XFRM: isize = 6;
/// SELinux event notifications.
pub const NETLINK_SELINUX: isize = 7;
/// Open-iSCSI.
pub const NETLINK_ISCSI: isize = 8;
/// Auditing.
pub const NETLINK_AUDIT: isize = 9;
/// Access to FIB lookup from user space.
pub const NETLINK_FIB_LOOKUP: isize = 10;
/// Kernel connector. See `Documentation/connector/*` in the Linux kernel source
/// tree for further information.
pub const NETLINK_CONNECTOR: isize = 11;
/// Netfilter subsystem.
pub const NETLINK_NETFILTER: isize = 12;
/// Transport IPv6 packets from netfilter to user space.  Used by ip6_queue
/// kernel module.
pub const NETLINK_IP6_FW: isize = 13;
/// DECnet routing messages.
pub const NETLINK_DNRTMSG: isize = 14;
/// Kernel messages to user space.
pub const NETLINK_KOBJECT_UEVENT: isize = 15;
///  Generic netlink family for simplified netlink usage.
pub const NETLINK_GENERIC: isize = 16;
/// SCSI transports.
pub const NETLINK_SCSITRANSPORT: isize = 18;
/// eCryptfs.
pub const NETLINK_ECRYPTFS: isize = 19;
/// Infiniband RDMA.
pub const NETLINK_RDMA: isize = 20;
/// Netlink interface to request information about ciphers registered with the
/// kernel crypto API as well as allow configuration of the kernel crypto API.
pub const NETLINK_CRYPTO: isize = 21;

/// Unspecified root traffic control attribute.
pub const TCA_ROOT_UNSPEC: int = 0;
/// Root traffic control table attribute.
pub const TCA_ROOT_TAB: int = 1;
/// Root traffic control flags attribute.
pub const TCA_ROOT_FLAGS: int = 2;
/// Root traffic control count attribute.
pub const TCA_ROOT_COUNT: int = 3;
/// Root traffic control time delta attribute.
pub const TCA_ROOT_TIME_DELTA: int = 4;

/// ELF machine type: no machine.
pub const EM_NONE: u32 = 0;
/// ELF machine type: AT&T WE 32100.
pub const EM_M32: u32 = 1;
/// ELF machine type: SPARC.
pub const EM_SPARC: u32 = 2;
/// ELF machine type: Intel 80386.
pub const EM_386: u32 = 3;
/// ELF machine type: Motorola 68000.
pub const EM_68K: u32 = 4;
/// ELF machine type: Motorola 88000.
pub const EM_88K: u32 = 5;
/// ELF machine type: Intel 80486.
pub const EM_486: u32 = 6;
/// ELF machine type: Intel 80860.
pub const EM_860: u32 = 7;
/// ELF machine type: MIPS I.
pub const EM_MIPS: u32 = 8;
/// ELF machine type: MIPS RS3 little-endian.
pub const EM_MIPS_RS3_LE: u32 = 10;
/// ELF machine type: MIPS RS4 big-endian (alias of
/// [`EM_MIPS_RS3_LE`]).
pub const EM_MIPS_RS4_BE: u32 = 10;
/// ELF machine type: HP PA-RISC.
pub const EM_PARISC: u32 = 15;
/// ELF machine type: SPARC v8+.
pub const EM_SPARC32PLUS: u32 = 18;
/// ELF machine type: PowerPC.
pub const EM_PPC: u32 = 20;
/// ELF machine type: PowerPC 64-bit.
pub const EM_PPC64: u32 = 21;
/// ELF machine type: Cell SPU.
pub const EM_SPU: u32 = 23;
/// ELF machine type: ARM.
pub const EM_ARM: u32 = 40;
/// ELF machine type: Hitachi SH.
pub const EM_SH: u32 = 42;
/// ELF machine type: SPARC v9 64-bit.
pub const EM_SPARCV9: u32 = 43;
/// ELF machine type: Hitachi H8/300.
pub const EM_H8_300: u32 = 46;
/// ELF machine type: Intel IA-64.
pub const EM_IA_64: u32 = 50;
/// ELF machine type: AMD x86-64.
pub const EM_X86_64: u32 = 62;
/// ELF machine type: IBM S/390.
pub const EM_S390: u32 = 22;
/// ELF machine type: Axis CRIS.
pub const EM_CRIS: u32 = 76;
/// ELF machine type: Mitsubishi M32R.
pub const EM_M32R: u32 = 88;
/// ELF machine type: Matsushita MN10300.
pub const EM_MN10300: u32 = 89;
/// ELF machine type: OpenRISC.
pub const EM_OPENRISC: u32 = 92;
/// ELF machine type: Analog Devices Blackfin.
pub const EM_BLACKFIN: u32 = 106;
/// ELF machine type: Altera Nios II.
pub const EM_ALTERA_NIOS2: u32 = 113;
/// ELF machine type: Texas Instruments C6000.
pub const EM_TI_C6000: u32 = 140;
/// ELF machine type: ARM AArch64.
pub const EM_AARCH64: u32 = 183;
/// ELF machine type: Tilera TILEPro.
pub const EM_TILEPRO: u32 = 188;
/// ELF machine type: Xilinx MicroBlaze.
pub const EM_MICROBLAZE: u32 = 189;
/// ELF machine type: Tilera TILE-Gx.
pub const EM_TILEGX: u32 = 191;
/// ELF machine type: Linux BPF.
pub const EM_BPF: u32 = 247;
/// ELF machine type: Fujitsu FR-V.
pub const EM_FRV: u32 = 21569;
/// ELF machine type: Alpha.
pub const EM_ALPHA: u32 = 36902;
/// ELF machine type: Cygnus M32R.
pub const EM_CYGNUS_M32R: u32 = 36929;
/// ELF machine type: legacy IBM S/390.
pub const EM_S390_OLD: u32 = 41872;
/// ELF machine type: Cygnus MN10300.
pub const EM_CYGNUS_MN10300: u32 = 48879;

/// Unused extended ACK attribute.
pub const NLMSGERR_ATTR_UNUSED: int = 0;
/// Human-readable extended ACK error message.
pub const NLMSGERR_ATTR_MSG: int = 1;
/// Offset of the offending field in an extended ACK.
pub const NLMSGERR_ATTR_OFFS: int = 2;
/// Cookie supplied by the caller in an extended ACK.
pub const NLMSGERR_ATTR_COOKIE: int = 3;
/// Highest valid extended ACK attribute number.
pub const NLMSGERR_ATTR_MAX: int = 3;

/// Netlink mmap ring slot is unused.
pub const NL_MMAP_STATUS_UNUSED: int = 0;
/// Netlink mmap ring slot is reserved.
pub const NL_MMAP_STATUS_RESERVED: int = 1;
/// Netlink mmap ring slot contains a valid message.
pub const NL_MMAP_STATUS_VALID: int = 2;
/// Netlink mmap ring slot must be copied.
pub const NL_MMAP_STATUS_COPY: int = 3;
/// Netlink mmap ring slot should be skipped.
pub const NL_MMAP_STATUS_SKIP: int = 4;

/// The netlink socket is not connected.
pub const NETLINK_UNCONNECTED: int = 0;
/// The netlink socket is connected.
pub const NETLINK_CONNECTED: int = 1;

/// Number of bits in a C `long` (kernel ABI constant).
pub const __BITS_PER_LONG: int = 64;
/// Size of the kernel `fd_set` type.
pub const __FD_SETSIZE: int = 1024;
/// Shift used by the kernel for fixed-point load average encoding.
pub const SI_LOAD_SHIFT: int = 16;
/// Maximum size of a socket address.
pub const _K_SS_MAXSIZE: int = 128;
/// Netlink family for the SMC protocol.
pub const NETLINK_SMC: int = 22;
/// Alias of [`NETLINK_SOCK_DIAG`] for socket diagnostics.
pub const NETLINK_INET_DIAG: int = 4;
/// Maximum number of netlink protocol families.
pub const MAX_LINKS: int = 32;

/// First message type available to netlink protocols.
pub const NLMSG_MIN_TYPE: int = 16;
/// Socket option: add a multicast group membership.
pub const NETLINK_ADD_MEMBERSHIP: int = 1;
/// Socket option: drop a multicast group membership.
pub const NETLINK_DROP_MEMBERSHIP: int = 2;
/// Socket option: enable packet information.
pub const NETLINK_PKTINFO: int = 3;
/// Socket option: report broadcast errors.
pub const NETLINK_BROADCAST_ERROR: int = 4;
/// Socket option: suppress `ENOBUFS` errors.
pub const NETLINK_NO_ENOBUFS: int = 5;
/// Socket option: netlink memory-mapped RX ring.
pub const NETLINK_RX_RING: int = 6;
/// Socket option: netlink memory-mapped TX ring.
pub const NETLINK_TX_RING: int = 7;
/// Socket option: receive notifications from all network namespaces.
pub const NETLINK_LISTEN_ALL_NSID: int = 8;
/// Socket option: get multicast group memberships.
pub const NETLINK_LIST_MEMBERSHIPS: int = 9;
/// Socket option: trim payloads from acknowledgment messages.
pub const NETLINK_CAP_ACK: int = 10;
/// Socket option: enable extended acknowledgment attributes.
pub const NETLINK_EXT_ACK: int = 11;
/// Alignment of messages in the netlink mmap ring.
pub const NL_MMAP_MSG_ALIGNMENT: int = 4;
/// Major device number of the network device node.
pub const NET_MAJOR: int = 36;

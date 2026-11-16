// SPDX-License-Identifier: MIT

// Build:
//
// ```
// cd netlink-sys
// cargo build --example audit_events_smol --features rg_socket
// ```
//
// Run *as root*:
//
// ```
// ../target/debug/examples/audit_events_smol
// ```
//
// This example uses `AsyncRtSocket` (backed by `async-io`) with the
// smol runtime. Any runtime that drives the `async-io` reactor can be
// used; `compio`, `glommio` & `monoio` are currently tested. See
// `src/rt.rs` for details.

use std::process;

use netlink_packet_audit::{AuditMessage, StatusMessage};
use netlink_packet_core::{
    NLM_F_ACK, NLM_F_REQUEST, NetlinkBuffer, NetlinkMessage,
};

use netlink_sys::{AsyncSocket, AsyncSocketExt, AsyncRtSocket, SocketAddr};

const NETLINK_AUDIT: isize = libc::NETLINK_AUDIT as isize;
const AUDIT_STATUS_ENABLED: u32 = 1;
const AUDIT_STATUS_PID: u32 = 4;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let kernel_unicast: SocketAddr = SocketAddr::new(0, 0);
        let socket = AsyncRtSocket::new(NETLINK_AUDIT)?;

        let mut status = StatusMessage::new();
        status.enabled = 1;
        status.pid = process::id();
        status.mask = AUDIT_STATUS_ENABLED | AUDIT_STATUS_PID;
        let payload = AuditMessage::SetStatus(status);
        let mut nl_msg = NetlinkMessage::from(payload);
        nl_msg.header.flags = NLM_F_REQUEST | NLM_F_ACK;
        nl_msg.finalize();

        let mut buf = vec![0; 1024 * 8];
        nl_msg.serialize(&mut buf[..nl_msg.buffer_len()]);

        println!(">>> {:?}", nl_msg);
        socket
            .send_to(&buf[..nl_msg.buffer_len()], &kernel_unicast)
            .await?;

        let mut buf = vec![0u8; 1024 * 8];
        loop {
            let (n, _addr) = socket.recv_from(&mut buf).await?;
            // This dance with the NetlinkBuffer should not be
            // necessary. It is here to work around a netlink bug. See:
            // https://github.com/mozilla/libaudit-go/issues/24
            // https://github.com/linux-audit/audit-userspace/issues/78
            {
                let mut nl_buf = NetlinkBuffer::new(&mut buf[..n]);
                if n != nl_buf.length() as usize {
                    nl_buf.set_length(n as u32);
                }
            }
            let parsed =
                NetlinkMessage::<AuditMessage>::deserialize(&buf[..n])?;
            println!("<<< {:?}", parsed);
        }
    })
}

// SPDX-License-Identifier: MIT

/*
 * This example shows the mimimal manual tokio initialization required to be
 * able to use netlink.
 */

use netlink_sys::{AsyncSocket, TokioSocket};

const NETLINK_AUDIT: isize = libc::NETLINK_AUDIT as isize;

fn main() -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    let future = async {
        TokioSocket::new(NETLINK_AUDIT).unwrap();
    };
    rt.handle().block_on(future);
    Ok(())
}

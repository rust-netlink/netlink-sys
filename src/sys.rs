// SPDX-License-Identifier: MIT

#[cfg(target_os = "freebsd")]
mod __private_sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
    include!("ffi/freebsd.rs");
}

#[cfg(not(target_os = "freebsd"))]
mod __private_sys {
    pub use libc::sockaddr_nl;
}

pub use self::__private_sys::*;

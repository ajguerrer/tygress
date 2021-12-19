#![allow(non_camel_case_types, non_snake_case, deref_nullptr)]

use std::io;
use std::os::unix::prelude::RawFd;
use std::time::Duration;

use nix::libc;
use nix::sys::select::{pselect, FdSet};
use nix::sys::time::TimeSpec;

pub(crate) mod owned;
pub(crate) use owned::OwnedFd;

#[cfg(all(feature = "bindgen", not(feature = "overwrite")))]
include!(concat!(env!("OUT_DIR"), "/sys.rs"));

#[cfg(any(
    not(feature = "bindgen"),
    all(feature = "bindgen", feature = "overwrite")
))]
include!("sys.rs");

pub fn ifreq_name(name: &str) -> [libc::c_char; libc::IF_NAMESIZE] {
    let mut ifreq_name = [b'\0' as i8; libc::IF_NAMESIZE];
    for (i, b) in name
        .as_bytes()
        .iter()
        // last byte must be '\0'
        .take(libc::IF_NAMESIZE - 1)
        .enumerate()
    {
        ifreq_name[i] = *b as libc::c_char;
    }
    ifreq_name
}
// pub const TUNSETIFF: libc::c_ulong = 0x400454CA;

nix::ioctl_write_ptr_bad!(
    ioctl_tunsetiff,
    nix::request_code_write!(b'T', 202, std::mem::size_of::<libc::c_int>()),
    ifreq
);

nix::ioctl_read_bad!(ioctl_siocgifmtu, libc::SIOCGIFMTU, ifreq);
nix::ioctl_read_bad!(ioctl_siocgifindex, SIOCGIFINDEX, ifreq);

pub fn poll(fd: RawFd, timeout: Option<Duration>) -> io::Result<bool> {
    let mut readfds = FdSet::new();
    readfds.insert(fd);
    let timeout = timeout.map(TimeSpec::from);
    pselect(None, &mut readfds, None, None, &timeout, None)?;

    Ok(readfds.contains(fd))
}

#![allow(unsafe_code)]

use std::fs::OpenOptions;
use std::io;
use std::time::Duration;

use super::{sys, Event};
use super::{HardwareType, NetDev};
use rustix::fd::OwnedFd;
use rustix::fs::{fcntl_setfl, OFlags};
use rustix::io::{read, write};
use rustix::net::{socket, AddressFamily, Protocol, SocketType};

/// A virtual TUN (IP) or TAP (Ethernet) interface. [Read more][tuntap]
///
///  [tuntap]: https://www.kernel.org/doc/html/latest/networking/tuntap.html
#[derive(Debug)]
pub struct TunTapInterface {
    fd: OwnedFd,
    mtu: usize,
    hw_type: HardwareType,
}

impl TunTapInterface {
    /// Create a device bound to a TUN/TAP interface called `name`.
    ///
    /// Depending on the ownership privileges of the interface, superuser privileges or
    /// `CAP_NET_ADMIN` capabilities may be required.
    pub fn bind(name: &str, hw_type: HardwareType) -> io::Result<Self> {
        let fd = OwnedFd::from(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/net/tun")?,
        );
        fcntl_setfl(&fd, OFlags::NONBLOCK)?;

        let ifreq_name = sys::ifreq_name(name);
        sys::ioctl_tunsetiff(&fd, hw_type, ifreq_name)?;

        let socket = socket(AddressFamily::INET, SocketType::DGRAM, Protocol::default())?;
        let mtu = sys::ioctl_siocgifmtu(&socket, ifreq_name)?;

        Ok(TunTapInterface { fd, mtu, hw_type })
    }
}

impl NetDev for TunTapInterface {
    type Error = io::Error;

    #[inline]
    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        write(&self.fd, buf).map_err(io::Error::from)
    }

    #[inline]
    fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        read(&self.fd, buf).map_err(io::Error::from)
    }

    #[inline]
    fn mtu(&self) -> usize {
        self.mtu
    }

    #[inline]
    fn poll(&self, interest: Event, timeout: Option<Duration>) -> io::Result<Event> {
        sys::poll(&self.fd, interest, timeout)
    }

    #[inline]
    fn hw_type(&self) -> HardwareType {
        self.hw_type
    }
}

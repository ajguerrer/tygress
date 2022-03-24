use std::io;
use std::os::unix::prelude::{AsRawFd, FromRawFd};
use std::time::Duration;

use nix::fcntl::{open, OFlag};
use nix::libc;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType};
use nix::sys::stat::Mode;
use nix::unistd::{read, write};

use super::{sys, Event};
use super::{Layer, NetDev};

/// A virtual TUN (IP) or TAP (Ethernet) interface. [Read more][tuntap]
///
///  [tuntap]: https://www.kernel.org/doc/html/latest/networking/tuntap.html
#[derive(Debug)]
pub struct TunTapInterface {
    fd: sys::OwnedFd,
    ifreq_name: [libc::c_char; libc::IF_NAMESIZE],
    layer: Layer,
}

impl TunTapInterface {
    /// Create a device bound to a TUN/TAP interface called `name`.
    ///
    /// Depending on the ownership privileges of the interface, superuser privileges or
    /// `CAP_NET_ADMIN` capabilities may be required.
    pub fn bind(name: &str, layer: Layer) -> io::Result<Self> {
        let fd = open(
            "/dev/net/tun",
            OFlag::O_RDWR | OFlag::O_NONBLOCK,
            Mode::empty(),
        )?;
        // SAFETY: nix open returns a valid open fd or an error.
        let fd = unsafe { sys::OwnedFd::from_raw_fd(fd) };

        let ifrn_name = sys::ifreq_name(name);
        let ifru_flags = match layer {
            Layer::Ip => libc::IFF_TUN as libc::c_short,
            Layer::Ethernet => libc::IFF_TAP as libc::c_short,
        } | libc::IFF_NO_PI as libc::c_short;
        let ifreq = sys::ifreq {
            ifr_ifrn: sys::ifreq__bindgen_ty_1 { ifrn_name },
            ifr_ifru: sys::ifreq__bindgen_ty_2 { ifru_flags },
        };

        unsafe { sys::ioctl_tunsetiff(fd.as_raw_fd(), &ifreq)? };

        Ok(TunTapInterface {
            fd,
            ifreq_name: ifrn_name,
            layer,
        })
    }
}

impl NetDev for TunTapInterface {
    type Error = io::Error;

    #[inline]
    fn send(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.fd.as_raw_fd(), buf).map_err(io::Error::from)
    }

    #[inline]
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd.as_raw_fd(), buf).map_err(io::Error::from)
    }

    fn mtu(&self) -> io::Result<usize> {
        let fd = socket::socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )?;
        // SAFETY: nix socket returns a valid open fd or an error.
        let fd = unsafe { sys::OwnedFd::from_raw_fd(fd) };

        let ifrn_name = self.ifreq_name;
        let mut ifreq = sys::ifreq {
            ifr_ifrn: sys::ifreq__bindgen_ty_1 { ifrn_name },
            ifr_ifru: sys::ifreq__bindgen_ty_2 { ifru_mtu: 0 },
        };

        unsafe {
            sys::ioctl_siocgifmtu(fd.as_raw_fd(), &mut ifreq)?;
            Ok(ifreq.ifr_ifru.ifru_mtu as usize)
        }
    }

    #[inline]
    fn poll(&self, timeout: Option<Duration>) -> io::Result<Event> {
        sys::poll(self.fd.as_raw_fd(), timeout)
    }

    #[inline]
    fn layer(&self) -> Layer {
        self.layer
    }
}

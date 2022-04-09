#![allow(unsafe_code)]

use std::io;
use std::os::unix::prelude::{AsRawFd, FromRawFd};
use std::time::Duration;

use nix::libc;
use nix::sys::socket::{self, AddressFamily, LinkAddr, MsgFlags, SockAddr, SockFlag, SockType};

use super::{sys, Event};
use super::{NetDev, Topology};

/// A socket of the AF_PACKET family. [Read more][packet]
///
/// [packet]: https://man7.org/linux/man-pages/man7/packet.7.html
#[derive(Debug)]
pub struct PacketSocket {
    fd: sys::OwnedFd,
    ifreq_name: [libc::c_char; libc::IF_NAMESIZE],
    layer: Topology,
}

impl PacketSocket {
    /// Creates a socket with family `AF_PACKET` and binds it to the interface called `name`.
    ///
    /// Requires superuser privileges or the `CAP_NET_RAW` capability.
    pub fn bind(name: &str, layer: Topology) -> io::Result<Self> {
        let ty = match layer {
            Topology::Ip => SockType::Datagram,
            Topology::EthernetII => SockType::Raw,
        };

        let fd = socket::socket(AddressFamily::Packet, ty, SockFlag::SOCK_NONBLOCK, None)?;
        // SAFETY: nix socket either produces a valid open socket or an error
        let fd = unsafe { sys::OwnedFd::from_raw_fd(fd) };

        let ifrn_name = sys::ifreq_name(name);
        let mut ifreq = sys::ifreq {
            ifr_ifrn: sys::ifreq__bindgen_ty_1 { ifrn_name },
            ifr_ifru: sys::ifreq__bindgen_ty_2 { ifru_ivalue: 0 },
        };
        let index = unsafe {
            sys::ioctl_siocgifindex(fd.as_raw_fd(), &mut ifreq)?;
            ifreq.ifr_ifru.ifru_ivalue
        };
        let linkaddr = LinkAddr(libc::sockaddr_ll {
            sll_family: libc::AF_PACKET as libc::c_ushort,
            // Equivalent to htons(libc::ETH_P_ALL).
            // To keep your sanity, make sure the integer width is 16 bits first!
            sll_protocol: (libc::ETH_P_ARP as libc::c_ushort).to_be(),
            sll_ifindex: index,
            sll_hatype: 1,
            sll_pkttype: 0,
            sll_halen: 6,
            sll_addr: [0; 8],
        });
        socket::bind(fd.as_raw_fd(), &SockAddr::Link(linkaddr))?;

        Ok(PacketSocket {
            fd,
            ifreq_name: ifrn_name,
            layer,
        })
    }
}

impl NetDev for PacketSocket {
    type Error = io::Error;
    #[inline]
    fn send(&mut self, buf: &[u8]) -> io::Result<usize> {
        socket::send(self.fd.as_raw_fd(), buf, MsgFlags::empty()).map_err(io::Error::from)
    }

    #[inline]
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        socket::recv(self.fd.as_raw_fd(), buf, MsgFlags::empty()).map_err(io::Error::from)
    }

    fn mtu(&self) -> io::Result<usize> {
        let ifrn_name = self.ifreq_name;
        let mut ifreq = sys::ifreq {
            ifr_ifrn: sys::ifreq__bindgen_ty_1 { ifrn_name },
            ifr_ifru: sys::ifreq__bindgen_ty_2 { ifru_mtu: 0 },
        };

        unsafe {
            sys::ioctl_siocgifmtu(self.fd.as_raw_fd(), &mut ifreq)?;
            Ok(ifreq.ifr_ifru.ifru_mtu as usize)
        }
    }

    #[inline]
    fn poll(&self, timeout: Option<Duration>) -> io::Result<Event> {
        sys::poll(self.fd.as_raw_fd(), timeout)
    }

    #[inline]
    fn topology(&self) -> Topology {
        self.layer
    }
}

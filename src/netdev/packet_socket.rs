#![allow(unsafe_code)]

use std::io;
use std::time::Duration;

use rustix::io::OwnedFd;
use rustix::net::{
    recv, send, socket_with, AddressFamily, Protocol, RecvFlags, SendFlags, SocketFlags, SocketType,
};

use super::{sys, Event};
use super::{NetDev, Topology};

/// A socket of the AF_PACKET family. [Read more][packet]
///
/// [packet]: https://man7.org/linux/man-pages/man7/packet.7.html
#[derive(Debug)]
pub struct PacketSocket {
    fd: OwnedFd,
    mtu: usize,
    topology: Topology,
}

impl PacketSocket {
    /// Creates a socket with family `AF_PACKET` and binds it to the interface called `name`.
    ///
    /// Requires superuser privileges or the `CAP_NET_RAW` capability.
    pub fn bind(name: &str, topology: Topology) -> io::Result<Self> {
        let type_ = match topology {
            Topology::Ip => SocketType::DGRAM,
            Topology::EthernetII => SocketType::RAW,
        };
        let fd = socket_with(
            AddressFamily::PACKET,
            type_,
            SocketFlags::NONBLOCK,
            Protocol::from_raw(0),
        )?;

        let ifreq_name = sys::ifreq_name(name);
        sys::bind_interface(&fd, ifreq_name)?;

        let mtu = sys::ioctl_siocgifmtu(&fd, ifreq_name)?;

        Ok(PacketSocket { fd, mtu, topology })
    }
}

impl NetDev for PacketSocket {
    type Error = io::Error;
    #[inline]
    fn send(&self, buf: &[u8]) -> io::Result<usize> {
        send(&self.fd, buf, SendFlags::empty()).map_err(io::Error::from)
    }

    #[inline]
    fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        recv(&self.fd, buf, RecvFlags::empty()).map_err(io::Error::from)
    }

    #[inline]
    fn poll(&self, interest: Event, timeout: Option<Duration>) -> io::Result<Event> {
        sys::poll(&self.fd, interest, timeout)
    }

    #[inline]
    fn mtu(&self) -> usize {
        self.mtu
    }

    #[inline]
    fn topology(&self) -> Topology {
        self.topology
    }
}

#![allow(unsafe_code)]

//! An interface to physical networking hardware.
//!
//! This module contains the [`NetDev`] trait. [`NetDev`] can be configured to work at the Data Link
//! layer or Network layer. Any hardware abstraction that can send and receive either EthernetII
//! frames or IP packets can be an implemented as a [`NetDev`].
//!
//! Once a [`NetDev`] is fed to the `tygress` executor, the executor multiplexes data between the
//! [`NetDev`] and any open sockets.
//!
//! If the `netdev` feature is enabled, some OS-specific [`NetDev`]s for Unix are provided,
//! including:
//!
//! - [`TunTapInterface`] - A [`NetDev`] for the [TUN/TAP][tuntap] device driver.
//! - [`PacketSocket`] - A [`NetDev`] for the [packet] socket family.
//!
//! Feel free to use these [`NetDev`]s as references for your own implementations.
//!
//! [tuntap]: https://www.kernel.org/doc/html/latest/networking/tuntap.html
//! [packet]: https://man7.org/linux/man-pages/man7/packet.7.html

#[cfg(all(feature = "netdev", unix))]
mod packet_socket;
#[cfg(all(feature = "netdev", any(target_os = "linux", target_os = "android")))]
mod tuntap_interface;

#[cfg(all(feature = "netdev", unix))]
pub use packet_socket::PacketSocket;
#[cfg(all(feature = "netdev", any(target_os = "linux", target_os = "android")))]
pub use tuntap_interface::TunTapInterface;
#[cfg(all(feature = "netdev", unix))]
mod sys;

use core::time::Duration;

/// Interface for network hardware capable of sending and receiving either raw IP packets or
/// Ethernet frames depending on which [`Layer`] the device operates.
pub trait NetDev {
    type Error;
    /// Sends a single raw network frame contained in `buf`. `buf` may not be larger than the
    /// devices [`mtu`][NetDev::mtu] plus 14 byte Ethernet header if the device operates on
    /// [`Layer::Ethernet`].
    fn send(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;
    /// Receives a single raw network frame and places it in `buf`. `buf` must be large enough to
    /// hold the devices [`mtu`][NetDev::mtu] plus 14 byte Ethernet header if the device operates on
    /// [`Layer::Ethernet`].
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
    /// Maximum transmission unit.
    ///
    /// Indicates the maximum number of bytes that can be transmitted in an IP packet.
    ///
    /// Note: To stay consistent with the standard, `mtu` *does not* factor in the Ethernet header
    /// for devices that operate on [`Layer::Ethernet`]. Those devices should add 14 bytes of extra
    /// space for the Ethernet header (without a 802.1Q tag).
    fn mtu(&self) -> Result<usize, Self::Error>;
    /// Checks readiness of incoming data without blocking for use in a async executor.
    fn poll(&self, timeout: Option<Duration>) -> Result<bool, Self::Error>;
}

/// Indicates the layer that a [`NetDev`] operates on.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Layer {
    /// Sends and receives IP packets without a Ethernet header.  
    Ip,
    /// Sends and receives Ethernet frames (Ip packets with Ethernet header).
    Ethernet,
}

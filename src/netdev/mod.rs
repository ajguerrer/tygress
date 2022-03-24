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

use core::fmt;
use core::ops;
use core::time::Duration;

/// Interface for network hardware capable of sending and receiving either raw IP packets or
/// Ethernet frames depending on which [`Layer`] the device operates.
pub trait NetDev {
    type Error;
    /// Sends a single raw network frame contained in `buf`. `buf` may not be larger than the
    /// devices [`mtu`][NetDev] plus 14 byte [`EthernetII`][crate::header::EthernetII]  header if
    /// the device operates on [`Layer::Ethernet`].
    fn send(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;
    /// Receives a single raw network frame and places it in `buf`. `buf` must be large enough to
    /// hold the devices [`mtu`][NetDev] plus 14 byte [`EthernetII`][crate::header::EthernetII]
    /// header if the device operates on [`Layer::Ethernet`].
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
    /// Checks io readiness so that calls to [`send`][NetDev] or [`recv`][NetDev] are guaranteed
    /// not to block. Called in the event loop of an async executor.
    fn poll(&self, timeout: Option<Duration>) -> Result<Event, Self::Error>;
    /// Maximum transmission unit.
    ///
    /// Indicates the maximum number of bytes that can be transmitted in an IP packet.
    ///
    /// # Note
    ///
    /// To stay consistent with the IETF standard, `mtu` *does not* factor in the 14 byte
    /// [`EthernetII`][crate::header::EthernetII]  header. [`send`][NetDev] and [`recv`][NetDev]
    /// should account for these extra bytes by increasing the buf size accordingly.
    fn mtu(&self) -> Result<usize, Self::Error>;
    /// Returns [`Layer`] device operates on. Devices operating on [`Layer::Ethernet`] include an
    /// additional [`EthernetII`][crate::header::EthernetII] header.
    fn layer(&self) -> Layer;
}

/// Indicates the layer that a [`NetDev`] operates on. Devices operating on [`Layer::Ethernet`]
/// include and additional [`EthernetII`][crate::header::EthernetII] header.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Layer {
    /// Sends and receives IP packets without a Ethernet header.  
    Ip,
    /// Sends and receives Ethernet frames (Ip packets with Ethernet header).
    Ethernet,
}

#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Event(u8);

// bits must be one-hot
const READABLE: u8 = 0b01;
const WRITABLE: u8 = 0b10;

impl Event {
    /// [`Event`] with read readiness only. to read from a [`NetDev`].
    pub const READABLE: Event = Event(READABLE);
    /// [`Event`] with write readiness only. to write from a [`NetDev`].
    pub const WRITABLE: Event = Event(WRITABLE);

    /// Constructs an [`Event`] without any readiness.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns `true` if a [`NetDev`] is ready to [`read`][NetDev].
    #[inline]
    pub const fn is_readable(&self) -> bool {
        (self.0 & READABLE) != 0
    }

    /// Returns `true` if a [`NetDev`] is ready to [`write`][NetDev].
    #[inline]
    pub const fn is_writable(&self) -> bool {
        (self.0 & WRITABLE) != 0
    }
}

impl ops::BitOr for Event {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl ops::BitOrAssign for Event {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 = (*self | other).0;
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("readable", &self.is_readable())
            .field("writable", &self.is_writable())
            .finish()
    }
}

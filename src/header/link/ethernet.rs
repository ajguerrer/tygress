//! [`EthernetII`] header
//!
//! [`EthernetII`] header with source and destination [`EtherAddr`]s and a [`EtherType`].
use core::fmt;

use crate::header::error::HeaderTruncated;
use crate::header::primitive::{non_exhaustive_enum, U16};
use crate::header::utils::as_header;

// check len
// check destination
// check fields valid

/// An EthernetII frame header. [Read more][RFC 1042]
///
/// Contains 48-bit source and destination MAC addresses with an EtherType indicating the protocol
/// contained in the payload of the frame. The optional 802.1Q tag is not included.
///
/// [RFC 1042]: https://tools.ietf.org/html/rfc1042
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub struct EthernetII {
    dst: EtherAddr,
    src: EtherAddr,
    ty: EtherTypeRepr,
}

impl EthernetII {
    /// Returns an immutable view of `bytes` as an EthernetII header followed by a payload or an
    /// error if the size or contents do not represent a valid EthernetII header.
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8]), HeaderTruncated> {
        as_header!(EthernetII, bytes)
    }

    /// Returns the source Ethernet address.
    #[inline]
    pub const fn src(&self) -> EtherAddr {
        self.src
    }

    /// Returns destination Ethernet address.
    #[inline]
    pub const fn dst(&self) -> EtherAddr {
        self.dst
    }

    /// Returns the EtherType of frame.
    #[inline]
    pub const fn ethertype(&self) -> EtherType {
        self.ty.get()
    }
}

impl fmt::Display for EthernetII {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EthernetII ({}) {} → {}",
            self.ethertype(),
            self.src(),
            self.dst(),
        )
    }
}

/// A 48-bit Ethernet address. [Read more][RFC 7042]
///
/// Commonly known as an Ethernet interface identifier or MAC address.
///
/// [RFC 7042]: https://tools.ietf.org/html/rfc7042#section-2
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(transparent)]
pub struct EtherAddr([u8; 6]);

impl EtherAddr {
    /// The broadcast EtherAddr. All nodes listen to frames sent to this address.
    pub const BROADCAST: EtherAddr = EtherAddr([0xFF; 6]);

    /// Create an EtherAddr from six network endian octets.
    #[inline]
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Convert EtherAddr to a sequence of octets. Bytes are network endian.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if EtherAddr is a individual unicast address.
    #[inline]
    pub const fn is_unicast(&self) -> bool {
        self.0[0] & 0x01 == 0
    }

    /// Returns `true` if EtherAddr is a group multicast address.
    #[inline]
    pub const fn is_multicast(&self) -> bool {
        !self.is_unicast()
    }

    /// Returns `true` if EtherAddr is the 'broadcast' address.
    #[inline]
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 0xFF
            && self.0[1] == 0xFF
            && self.0[2] == 0xFF
            && self.0[3] == 0xFF
            && self.0[4] == 0xFF
            && self.0[5] == 0xFF
    }

    /// Returns `true` if EtherAddr is universally administered.
    #[inline]
    pub const fn is_universal(&self) -> bool {
        self.0[0] & 0x02 == 0
    }

    /// Returns `true` if EtherAddr is locally administered.
    #[inline]
    pub const fn is_local(&self) -> bool {
        !self.is_universal()
    }
}

impl fmt::Display for EtherAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        )
    }
}

non_exhaustive_enum! {
/// A list of Ethernet protocol parameters. [Read more][IANA]
///
/// The protocol used in the payload of an Ethernet frame. A complete list is maintained by [IANA].
///
/// [IANA]: https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml
pub enum EtherType(u16) {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    Ipv6 = 0x86DD,
}
}

/// Representation of [`EtherType`] cast from a slice of bytes instead of constructed. It is assumed
/// that [`verify`][EtherTypeRepr::verify] is called directly after casting before any other
/// methods are called.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub(crate) struct EtherTypeRepr(U16);

impl EtherTypeRepr {
    /// Get the underlying [`EtherType`].
    #[inline]
    pub(crate) const fn get(&self) -> EtherType {
        EtherType::new(self.0.get())
    }
}

impl From<EtherType> for EtherTypeRepr {
    #[inline]
    fn from(value: EtherType) -> Self {
        EtherTypeRepr(U16::new(value.get()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 13];
        assert_eq!(EthernetII::from_bytes(&bytes).unwrap_err(), HeaderTruncated);
    }

    #[test]
    fn invalid_ethertype() {
        let bytes = [0; 14];
        let (header, _) = EthernetII::from_bytes(&bytes).unwrap();
        assert_eq!(header.ethertype(), EtherType::Unknown(0));
    }

    #[test]
    fn valid_ethertypes() {
        // ipv4
        let bytes = [&[0; 12][..], &[0x08, 0x00][..]].concat();
        let (header, _) = EthernetII::from_bytes(&bytes).unwrap();
        assert_eq!(header.ethertype(), EtherType::Ipv4);
        // arp
        let bytes = [&[0; 12][..], &[0x08, 0x06][..]].concat();
        let (header, _) = EthernetII::from_bytes(&bytes).unwrap();
        assert_eq!(header.ethertype(), EtherType::Arp);
        // ipv6
        let bytes = [&[0; 12][..], &[0x86, 0xDD][..]].concat();
        let (header, _) = EthernetII::from_bytes(&bytes).unwrap();
        assert_eq!(header.ethertype(), EtherType::Ipv6);
    }

    #[test]
    fn ether_addr() {
        let mut addr = EtherAddr([0xFF; 6]);
        assert!(addr.is_broadcast());
        assert_eq!((true, false), (addr.is_local(), addr.is_universal()));
        assert_eq!((true, false), (addr.is_multicast(), addr.is_unicast()));

        addr.0[0] = 0x0EF;
        assert!(!addr.is_broadcast());
        assert_eq!((true, false), (addr.is_local(), addr.is_universal()));
        assert_eq!((true, false), (addr.is_multicast(), addr.is_unicast()));

        addr.0[0] = 0x0FE;
        assert!(!addr.is_broadcast());
        assert_eq!((true, false), (addr.is_local(), addr.is_universal()));
        assert_eq!((false, true), (addr.is_multicast(), addr.is_unicast()));

        addr.0[0] = 0x0FD;
        assert!(!addr.is_broadcast());
        assert_eq!((false, true), (addr.is_local(), addr.is_universal()));
        assert_eq!((true, false), (addr.is_multicast(), addr.is_unicast()));

        addr.0[0] = 0xFC;
        assert!(!addr.is_broadcast());
        assert_eq!((false, true), (addr.is_local(), addr.is_universal()));
        assert_eq!((false, true), (addr.is_multicast(), addr.is_unicast()));
    }
}

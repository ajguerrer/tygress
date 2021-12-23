use core::fmt::{self, Debug};

use super::{as_header, as_header_mut};
use crate::error::{Error, Result};

/// An EthernetII frame header. [Read more][RFC 1042]
///
/// Contains six-octet source and destination MAC addresses with an EtherType indicating the
/// protocol contained in the payload of the frame. The optional 802.1Q tag is not included.
///
/// [RFC 1042]: https://tools.ietf.org/html/rfc1042
#[derive(Debug, Clone)]
#[repr(C)]
pub struct EthernetII {
    pub destination: EtherAddr,
    pub source: EtherAddr,
    pub ethertype: EtherTypeRepr,
}

impl EthernetII {
    /// Parse bytes as an immutable view of an Ethernet header followed by a payload. Returns an
    /// error if the size or contents do not represent a valid EthernetII header.
    #[inline]
    pub fn split_header(bytes: &[u8]) -> Result<(&Self, &[u8])> {
        let (header, payload) = as_header!(EthernetII, bytes)?;
        EtherTypeRepr::parse(header.ethertype.0)?;
        Ok((header, payload))
    }

    /// Parse bytes as an mutable view of an Ethernet header followed by a payload. Returns an error
    /// if the size or contents do not represent a valid EthernetII header.
    #[inline]
    pub fn split_header_mut(bytes: &mut [u8]) -> Result<(&mut Self, &mut [u8])> {
        let (header, payload) = as_header_mut!(EthernetII, bytes)?;
        EtherTypeRepr::parse(header.ethertype.0)?;
        Ok((header, payload))
    }
}

impl fmt::Display for EthernetII {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EthernetII src: {}, dst: {}, type: {}",
            self.source, self.destination, self.ethertype
        )
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
/// A six-octet Ethernet address.
///
/// Also known as a physical address or a MAC address.
pub struct EtherAddr([u8; 6]);

impl EtherAddr {
    /// The broadcast EtherAddr. All network devices listen to frames sent to this address.
    pub const BROADCAST: EtherAddr = EtherAddr([0xFF; 6]);

    /// Create an EtherAddr from six network endian octets.
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Convert EtherAddr to a sequence of octets. Bytes are network endian.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// EtherAddr is a individual unicast address.
    #[inline]
    pub const fn is_unicast(&self) -> bool {
        self.0[0] & 0x01 == 0
    }

    /// EtherAddr is a group multicast address.
    #[inline]
    pub const fn is_multicast(&self) -> bool {
        !self.is_unicast()
    }

    /// EtherAddr is the broadcast address.
    #[inline]
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 0xFF
            && self.0[1] == 0xFF
            && self.0[2] == 0xFF
            && self.0[3] == 0xFF
            && self.0[4] == 0xFF
            && self.0[5] == 0xFF
    }

    /// EtherAddr is universally administered.
    #[inline]
    pub const fn is_universal(&self) -> bool {
        self.0[0] & 0x02 == 0
    }

    /// EtherAddr is locally administered.
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

/// The protocol used for the payload of an Ethernet frame.
///
/// Value is network endian.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u16)]
pub enum EtherType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    Ipv6 = 0x86DD,
}

impl From<EtherType> for u16 {
    fn from(val: EtherType) -> Self {
        val as u16
    }
}

/// A two-octet EtherType field.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub struct EtherTypeRepr([u8; 2]);

impl EtherTypeRepr {
    const IPV4: EtherTypeRepr = EtherTypeRepr(u16::to_be_bytes(EtherType::Ipv4 as u16));
    const ARP: EtherTypeRepr = EtherTypeRepr(u16::to_be_bytes(EtherType::Arp as u16));
    const IPV6: EtherTypeRepr = EtherTypeRepr(u16::to_be_bytes(EtherType::Ipv6 as u16));

    /// Parse bytes as a representation of an [`EtherType`]. Returns an error if the bytes do not
    /// match a known EtherType.
    #[inline]
    pub const fn parse(bytes: [u8; 2]) -> Result<Self> {
        match EtherTypeRepr(bytes) {
            Self::IPV4 | Self::ARP | Self::IPV6 => Ok(Self(bytes)),
            _ => Err(Error::Unknown),
        }
    }

    /// Get the underlying [`EtherType`].
    #[inline]
    pub const fn get(&self) -> EtherType {
        match *self {
            Self::IPV4 => EtherType::Ipv4,
            Self::ARP => EtherType::Arp,
            Self::IPV6 => EtherType::Ipv6,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for EtherTypeRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::IPV4 => write!(f, "IPv4"),
            Self::ARP => write!(f, "ARP"),
            Self::IPV6 => write!(f, "IPv6"),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::*;

    #[test]
    fn short_header() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        assert_eq!(EthernetII::split_header(&bytes).unwrap_err(), Error::Length);
    }

    #[test]
    fn invalid_ethertype() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        assert_eq!(
            EthernetII::split_header(&bytes).unwrap_err(),
            Error::Unknown
        );
    }

    #[test]
    fn valid_ethertypes() {
        // ipv4
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x08, 0x00];
        let (header, _) = EthernetII::split_header(&bytes).unwrap();
        assert_eq!(header.ethertype.get(), EtherType::Ipv4);
        // arp
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x08, 0x06];
        let (header, _) = EthernetII::split_header(&bytes).unwrap();
        assert_eq!(header.ethertype.get(), EtherType::Arp);
        // ipv6
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x86, 0xDD];
        let (header, _) = EthernetII::split_header(&bytes).unwrap();
        assert_eq!(header.ethertype.get(), EtherType::Ipv6);
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

//! [`Ipv4`] header
//!
//! [`Ipv4`] header supporting 32-bit addressing (see [`Ipv4Addr`]) and fragmentation.
use core::fmt;

use crate::header::error::{Error, Result};
use crate::header::macros::{as_header, verify_checksum};
use crate::header::primitive::{U16, U8};

use super::ip::{Dscp, Ecn, Protocol, ProtocolRepr, Version};

/// An IPv4 header. [Read more][RFC 791]
///
/// IPv4 features 32-bit addressing between uniquely addressed nodes on a network and fragmentation
/// of data into multiple packets.
///
/// Though the the IP layer supports packet fragmentation, it is considered fragile according to
/// [RFC 8900]. Instead, alternatives to work around fragmentation, such as TCP segmentation and MTU
/// discovery, are delegated to the transport and application layer.
///
/// [RFC 791]: https://tools.ietf.org/html/rfc791#section-3
/// [RFC 8900]: https://tools.ietf.org/html/rfc8900
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub struct Ipv4 {
    ver_ihl: VerIhl,
    diff_serv: DiffServ,
    tlen: U16,
    id: U16,
    flgs_ofst: FlagsFragOffset,
    ttl: U8,
    proto: ProtocolRepr,
    cks: U16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
}

impl Ipv4 {
    /// Returns an immutable view if `bytes` an an IPv4 header followed by a payload or an error if
    /// the size or contents do not represent a valid IPv4 header. Since IPv4 options are dynamic in
    /// length, they are not included in the header and are instead returned as part of the payload.
    #[inline]
    pub fn split_header(bytes: &[u8]) -> Result<(&Self, &[u8], &[u8])> {
        let (header, options_payload) = as_header!(Ipv4, bytes)?;

        if header.ver_ihl.version() != 4 {
            return Err(Error::Unsupported);
        }

        // options_payload starts with some amount of Ipv4 options if any. Check their length.
        if options_payload.len() < usize::from(header.options_len()) {
            return Err(Error::Truncated);
        }

        if header.flgs_ofst.flags() > 3 {
            return Err(Error::Malformed);
        }

        header.proto.check()?;

        verify_checksum(&bytes[..header.header_len().into()])?;

        let (options, payload) = options_payload.split_at(header.options_len().into());
        Ok((header, options, payload))
    }

    /// Always returns [`Version::Ipv4`].
    #[inline]
    pub fn version(&self) -> Version {
        Version::Ipv4
    }

    /// Returns the length of the IPv4 header in bytes. At a minimum, the length of a header with no
    /// options is 20 bytes. A header with the maximum amount of options/padding has a length of 60
    /// bytes.
    #[inline]
    pub fn header_len(&self) -> u8 {
        self.ver_ihl.header_len() * 4
    }

    /// Returns the length of the IPv4 options in bytes. Options are optional. A header may have up
    /// to 40 bytes of options.
    #[inline]
    pub fn options_len(&self) -> u8 {
        // Options start after 20 bytes of header
        (self.ver_ihl.header_len() * 4).saturating_sub(20)
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn dscp(&self) -> Dscp {
        Dscp(self.diff_serv.dscp())
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn ecn(&self) -> Ecn {
        Ecn::try_from(self.diff_serv.ecn()).unwrap()
    }

    /// Returns the total length of the assembled IPv4 packet. Does not include link layer header.
    #[inline]
    pub fn total_len(&self) -> u16 {
        u16::from(self.tlen)
    }

    /// Returns fragmentation ID of the IPv4 packet.
    #[inline]
    pub fn id(&self) -> u16 {
        u16::from(self.id)
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn flags(&self) -> Flags {
        Flags::try_from(self.flgs_ofst.flags()).unwrap()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn frag_offset(&self) -> u16 {
        self.flgs_ofst.frag_offset()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn ttl(&self) -> u8 {
        u8::from(self.ttl)
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn protocol(&self) -> Protocol {
        self.proto.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from(self.cks)
    }

    /// Returns source IPv4 address.
    #[inline]
    pub fn source(&self) -> Ipv4Addr {
        self.src
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub fn destination(&self) -> Ipv4Addr {
        self.dst
    }
}

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IPv4 proto: {}, src: {}, dst: {}, len: {}, ttl: {}, flags: {}, dscp: {}, ecn: {}, id: {}, offset: {}",
            self.protocol(), self.source(), self.destination(), self.total_len(), self.ttl(), self.flags(), self.dscp(), self.ecn(), self.id(), self.frag_offset()
        )
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(transparent)]
/// A 32-bit IPv4 address.
///
/// Many addresses or address blocks cary a special meaning defined by the [IANA].
///
/// [IANA]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
pub struct Ipv4Addr([u8; 4]);

impl Ipv4Addr {
    /// The 'localhost' IPv4 address pointing to `127.0.0.1`.
    pub const LOCALHOST: Ipv4Addr = Ipv4Addr([127, 0, 0, 1]);

    /// The 'unspecified' IPv4 address, also known as the 'any' address, pointing to `0.0.0.0`.
    pub const UNSPECIFIED: Ipv4Addr = Ipv4Addr([0; 4]);

    /// The 'broadcast' IPv4 address pointing to `255.255.255.255`.
    pub const BROADCAST: Ipv4Addr = Ipv4Addr([255; 4]);

    /// Create an Ipv4Addr from four network endian octets.
    #[inline]
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    /// Convert Ipv4Addr to a sequence of octets. Bytes are network endian.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Returns `true` if address is the 'unspecified', also known as the 'any' address.
    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Returns `true` if address belongs to the 'loopback' block `127.0.0.0/8`.
    #[inline]
    pub const fn is_loopback(&self) -> bool {
        self.0[0] == 127
    }

    /// Returns `true` if address belongs to the 'private' blocks:
    /// - `10.0.0.0/8`
    /// - `172.16.0.0/12`
    /// - `192.168.0.0/16`
    ///
    /// Defined in [IETF RFC 1918]
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    #[inline]
    pub const fn is_private(&self) -> bool {
        self.0[0] == 10
            || (self.0[0] == 172 && (self.0[1] & 0b1111_0000 == 16))
            || (self.0[0] == 192 && self.0[1] == 168)
    }

    /// Returns `true` if address belongs to the 'link-local' block `169.254.0.0/16`.
    #[inline]
    pub const fn is_link_local(&self) -> bool {
        self.0[0] == 169 && self.0[1] == 254
    }

    /// Returns `true` if the address is globally routable. [Read more][IANA]
    ///
    /// Several address blocks return `false` including:
    ///
    /// - 'private' block (see [`Ipv4Addr::is_private()`])
    /// - 'loopback' address (see [`Ipv4Addr::is_loopback()`])
    /// - 'link-local' block (see [`Ipv4Addr::is_link_local()`])
    /// - 'broadcast' address (see [`Ipv4Addr::is_broadcast()`])
    /// - 'documentation' block (see [`Ipv4Addr::is_documentation()`])
    /// - 'unspecified' address (see [`Ipv4Addr::is_unspecified()`]), and the whole `0.0.0.0/8`
    ///   block
    /// - `192.0.0.0/24` block excluding `192.0.0.9/32` and `192.0.0.10/32` which are globally
    ///   routable
    /// - 'reserved' block (see [`Ipv4Addr::is_reserved()`]
    /// - 'benchmarking' block (see [`Ipv4Addr::is_benchmarking()`])
    ///
    /// [IANA]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
    #[inline]
    pub const fn is_global(&self) -> bool {
        // Port Control Protocol Anycast, and
        // Traversal Using Relays around NAT Anycast
        if self.0[0] == 192
            && self.0[1] == 0
            && self.0[2] == 0
            && (self.0[3] == 9 || self.0[3] == 10)
        {
            return true;
        }

        !self.is_private()
            && !self.is_loopback()
            && !self.is_link_local()
            && !self.is_broadcast()
            && !self.is_documentation()
            && !self.is_shared()
            // addresses reserved for future protocols (`192.0.0.0/24`)
            && !(self.0[0] == 192 && self.0[1] == 0 && self.0[2] == 0)
            && !self.is_reserved()
            && !self.is_benchmarking()
            // Make sure the address is not in 0.0.0.0/8
            && self.0[0] != 0
    }

    /// Returns `true` if address belongs to the 'shared' block `100.64.0.0/10`.
    #[inline]
    pub const fn is_shared(&self) -> bool {
        self.0[0] == 100 && (self.0[1] & 0b1100_0000 == 64)
    }

    /// Returns `true` if address belongs to the 'benchmarking' block `198.18.0.0/15`.
    #[inline]
    pub const fn is_benchmarking(&self) -> bool {
        self.0[0] == 192 && (self.0[1] & 0b1111_1110 == 18)
    }

    /// Returns `true` if address belongs to the 'reserved' block `240.0.0.0/4`, excluding the
    /// 'broadcast' address `255.255.255.255`.
    #[inline]
    pub const fn is_reserved(&self) -> bool {
        (self.0[0] & 0b1111_0000 == 240) && !self.is_broadcast()
    }

    /// Returns `true` if address belongs to the 'multicast' block `224.0.0.0/4`.
    #[inline]
    pub const fn is_multicast(&self) -> bool {
        self.0[0] & 0b1111_0000 == 224
    }

    /// Returns `true` if address is the 'broadcast' address `255.255.255.255`.
    #[inline]
    pub const fn is_broadcast(&self) -> bool {
        self.0[0] == 255 && self.0[1] == 255 && self.0[2] == 255 && self.0[3] == 255
    }

    /// Returns `true` if address belongs to the 'documentation' blocks:
    /// - `192.0.2.0/24`
    /// - `198.51.100.0/24`
    /// - `203.0.113.0/24`
    #[inline]
    pub const fn is_documentation(&self) -> bool {
        matches!(
            (self.0[0], self.0[1], self.0[2]),
            (192, 0, 2) | (198, 51, 100) | (203, 0, 113)
        )
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// IPv4 Flags
///
/// 3 bits total
/// Bit 0: reserved, must be zero
/// Bit 1: (DF) 0 = May Fragment,  1 = Don't Fragment.
/// Bit 2: (MF) 0 = Last Fragment, 1 = More Fragments.
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum Flags {
    LastFrag = 0b000,
    MoreFrag = 0b001,
    DontFrag = 0b010,
}

impl From<Flags> for u8 {
    #[inline]
    fn from(value: Flags) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Flags {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        match value {
            value if value == Self::LastFrag as u8 => Ok(Self::LastFrag),
            value if value == Self::MoreFrag as u8 => Ok(Self::MoreFrag),
            value if value == Self::DontFrag as u8 => Ok(Self::DontFrag),
            _ => Err(Error::Unsupported),
        }
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// ```text
///  0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+
/// |Version|  IHL  |
/// +-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct VerIhl([u8; 1]);

impl VerIhl {
    /// Returns a `u4`
    #[inline]
    pub fn version(&self) -> u8 {
        (self.0[0] & 0b1111_0000) >> 4
    }

    /// Assumes `value` is a `u4`
    #[inline]
    pub fn _set_version(&mut self, value: u8) {
        self.0[0] = (self.0[0] & 0b0000_1111) | (value << 4);
    }

    /// Returns a `u4`
    #[inline]
    pub fn header_len(&self) -> u8 {
        self.0[0] & 0b0000_1111
    }

    /// Assumes `value` is a `u4`
    #[inline]
    pub fn _set_header_len(&mut self, value: u8) {
        self.0[0] = (self.0[0] & 0b1111_0000) | (value & 0b0000_1111);
    }
}

/// ```text
///  0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+
/// |   DSCP    |ECN|
/// +-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct DiffServ([u8; 1]);

impl DiffServ {
    /// Returns a `u6`
    #[inline]
    pub fn dscp(&self) -> u8 {
        (self.0[0] & 0b1111_1100) >> 2
    }

    /// Assumes `value` is a `u6`
    #[inline]
    pub fn _set_dscp(&mut self, value: u8) {
        self.0[0] = (self.0[0] & 0b0000_0011) | (value << 2);
    }

    /// Returns a `u2`
    #[inline]
    pub fn ecn(&self) -> u8 {
        self.0[0] & 0b0000_0011
    }

    /// Assumes `value` is a `u2`
    #[inline]
    pub fn _set_ecn(&mut self, value: u8) {
        self.0[0] = (self.0[0] & 0b1111_1100) | (value & 0b0000_0011);
    }
}

/// ```text
///  0 1 2 3 4 5 6 7 8 9 A B C D E F
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Flags|    Fragment Offset      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct FlagsFragOffset([u8; 2]);

impl FlagsFragOffset {
    /// Returns a `u3`
    #[inline]
    pub fn flags(&self) -> u8 {
        (self.0[0] & 0b1110_0000) >> 5
    }

    /// Assumes `value` is a `u3`
    #[inline]
    pub fn _set_flags(&mut self, value: u8) {
        self.0[0] = (self.0[0] & 0b0001_1111) | (value << 5);
    }

    /// Returns a `u13`
    #[inline]
    pub fn frag_offset(&self) -> u16 {
        u16::from_be_bytes(self.0) & 0b0001_1111_1111_1111
    }

    /// Assumes `value` is a `u13`
    #[inline]
    pub fn _set_frag_offset(&mut self, value: u16) {
        let bytes = u16::to_be_bytes(value);
        self.0[0] = (self.0[0] & 0b1110_0000) | (bytes[0] & 0b0001_1111);
        self.0[1] = bytes[1];
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 19];
        assert_eq!(Ipv4::split_header(&bytes).unwrap_err(), Error::Truncated);
    }
}

//! [`Ipv4`] header
//!
//! [`Ipv4`] header supporting 32-bit addressing (see [`Ipv4Addr`]) and fragmentation.
use core::fmt;
use core::mem::size_of;

use crate::header::checksum::verify_checksum;
use crate::header::error::{Error, Result};
use crate::header::primitive::{U16, U8};
use crate::header::utils::{as_header, return_err, return_err_if, split_at};

use super::ip::{Dscp, Ecn, IpProtocol, IpVersion, ProtocolRepr};
use super::StdDscp;

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
    flag_frag: FlagsFragOffset,
    ttl: U8,
    proto: ProtocolRepr,
    cks: U16,
    src: Ipv4Addr,
    dst: Ipv4Addr,
}

impl Ipv4 {
    /// Returns an immutable view if `bytes` an an IPv4 header followed by a payload or an error if
    /// the size or contents do not represent a valid IPv4 header. Since IPv4 options are dynamic in
    /// length, they are not included in the header and are instead returned as a split payload.
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8], &[u8])> {
        let (header, options_payload) = match as_header!(Ipv4, bytes) {
            Some(v) => v,
            None => return Err(Error::Truncated),
        };

        return_err!(header.ver_ihl.verify());
        return_err!(header.diff_serv.verify());
        return_err!(header.flag_frag.verify());
        return_err!(header.proto.verify());

        let full_header = match split_at(bytes, header.header_len()) {
            Some(v) => v,
            None => return Err(Error::Truncated),
        }
        .0;
        return_err!(verify_checksum(full_header));

        let (options, payload) = match split_at(options_payload, header.options_len()) {
            Some(v) => v,
            None => return Err(Error::Truncated),
        };

        return_err_if!(header.total_len() as usize != bytes.len(), Error::Truncated);

        Ok((header, options, payload))
    }

    /// Always returns [`IpVersion::Ipv4`].
    #[inline]
    pub const fn version(&self) -> IpVersion {
        self.ver_ihl.version()
    }

    /// Returns the length of the IPv4 header in bytes. At a minimum, the length of a header with no
    /// options is 20 bytes. A header with the maximum amount of options/padding has a length of 60
    /// bytes.
    #[inline]
    pub const fn header_len(&self) -> usize {
        self.ver_ihl.header_len() * 4
    }

    /// Returns the length of the IPv4 options in bytes. Options are optional. A header may have up
    /// to 40 bytes of options.
    #[inline]
    pub const fn options_len(&self) -> usize {
        // Options start after 20 bytes of header
        (self.ver_ihl.header_len() * 4).saturating_sub(size_of::<Ipv4>())
    }

    /// Returns Differentiated Services codepoint (DSCP) used to classify and manage network traffic.
    #[inline]
    pub const fn dscp(&self) -> Dscp {
        self.diff_serv.dscp()
    }

    /// Returns Explicit Congestion Notification (ECN) an optional feature used to indicate
    /// impending network traffic congestion.
    #[inline]
    pub const fn ecn(&self) -> Ecn {
        self.diff_serv.ecn()
    }

    /// Returns the total length of the IPv4 packet. Does not include link layer header.
    ///
    /// Note: for fragmented packets, total length indicates the length of the fragment, not the
    /// assembled packet.
    #[inline]
    pub const fn total_len(&self) -> u16 {
        self.tlen.get()
    }

    /// Returns ID of the IPv4 packet. Fragments with the same ID are assembled together.
    #[inline]
    pub const fn id(&self) -> u16 {
        self.id.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn flags(&self) -> Ipv4Flags {
        self.flag_frag.flags()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn offset(&self) -> u16 {
        self.flag_frag.frag_offset()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn ttl(&self) -> u8 {
        self.ttl.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn protocol(&self) -> IpProtocol {
        self.proto.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn cks(&self) -> u16 {
        self.cks.get()
    }

    /// Returns source IPv4 address.
    #[inline]
    pub const fn src(&self) -> Ipv4Addr {
        self.src
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn dst(&self) -> Ipv4Addr {
        self.dst
    }
}

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IPv4 ({}) {} → {} ttl={} len={}",
            self.protocol(),
            self.src(),
            self.dst(),
            self.ttl(),
            self.total_len(),
        )?;
        if self.options_len() != 0 {
            write!(f, "({})", self.options_len())?;
        }

        match (self.flags(), self.offset()) {
            (Ipv4Flags::DontFrag, _) | (Ipv4Flags::LastFrag, 0) => {
                write!(f, " flags={}", self.flags())?
            }
            (flags, offset) => write!(f, " flags={flags} (id={:#x} offset={offset})", self.id())?,
        };

        match self.ecn() {
            Ecn::NonECT => {}
            ecn => write!(f, " ecn={ecn}")?,
        };

        match StdDscp::try_from(self.dscp()) {
            Ok(StdDscp::CS0) => {}
            Ok(dscp) => write!(f, " dscp={dscp}")?,
            Err(_) => write!(f, " dscp={}", self.dscp())?,
        };

        Ok(())
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
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
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
pub enum Ipv4Flags {
    LastFrag = 0b000,
    MoreFrag = 0b001,
    DontFrag = 0b010,
    // 0b011 is invalid
}

impl Ipv4Flags {
    #[inline]
    pub(crate) const fn new(value: u8) -> Result<Self> {
        match value {
            value if value == Self::LastFrag as u8 => Ok(Self::LastFrag),
            value if value == Self::MoreFrag as u8 => Ok(Self::MoreFrag),
            value if value == Self::DontFrag as u8 => Ok(Self::DontFrag),
            _ => Err(Error::Malformed),
        }
    }
}

impl From<Ipv4Flags> for u8 {
    #[inline]
    fn from(value: Ipv4Flags) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Ipv4Flags {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        Ipv4Flags::new(value)
    }
}

impl fmt::Display for Ipv4Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ipv4Flags::LastFrag => write!(f, "LF"),
            Ipv4Flags::MoreFrag => write!(f, "MF"),
            Ipv4Flags::DontFrag => write!(f, "DF"),
        }
    }
}

/// ```text
///  0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+
/// |Version|  IHL  |
/// +-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct VerIhl(U8);

impl VerIhl {
    #[inline]
    pub(crate) const fn verify(&self) -> Result<()> {
        // version must be 4
        // minimum header length must be at least 5
        if self._version() != 4 && self.header_len() < 5 {
            return Err(Error::Malformed);
        }

        Ok(())
    }

    // Returns a `u4`
    #[inline]
    const fn _version(&self) -> u8 {
        (self.0.get() & 0b1111_0000) >> 4
    }

    /// Returns [`IpVersion::Ipv4`]
    #[inline]
    pub const fn version(&self) -> IpVersion {
        IpVersion::Ipv4
    }

    /// Returns a `u4`
    #[inline]
    pub const fn header_len(&self) -> usize {
        (self.0.get() & 0b0000_1111) as usize
    }
}

/// ```text
///  0 1 2 3 4 5 6 7
/// +-+-+-+-+-+-+-+-+
/// |   DSCP    |ECN|
/// +-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct DiffServ(U8);

impl DiffServ {
    pub(crate) const fn verify(&self) -> Result<()> {
        // nothing to check here
        Ok(())
    }

    /// Returns a `u6`
    #[inline]
    pub const fn dscp(&self) -> Dscp {
        Dscp((self.0.get() & 0b1111_1100) >> 2)
    }

    /// Returns a `u2`
    #[inline]
    pub const fn ecn(&self) -> Ecn {
        match Ecn::new(self.0.get() & 0b0000_0011) {
            Ok(ecn) => ecn,
            Err(_) => unreachable!(),
        }
    }
}

/// ```text
///  0 1 2 3 4 5 6 7 8 9 A B C D E F
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Flags|    Fragment Offset      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct FlagsFragOffset(U16);

const FLAG_MASK: u16 = 0b1110_0000_0000_0000;
const FLAG_SHIFT: usize = 13;

const FRAG_OFFSET_MASK: u16 = 0b0001_1111_1111_1111;
const FRAG_OFFSET_SHIFT: usize = 0;

impl FlagsFragOffset {
    pub(crate) const fn verify(&self) -> Result<()> {
        if let Err(e) = Ipv4Flags::new(self._flags()) {
            return Err(e);
        }

        // all values of fragment offset are valid
        Ok(())
    }

    /// Returns a `u3`
    #[inline]
    pub const fn flags(&self) -> Ipv4Flags {
        match Ipv4Flags::new(self._flags()) {
            Ok(flags) => flags,
            Err(_) => unreachable!(),
        }
    }

    #[inline]
    pub const fn frag_offset(&self) -> u16 {
        self._frag_offset()
    }

    // Returns a `u3`
    #[inline]
    pub const fn _flags(&self) -> u8 {
        ((self.0.get() & FLAG_MASK) >> FLAG_SHIFT) as u8
    }

    // Returns a `u13`
    #[inline]
    pub const fn _frag_offset(&self) -> u16 {
        (self.0.get() & FRAG_OFFSET_MASK) >> FRAG_OFFSET_SHIFT
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 19];
        assert_eq!(Ipv4::from_bytes(&bytes).unwrap_err(), Error::Truncated);
    }
}

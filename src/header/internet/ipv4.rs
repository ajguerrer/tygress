//! [`Ipv4`] header
//!
//! [`Ipv4`] header supporting 32-bit addressing (see [`Ipv4Addr`]) and fragmentation.
use core::fmt;
use core::mem::size_of;

use crate::header::error::HeaderTruncated;
use crate::header::primitive::{non_exhaustive_enum, U16, U8};
use crate::header::utils::{as_header, split_at};

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
pub struct Ipv4<'a> {
    required: &'a Ipv4Required,
    options: &'a [u8],
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
struct Ipv4Required {
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

impl<'a> Ipv4<'a> {
    /// Returns an immutable view if `bytes` an an IPv4 header followed by a payload or an error if
    /// the size or contents do not represent a valid IPv4 header. Since IPv4 options are dynamic in
    /// length, they are not included in the header and are instead returned as a split payload.
    #[inline]
    pub const fn from_bytes(bytes: &'a [u8]) -> Result<(Self, &[u8]), HeaderTruncated> {
        let (required, options_payload) = match as_header!(Ipv4Required, bytes) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let (options, payload) = match split_at(options_payload, required.ver_ihl.options_len()) {
            Some(v) => v,
            None => return Err(HeaderTruncated),
        };

        Ok((Ipv4 { required, options }, payload))
    }

    /// Always returns [`IpVersion::Ipv4`].
    #[inline]
    pub const fn version(&self) -> IpVersion {
        self.required.ver_ihl.version()
    }

    /// Returns the length of the IPv4 header in bytes. At a minimum, the length of a header with no
    /// options is 20 bytes. A header with the maximum amount of options/padding has a length of 60
    /// bytes.
    #[inline]
    pub const fn header_len(&self) -> usize {
        self.required.ver_ihl.header_len()
    }

    /// Returns the length of the IPv4 options in bytes. Options are optional. A header may have up
    /// to 40 bytes of options.
    #[inline]
    pub const fn options_len(&self) -> usize {
        // Options start after 20 bytes of header
        self.required.ver_ihl.options_len()
    }

    /// Returns Differentiated Services codepoint (DSCP) used to classify and manage network traffic.
    #[inline]
    pub const fn dscp(&self) -> Dscp {
        self.required.diff_serv.dscp()
    }

    /// Returns Explicit Congestion Notification (ECN) an optional feature used to indicate
    /// impending network traffic congestion.
    #[inline]
    pub const fn ecn(&self) -> Ecn {
        self.required.diff_serv.ecn()
    }

    /// Returns the total length of the IPv4 packet. Does not include link layer header.
    ///
    /// Note: for fragmented packets, total length indicates the length of the fragment, not the
    /// assembled packet.
    #[inline]
    pub const fn total_len(&self) -> u16 {
        self.required.tlen.get()
    }

    /// Returns ID of the IPv4 packet. Fragments with the same ID are assembled together.
    #[inline]
    pub const fn id(&self) -> u16 {
        self.required.id.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn flags(&self) -> Ipv4Flags {
        self.required.flag_frag.flags()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn offset(&self) -> u16 {
        self.required.flag_frag.frag_offset()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn ttl(&self) -> u8 {
        self.required.ttl.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn protocol(&self) -> IpProtocol {
        self.required.proto.get()
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn cks(&self) -> u16 {
        self.required.cks.get()
    }

    /// Returns source IPv4 address.
    #[inline]
    pub const fn src(&self) -> Ipv4Addr {
        self.required.src
    }

    /// Returns destination IPv4 address.
    #[inline]
    pub const fn dst(&self) -> Ipv4Addr {
        self.required.dst
    }

    /// Returns iterator IPv4 of [`Ipv4Option`]
    #[inline]
    pub const fn options(&self) -> Ipv4Options {
        Ipv4Options {
            options: self.options,
        }
    }
}

impl<'a> fmt::Display for Ipv4<'a> {
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
            (Ipv4Flags::DF, _) | (Ipv4Flags::LF, 0) => write!(f, " flags={}", self.flags())?,
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

/// Iterator of [`Ipv4Option`].
///
/// Currently no Ipv4 options are supported, so iterating immediately returns [`None`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Ipv4Options<'a> {
    options: &'a [u8],
}

impl<'a> Iterator for Ipv4Options<'a> {
    type Item = Ipv4Option;

    fn next(&mut self) -> Option<Self::Item> {
        let _options = self.options;
        None
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Ipv4Option {}

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

non_exhaustive_enum! {
/// IPv4 Flags
///
/// 3 bits total
/// Bit 0: reserved, must be zero
/// Bit 1: (DF) 0 = May Fragment,  1 = Don't Fragment.
/// Bit 2: (MF) 0 = Last Fragment, 1 = More Fragments.
pub enum Ipv4Flags(u8) {
    /// Last Fragment
    LF = 0b000,
    /// More Fragments
    MF = 0b001,
    /// Don't Fragment
    DF = 0b010,
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
    /// Should return [`IpVersion::Ipv4`]
    #[inline]
    pub const fn version(&self) -> IpVersion {
        IpVersion::new((self.0.get() & 0b1111_0000) >> 4)
    }

    /// Returns IPv4 header length in bytes
    #[inline]
    pub const fn header_len(&self) -> usize {
        (self.0.get() & 0b0000_1111) as usize * 4
    }

    /// Returns IPv4 header length minus required portion of header (20 bytes)
    #[inline]
    pub const fn options_len(&self) -> usize {
        (self.header_len()).saturating_sub(size_of::<Ipv4Required>())
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
    const DSCP_MASK: u8 = 0b1111_1100;
    const DSCP_SHIFT: usize = 2;

    const ECN_MASK: u8 = 0b0000_0011;
    const ECN_SHIFT: usize = 0;

    /// Returns a `u6`
    #[inline]
    pub const fn dscp(&self) -> Dscp {
        Dscp((self.0.get() & Self::DSCP_MASK) >> Self::DSCP_SHIFT)
    }

    /// Returns a `u2`
    #[inline]
    pub const fn ecn(&self) -> Ecn {
        Ecn::new((self.0.get() & Self::ECN_MASK) >> Self::ECN_SHIFT)
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

impl FlagsFragOffset {
    const FLAG_MASK: u16 = 0b1110_0000_0000_0000;
    const FLAG_SHIFT: usize = 13;

    const FRAG_OFFSET_MASK: u16 = 0b0001_1111_1111_1111;
    const FRAG_OFFSET_SHIFT: usize = 0;

    /// Returns a `u3`
    #[inline]
    pub const fn flags(&self) -> Ipv4Flags {
        Ipv4Flags::new(((self.0.get() & Self::FLAG_MASK) >> Self::FLAG_SHIFT) as u8)
    }

    #[inline]
    pub const fn frag_offset(&self) -> u16 {
        (self.0.get() & Self::FRAG_OFFSET_MASK) >> Self::FRAG_OFFSET_SHIFT
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 19];
        assert_eq!(Ipv4::from_bytes(&bytes).unwrap_err(), HeaderTruncated);
    }
}

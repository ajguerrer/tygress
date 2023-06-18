use core::fmt;

use crate::header::{
    error::{Error, Result},
    primitive::U8,
};

/// An IP version number.
///
/// Version of IP protocol used by an IP packet. Supported versions are IPv4 and IPv6.
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum IpVersion {
    Ipv4 = 4,
    Ipv6 = 6,
}

impl IpVersion {
    /// Return version field of an IP packet stored in bytes.
    ///
    /// Version is a field common to IP packets of all versions. This function determines the
    /// version of an IP packet so the IP header may be interpreted correctly.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<IpVersion> {
        match bytes.first() {
            Some(byte) => match byte >> 4 {
                4 => Ok(IpVersion::Ipv4),
                6 => Ok(IpVersion::Ipv6),
                _ => Err(Error::Unsupported),
            },
            None => Err(Error::Truncated),
        }
    }
}

impl From<IpVersion> for u8 {
    #[inline]
    fn from(value: IpVersion) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for IpVersion {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        match value {
            value if value == Self::Ipv4 as u8 => Ok(Self::Ipv4),
            value if value == Self::Ipv6 as u8 => Ok(Self::Ipv6),
            _ => Err(Error::Unsupported),
        }
    }
}

impl fmt::Display for IpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// An IP protocol number. [Read more][IANA]
///
/// Identifies the protocol encapsulated within the IP packet. A complete list of protocols is
/// maintained by the [IANA].
///
/// [IANA]: https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum IpProtocol {
    HopByHop = 0x00,
    ICMP = 0x01,
    IGMP = 0x02,
    TCP = 0x06,
    UDP = 0x11,
    IPv6Route = 0x2b,
    IPv6Frag = 0x2c,
    ICMPv6 = 0x3a,
    IPv6NoNxt = 0x3b,
    IPv6Opts = 0x3c,
}

impl From<IpProtocol> for u8 {
    #[inline]
    fn from(value: IpProtocol) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for IpProtocol {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        match value {
            value if value == Self::HopByHop as u8 => Ok(Self::HopByHop),
            value if value == Self::ICMP as u8 => Ok(Self::ICMP),
            value if value == Self::IGMP as u8 => Ok(Self::IGMP),
            value if value == Self::TCP as u8 => Ok(Self::TCP),
            value if value == Self::UDP as u8 => Ok(Self::UDP),
            value if value == Self::IPv6Route as u8 => Ok(Self::IPv6Route),
            value if value == Self::IPv6Frag as u8 => Ok(Self::IPv6Frag),
            value if value == Self::ICMPv6 as u8 => Ok(Self::ICMPv6),
            value if value == Self::IPv6NoNxt as u8 => Ok(Self::IPv6NoNxt),
            value if value == Self::IPv6Opts as u8 => Ok(Self::IPv6Opts),
            _ => Err(Error::Unsupported),
        }
    }
}

impl fmt::Display for IpProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// An array representing [`Protocol`] cast from a slice of bytes instead of constructed. It is
/// assumed that [`verify`][ProtocolRepr::verify] is called directly after casting before any other
/// methods are called.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub(crate) struct ProtocolRepr(U8);

impl ProtocolRepr {
    const HOPBYHOP: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::HopByHop as u8));
    const ICMP: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::ICMP as u8));
    const IGMP: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::IGMP as u8));
    const TCP: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::TCP as u8));
    const UDP: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::UDP as u8));
    const IPV6ROUTE: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::IPv6Route as u8));
    const IPV6FRAG: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::IPv6Frag as u8));
    const ICMPV6: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::ICMPv6 as u8));
    const IPV6NONXT: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::IPv6NoNxt as u8));
    const IPV6OPTS: ProtocolRepr = ProtocolRepr(U8::new(IpProtocol::IPv6Opts as u8));

    /// Check inner self for validity.
    #[inline]
    pub(crate) const fn verify(&self) -> Result<()> {
        match *self {
            Self::HOPBYHOP
            | Self::ICMP
            | Self::IGMP
            | Self::TCP
            | Self::UDP
            | Self::IPV6ROUTE
            | Self::IPV6FRAG
            | Self::ICMPV6
            | Self::IPV6NONXT
            | Self::IPV6OPTS => Ok(()),
            _ => Err(Error::Unsupported),
        }
    }

    /// Get the underlying [`Protocol`].
    #[inline]
    pub(crate) const fn get(&self) -> IpProtocol {
        match *self {
            Self::HOPBYHOP => IpProtocol::HopByHop,
            Self::ICMP => IpProtocol::ICMP,
            Self::IGMP => IpProtocol::IGMP,
            Self::TCP => IpProtocol::TCP,
            Self::UDP => IpProtocol::UDP,
            Self::IPV6ROUTE => IpProtocol::IPv6Route,
            Self::IPV6FRAG => IpProtocol::IPv6Frag,
            Self::ICMPV6 => IpProtocol::ICMPv6,
            Self::IPV6NONXT => IpProtocol::IPv6NoNxt,
            Self::IPV6OPTS => IpProtocol::IPv6Opts,
            _ => unreachable!(),
        }
    }
}

impl From<IpProtocol> for ProtocolRepr {
    #[inline]
    fn from(value: IpProtocol) -> Self {
        ProtocolRepr(U8::new(value as u8))
    }
}

impl fmt::Display for ProtocolRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

/// A Differentiated Services codepoint (DSCP). [Read more][RFC 2474]
///
/// DSCP selects the per-hop behavior (PHB) a packet experiences at each node.
///
/// There is a standardized PHB to DSCP mapping which serves as a framework for service providers.
/// With the exception of Class Selector codepoints defined as 'xxx000', it is not required to
/// adhere to the standard. The standard mapping is recorded in the [StdDscp] enum taken from
/// [IANA].
///
/// There must be a default PHB and it is recommended that it correspond to the codepoint '000000'
/// (Default Forwarding).
///
/// [RFC 2474]: https://tools.ietf.org/html/rfc2474
/// [IANA]: https://www.iana.org/assignments/dscp-registry/dscp-registry.xhtml
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(transparent)]
pub struct Dscp(pub(crate) u8);

impl From<Dscp> for u8 {
    #[inline]
    fn from(value: Dscp) -> Self {
        value.0
    }
}

impl TryFrom<u8> for Dscp {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        if value <= 0b111111 {
            Ok(Dscp(value))
        } else {
            Err(Error::Unsupported)
        }
    }
}

impl fmt::Display for Dscp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// A standardized listing of [Dscp]s as defined by the [IANA].
///
/// [IANA]: https://www.iana.org/assignments/dscp-registry/dscp-registry.xhtml
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum StdDscp {
    CS0 = 0b000000,
    CS1 = 0b001000,
    CS2 = 0b010000,
    CS3 = 0b011000,
    CS4 = 0b100000,
    CS5 = 0b101000,
    CS6 = 0b110000,
    CS7 = 0b111000,
    AF11 = 0b001010,
    AF12 = 0b001100,
    AF13 = 0b001110,
    AF21 = 0b010010,
    AF22 = 0b010100,
    AF23 = 0b010110,
    AF31 = 0b011010,
    AF32 = 0b011100,
    AF33 = 0b011110,
    AF41 = 0b100010,
    AF42 = 0b100100,
    AF43 = 0b100110,
    EF = 0b101110,
    VoiceAdmit = 0b101100,
    LE = 0b000001,
}

impl From<StdDscp> for u8 {
    #[inline]
    fn from(value: StdDscp) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for StdDscp {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        match value {
            value if value == Self::CS0 as u8 => Ok(Self::CS0),
            value if value == Self::CS1 as u8 => Ok(Self::CS1),
            value if value == Self::CS2 as u8 => Ok(Self::CS2),
            value if value == Self::CS3 as u8 => Ok(Self::CS3),
            value if value == Self::CS4 as u8 => Ok(Self::CS4),
            value if value == Self::CS5 as u8 => Ok(Self::CS5),
            value if value == Self::CS6 as u8 => Ok(Self::CS6),
            value if value == Self::CS7 as u8 => Ok(Self::CS7),
            value if value == Self::AF11 as u8 => Ok(Self::AF11),
            value if value == Self::AF12 as u8 => Ok(Self::AF12),
            value if value == Self::AF13 as u8 => Ok(Self::AF13),
            value if value == Self::AF21 as u8 => Ok(Self::AF21),
            value if value == Self::AF22 as u8 => Ok(Self::AF22),
            value if value == Self::AF23 as u8 => Ok(Self::AF23),
            value if value == Self::AF31 as u8 => Ok(Self::AF31),
            value if value == Self::AF32 as u8 => Ok(Self::AF32),
            value if value == Self::AF33 as u8 => Ok(Self::AF33),
            value if value == Self::AF41 as u8 => Ok(Self::AF41),
            value if value == Self::AF42 as u8 => Ok(Self::AF42),
            value if value == Self::AF43 as u8 => Ok(Self::AF43),
            value if value == Self::EF as u8 => Ok(Self::EF),
            value if value == Self::VoiceAdmit as u8 => Ok(Self::VoiceAdmit),
            value if value == Self::LE as u8 => Ok(Self::LE),
            _ => Err(Error::Unsupported),
        }
    }
}

impl From<StdDscp> for Dscp {
    #[inline]
    fn from(value: StdDscp) -> Self {
        Self(value as u8)
    }
}

impl TryFrom<Dscp> for StdDscp {
    type Error = Error;

    #[inline]
    fn try_from(value: Dscp) -> Result<Self> {
        Self::try_from(value.0)
    }
}

impl fmt::Display for StdDscp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// A Explicit Congestion Notification (ECN). [Read more][RFC 3168]
///
/// ECN is an optional feature which, when combined with protocol-specific support in the transport
/// layer, provides alternative ways relieve congestion than simply dropping packets.
///
/// Routers that support Active Queue Management (AQM), a practice recommended by [RFC 7567], may
/// set the ECN to CE indicating congestion encountered.
///
/// Endpoints using a ECN-capable transport protocol set the ECN to ECT(0) or ECT(1).
///
/// [RFC 3168]: https://tools.ietf.org/html/rfc3168
/// [RFC 7567]: https://tools.ietf.org/html/rfc7567
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum Ecn {
    /// Non ECN-Capable Transport
    NonECT = 0b00,
    /// ECN Capable Transport 0
    ECT0 = 0b10,
    /// ECN Capable Transport 1
    ECT1 = 0b01,
    /// Congestion Encountered
    CE = 0b11,
}

impl Ecn {
    #[inline]
    pub(crate) const fn new(value: u8) -> Result<Self> {
        match value {
            value if value == Self::NonECT as u8 => Ok(Self::NonECT),
            value if value == Self::ECT0 as u8 => Ok(Self::ECT0),
            value if value == Self::ECT1 as u8 => Ok(Self::ECT1),
            value if value == Self::CE as u8 => Ok(Self::CE),
            _ => Err(Error::Malformed),
        }
    }
}

impl From<Ecn> for u8 {
    #[inline]
    fn from(value: Ecn) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Ecn {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        Ecn::new(value)
    }
}

impl fmt::Display for Ecn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

use core::fmt;

use crate::error::{Error, Result};

/// An IP version number.
///
/// Version of IP protocol used by an IP packet. Supported versions are IPv4 and IPv6.
#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u8)]
pub enum Version {
    Ipv4 = 4,
    Ipv6 = 6,
}

impl From<Version> for u8 {
    #[inline]
    fn from(value: Version) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Version {
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

impl fmt::Display for Version {
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
pub enum Protocol {
    HopByHop = 0x00,
    Icmp = 0x01,
    Igmp = 0x02,
    Tcp = 0x06,
    Udp = 0x11,
    Ipv6Route = 0x2b,
    Ipv6Frag = 0x2c,
    Icmpv6 = 0x3a,
    Ipv6NoNxt = 0x3b,
    Ipv6Opts = 0x3c,
}

impl From<Protocol> for u8 {
    #[inline]
    fn from(value: Protocol) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Protocol {
    type Error = Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self> {
        match value {
            value if value == Self::HopByHop as u8 => Ok(Self::HopByHop),
            value if value == Self::Icmp as u8 => Ok(Self::Icmp),
            value if value == Self::Igmp as u8 => Ok(Self::Igmp),
            value if value == Self::Tcp as u8 => Ok(Self::Tcp),
            value if value == Self::Udp as u8 => Ok(Self::Udp),
            value if value == Self::Ipv6Route as u8 => Ok(Self::Ipv6Route),
            value if value == Self::Ipv6Frag as u8 => Ok(Self::Ipv6Frag),
            value if value == Self::Icmpv6 as u8 => Ok(Self::Icmpv6),
            value if value == Self::Ipv6NoNxt as u8 => Ok(Self::Ipv6NoNxt),
            value if value == Self::Ipv6Opts as u8 => Ok(Self::Ipv6Opts),
            _ => Err(Error::Unsupported),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

/// An array representing [`Protocol`] cast from a slice of bytes instead of constructed. It is
/// assumed that [`check`][ProtocolRepr::check] is called directly after casting before any other
/// methods are called.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub(crate) struct ProtocolRepr([u8; 1]);

impl ProtocolRepr {
    const HOPBYHOP: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::HopByHop as u8));
    const ICMP: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Icmp as u8));
    const IGMP: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Igmp as u8));
    const TCP: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Tcp as u8));
    const UDP: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Udp as u8));
    const IPV6ROUTE: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Ipv6Route as u8));
    const IPV6FRAG: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Ipv6Frag as u8));
    const ICMPV6: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Icmpv6 as u8));
    const IPV6NONXT: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Ipv6NoNxt as u8));
    const IPV6OPTS: ProtocolRepr = ProtocolRepr(u8::to_be_bytes(Protocol::Ipv6Opts as u8));

    /// Check inner self for validity.
    #[inline]
    pub(crate) const fn check(&self) -> Result<()> {
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
    pub(crate) const fn get(&self) -> Protocol {
        match *self {
            Self::HOPBYHOP => Protocol::HopByHop,
            Self::ICMP => Protocol::Icmp,
            Self::IGMP => Protocol::Igmp,
            Self::TCP => Protocol::Tcp,
            Self::UDP => Protocol::Udp,
            Self::IPV6ROUTE => Protocol::Ipv6Route,
            Self::IPV6FRAG => Protocol::Ipv6Frag,
            Self::ICMPV6 => Protocol::Icmpv6,
            Self::IPV6NONXT => Protocol::Ipv6NoNxt,
            Self::IPV6OPTS => Protocol::Ipv6Opts,
            _ => unreachable!(),
        }
    }
}

impl From<Protocol> for ProtocolRepr {
    #[inline]
    fn from(value: Protocol) -> Self {
        ProtocolRepr(u8::to_be_bytes(value as u8))
    }
}

impl fmt::Display for ProtocolRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

/// A Differentiated Services Codepoint (DSCP). [Read more][RFC 2474]
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
        value.0 as u8
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
    NonEct = 0b00,
    /// ECN Capable Transport 0
    Ect0 = 0b10,
    /// ECN Capable Transport 1
    Ect1 = 0b01,
    /// Congestion Encountered
    Ce = 0b11,
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
        match value {
            value if value == Self::NonEct as u8 => Ok(Self::NonEct),
            value if value == Self::Ect0 as u8 => Ok(Self::Ect0),
            value if value == Self::Ect1 as u8 => Ok(Self::Ect1),
            value if value == Self::Ce as u8 => Ok(Self::Ce),
            _ => Err(Error::Unsupported),
        }
    }
}

impl fmt::Display for Ecn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

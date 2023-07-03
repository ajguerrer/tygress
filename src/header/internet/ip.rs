use core::fmt;

use crate::header::{
    error::ValueToLarge,
    primitive::{non_exhaustive_enum, U8},
};

non_exhaustive_enum! {
/// An IP version number.
///
/// Version of IP protocol used by an IP packet. Supported versions are IPv4 and IPv6.
pub enum IpVersion(u8) {
    Ipv4 = 4,
    Ipv6 = 6,
}
}

non_exhaustive_enum! {
/// An IP protocol number. [Read more][IANA]
///
/// Identifies the protocol encapsulated within the IP packet. A complete list of protocols is
/// maintained by the [IANA].
///
/// [IANA]: https://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml
pub enum IpProtocol(u8) {
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
}

/// An array representing [`Protocol`] cast from a slice of bytes instead of constructed.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub(crate) struct ProtocolRepr(U8);

impl ProtocolRepr {
    #[inline]
    pub(crate) const fn get(&self) -> IpProtocol {
        IpProtocol::new(self.0.get())
    }
}

impl From<IpProtocol> for ProtocolRepr {
    #[inline]
    fn from(value: IpProtocol) -> Self {
        ProtocolRepr(U8::new(value.get()))
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
    type Error = ValueToLarge;

    fn try_from(value: u8) -> Result<Self, ValueToLarge> {
        if value <= 0b111111 {
            Ok(Dscp(value))
        } else {
            Err(ValueToLarge)
        }
    }
}

impl fmt::Display for Dscp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

non_exhaustive_enum! {
/// A standardized listing of [Dscp]s as defined by the [IANA].
///
/// [IANA]: https://www.iana.org/assignments/dscp-registry/dscp-registry.xhtml
pub enum StdDscp(u8) {
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
}

impl From<StdDscp> for Dscp {
    #[inline]
    fn from(value: StdDscp) -> Self {
        Self(value.get())
    }
}

impl From<Dscp> for StdDscp {
    #[inline]
    fn from(value: Dscp) -> Self {
        Self::from(value.0)
    }
}

non_exhaustive_enum! {
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
pub enum Ecn(u8) {
    /// Non ECN-Capable Transport
    NonECT = 0b00,
    /// ECN Capable Transport 0
    ECT0 = 0b10,
    /// ECN Capable Transport 1
    ECT1 = 0b01,
    /// Congestion Encountered
    CE = 0b11,
}
}

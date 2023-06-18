use core::{fmt, mem::size_of};

use super::Ipv4Addr;
use crate::header::{
    link::{EtherAddr, EtherType as Protocol, EtherTypeRepr as ProtocolRepr},
    primitive::{U16, U8},
    utils::{as_header, return_err, return_err_if},
    Error, Result,
};

#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u16)]
pub enum Hardware {
    Ethernet = 1,
}

impl fmt::Display for Hardware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[non_exhaustive]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
#[repr(u16)]
pub enum Operation {
    Request = 1,
    Reply = 2,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub struct Arp {
    htype: HardwareRepr,
    ptype: ProtocolRepr,
    hlen: U8,
    plen: U8,
    oper: OperationRepr,
    src_haddr: EtherAddr,
    src_paddr: Ipv4Addr,
    dest_haddr: EtherAddr,
    dest_paddr: Ipv4Addr,
}

/// An Ipv4 Ethernet Address Resolution Protocol (ARP) header. [Read more][RFC 826]
///
/// Acting as a bridge between the internet and link layers, ARP is used to resolve the link layer
/// address for a given internet address. It can also be used to announce the sender address. This
/// is known as a gratuitous ARP (GARP).
///
/// Though ARP has plenty of room for future hardware and protocol types, in practice ARP is only
/// used to resolve Ethernet addresses from Ipv4 addresses. Validation will fail if any other
/// hardware or protocol types are used.
impl Arp {
    /// Returns an immutable view of `bytes` as an Arp header followed by a payload or an error if
    /// the size or contents do not represent a valid Arp header.
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8])> {
        let (header, payload) = match as_header!(Arp, bytes) {
            Some(v) => v,
            None => return Err(Error::Truncated),
        };

        return_err!(header.htype.verify());
        return_err!(header.ptype.verify());
        return_err_if!(
            header.ptype.get() as u16 != Protocol::Ipv4 as u16,
            Error::Unsupported
        );
        return_err!(header.oper.verify());
        return_err_if!(
            header.hlen.get() as usize != size_of::<EtherAddr>(),
            Error::Unsupported
        );
        return_err_if!(
            header.plen.get() as usize != size_of::<Ipv4Addr>(),
            Error::Unsupported
        );

        Ok((header, payload))
    }

    /// Link layer hardware type, always [`Hardware::Ethernet`].
    #[inline]
    pub const fn hardware(&self) -> Hardware {
        self.htype.get()
    }

    /// Internet layer protocol type, always [`Protocol::Ipv4`]. Shares the same numbering space as
    /// [`crate::header::link::EtherType`].
    #[inline]
    pub const fn protocol(&self) -> Protocol {
        self.ptype.get()
    }

    /// Link layer hardware address length in bytes. Always `size_of::<EtherAddr>() == 6`.
    #[inline]
    pub const fn hw_addr_len(&self) -> u8 {
        self.hlen.get()
    }

    /// Internet layer address length in bytes. Always `size_of::<Ipv4Addr>() == 4`.
    #[inline]
    pub const fn proto_addr_len(&self) -> u8 {
        self.plen.get()
    }

    #[inline]
    pub const fn operation(&self) -> Operation {
        self.oper.get()
    }

    /// Source link layer [`EtherAddr`].
    #[inline]
    pub const fn src_hw_addr(&self) -> EtherAddr {
        self.src_haddr
    }

    /// Source internet layer [`Ipv4Addr`].
    #[inline]
    pub const fn src_proto_addr(&self) -> Ipv4Addr {
        self.src_paddr
    }

    /// Destination link layer [`EtherAddr`].
    #[inline]
    pub const fn dest_hw_addr(&self) -> EtherAddr {
        self.dest_haddr
    }

    /// Destination internet layer [`Ipv4Addr`].
    #[inline]
    pub const fn dest_proto_addr(&self) -> Ipv4Addr {
        self.dest_paddr
    }
}

impl fmt::Display for Arp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ARP ({}) src: {} ({}), dst: {} ({})",
            self.operation(),
            self.src_proto_addr(),
            self.src_hw_addr(),
            self.dest_proto_addr(),
            self.dest_hw_addr()
        )
    }
}

/// Representation of [`Hardware`] cast from a slice of bytes instead of constructed. It is assumed
/// that [`verify`][HardwareRepr::verify] is called directly after casting before any other
/// methods are called.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
struct HardwareRepr(U16);

impl HardwareRepr {
    const ETHERNET: HardwareRepr = HardwareRepr(U16::new(Hardware::Ethernet as u16));

    /// Check inner self for validity.
    #[inline]
    pub(crate) const fn verify(&self) -> Result<()> {
        match *self {
            Self::ETHERNET => Ok(()),
            _ => Err(Error::Unsupported),
        }
    }

    /// Get the underlying [`Hardware`].
    #[inline]
    pub(crate) const fn get(&self) -> Hardware {
        match *self {
            Self::ETHERNET => Hardware::Ethernet,
            _ => unreachable!(),
        }
    }
}

impl From<Hardware> for HardwareRepr {
    #[inline]
    fn from(value: Hardware) -> Self {
        HardwareRepr(U16::from(value as u16))
    }
}

impl fmt::Display for HardwareRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

/// Representation of [`Operation`] cast from a slice of bytes instead of constructed. It is assumed
/// that [`verify`][OperationRepr::verify] is called directly after casting before any other
/// methods are called.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
struct OperationRepr(U16);

impl OperationRepr {
    const REQUEST: OperationRepr = OperationRepr(U16::new(Operation::Request as u16));
    const REPLY: OperationRepr = OperationRepr(U16::new(Operation::Reply as u16));

    /// Check inner self for validity.
    #[inline]
    pub(crate) const fn verify(&self) -> Result<()> {
        match *self {
            Self::REQUEST | Self::REPLY => Ok(()),
            _ => Err(Error::Unsupported),
        }
    }

    /// Get the underlying [`Operation`].
    #[inline]
    pub(crate) const fn get(&self) -> Operation {
        match *self {
            Self::REQUEST => Operation::Request,
            Self::REPLY => Operation::Reply,
            _ => unreachable!(),
        }
    }
}

impl From<Hardware> for OperationRepr {
    #[inline]
    fn from(value: Hardware) -> Self {
        OperationRepr(U16::from(value as u16))
    }
}

impl fmt::Display for OperationRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 27];
        assert_eq!(Arp::from_bytes(&bytes).unwrap_err(), Error::Truncated);
    }
}

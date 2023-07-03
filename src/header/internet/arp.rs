use core::fmt;

use super::Ipv4Addr;
use crate::header::{
    error::HeaderTruncated,
    link::{EtherAddr, EtherType as Protocol, EtherTypeRepr as ProtocolRepr},
    primitive::{non_exhaustive_enum, U16, U8},
    utils::as_header,
};

non_exhaustive_enum! {
pub enum Hardware(u16) {
    Ethernet = 1,
}
}

non_exhaustive_enum! {
pub enum Operation(u16) {
    Request = 1,
    Reply = 2,
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
/// used to resolve Ethernet addresses from Ipv4 addresses.
impl Arp {
    /// Returns an immutable view of `bytes` as an Arp header followed by a payload or an error if
    /// the size or contents do not represent a valid Arp header.
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8]), HeaderTruncated> {
        as_header!(Arp, bytes)
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

/// Representation of [`Hardware`] cast from a slice of bytes instead of constructed.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
struct HardwareRepr(U16);

impl HardwareRepr {
    /// Get the underlying [`Hardware`].
    #[inline]
    pub(crate) const fn get(&self) -> Hardware {
        Hardware::new(self.0.get())
    }
}

impl From<Hardware> for HardwareRepr {
    #[inline]
    fn from(value: Hardware) -> Self {
        HardwareRepr(U16::from(value.get()))
    }
}

impl fmt::Display for HardwareRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

/// Representation of [`Operation`] cast from a slice of bytes instead of constructed.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
struct OperationRepr(U16);

impl OperationRepr {
    /// Get the underlying [`Operation`].
    #[inline]
    pub(crate) const fn get(&self) -> Operation {
        Operation::new(self.0.get())
    }
}

impl From<Hardware> for OperationRepr {
    #[inline]
    fn from(value: Hardware) -> Self {
        OperationRepr(U16::from(value.get()))
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
        assert_eq!(Arp::from_bytes(&bytes).unwrap_err(), HeaderTruncated);
    }
}

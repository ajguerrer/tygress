//! [`Udp`] header
//!
//! [`Udp`] header supporting connectionless transport of messages and multiplexing by service per
//! node.

use core::fmt;

use crate::header::error::HeaderTruncated;
use crate::header::primitive::U16;
use crate::header::utils::as_header;

/// A UDP header. [Read more][RFC 768]
///
/// UDP is a simple, connectionless transport protocol that allows multiplexing by services
/// provided by a node. Each service is enumerated by port.
///
/// UDP has a checksum for data integrity, but otherwise provides no additional reliability.
///
///  [RFC 768]: https://tools.ietf.org/html/rfc768
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub struct Udp {
    src_port: U16,
    dst_port: U16,
    len: U16,
    checksum: U16,
}

impl Udp {
    #[inline]
    pub const fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8]), HeaderTruncated> {
        as_header!(Udp, bytes)
    }

    // Returns the source port.
    #[inline]
    pub const fn source_port(&self) -> u16 {
        self.src_port.get()
    }

    // Returns the destination port.
    #[inline]
    pub const fn destination_port(&self) -> u16 {
        self.dst_port.get()
    }

    /// Returns the length of the UDP header and payload in bytes. Will return at least 8, the
    /// length of a UDP header.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> u16 {
        self.len.get()
    }

    /// Returns the checksum of the UDP header. If unused, field will carry all zeros.
    #[inline]
    pub(crate) const fn _checksum(&self) -> u16 {
        self.checksum.get()
    }
}

impl fmt::Display for Udp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UDP src_port: {}, dst_port: {}, len: {}",
            self.source_port(),
            self.destination_port(),
            self.len()
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn short_header() {
        let bytes = [0; 7];
        assert_eq!(Udp::from_bytes(&bytes).unwrap_err(), HeaderTruncated);
    }
}

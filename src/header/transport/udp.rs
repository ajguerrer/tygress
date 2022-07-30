//! [`Udp`] header
//!
//! [`Udp`] header supporting connectionless transport of messages and multiplexing by service per
//! node.

use core::fmt;

use crate::header::error::{Error, Result};
use crate::header::macros::as_header;
use crate::header::primitive::U16;

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
    cks: U16,
}

impl Udp {
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<(&Self, &[u8])> {
        let (header, payload) = as_header!(Udp, bytes)?;

        if header.len() < 8 {
            return Err(Error::Truncated);
        }

        if u16::from(header.cks) != 0 {
            // TODO: call verify_checksum on pseudo header and payload
        }

        Ok((header, payload))
    }

    // Returns the source port.
    #[inline]
    pub fn source_port(&self) -> u16 {
        u16::from(self.src_port)
    }

    // Returns the destination port.
    #[inline]
    pub fn destination_port(&self) -> u16 {
        u16::from(self.dst_port)
    }

    /// Returns the length of the UDP header and payload in bytes. Will return at least 8, the
    /// length of a UDP header.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u16 {
        u16::from(self.len)
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
        assert_eq!(Udp::from_bytes(&bytes).unwrap_err(), Error::Truncated);
    }
}

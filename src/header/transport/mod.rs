//! The transport layer of the Internet Protocol suite
//!
//! This layer establishes the basic protocol for transmitting data between hosts without
//! considering the lower level logistics of how it gets there. Assuming an error prone network,
//! decisions about reliability are made in this layer. Beyond IP addressing, this layer also
//! introduces the concept of ports through which a host can offer and access multiple distinct
//! services. For more info, see [RFC 1122].
//!
//! [RFC 1122]: https://tools.ietf.org/html/rfc1122#section-4

mod udp;
pub use udp::*;

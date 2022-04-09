//! The internet layer of the Internet Protocol suite
//!
//! This layer implements protocols used for routing data between hosts communicating on separate
//! networks. As such, internet headers focus primarily on addressing. For more info, see [RFC
//! 1122].
//!
//! [RFC 1122]: https://tools.ietf.org/html/rfc1122#section-3

mod ip;
mod ipv4;

pub use ip::*;
pub use ipv4::*;

//! The link layer of the Internet Protocol suite.
//!
//! This layer implements protocols used to communicate between devices connected locally by a
//! particular hardware medium, either virtual or physical. For more info, see [RFC 1122].
//!
//! [RFC 1122]: https://tools.ietf.org/html/rfc1122#section-2

mod ethernet;
pub use ethernet::*;

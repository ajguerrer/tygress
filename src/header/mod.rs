//! Type-safe views of byte slices as network headers.
//!
//! Network data is more than just a slice of bytes; it has structure and meaning. Though
//! application level data is opaque, it is prefixed by a sequence of communication headers which
//! help move the data through the network stack. Eventually application data will reach an open
//! socket or be discarded. This module enforces the structure and meaning of those prefixed bytes
//! as headers followed by a payload using the type system.
//!
//! The headers in this module are categorized by layers of the Internet protocol suite described in
//! [RFC 1122]. Each header has a `from_bytes` function that interprets a `&[u8]` as an immutable
//! view of a header and it's payload, validating the header along the way. If a header has dynamic
//! fields, then more slices may be returned.
//!  
//! [RFC 1122]: https://tools.ietf.org/html/rfc1122

pub mod internet;
pub mod link;
pub mod transport;

pub(crate) mod checksum;
mod error;
pub(crate) mod primitive;
pub(crate) mod utils;

pub use error::{Error, Result};

//! Type-safe views of byte slices as network headers.
//!
//! Network data is more than just a slice of bytes; it has structure and meaning. Though
//! application level data is opaque, it is prefixed by a sequence of communication headers which
//! help move the data through the network stack to the correct socket. This module enforces the
//! structure and meaning of those prefixed bytes as headers followed by a payload using the type
//! system.
//!
//! Below is a list of types representing headers and header fields. Each header has a
//! `split_header` function that interprets a [`&[u8]`] as an immutable view of a header and it's
//! payload, validating the header along the way. If a header has dynamic fields, then more slices
//! may be returned.

mod ethernet;
mod ip;
mod ipv4;
pub(crate) mod primitive;

pub use ethernet::*;
pub use ip::*;
pub use ipv4::*;
// Each header type must uphold a set of invariants in order to soundly cast between it and a slice
// of bytes. Invariants include:
//
// - Alignment
//
//   While it is sufficient to check if a particular slice is aligned with the minimum alignment of
//   the header type, this macro enforces that the header type be unaligned (minimum alignment  == 1
//   byte) at compile time by panicking.
//
// - Padding
//
//   If the above alignment requirement is met, then an unaligned type, by definition, will never
//   need any padding inserted to meet it's minimum alignment requirements. Therefore, this macro
//   contains no padding check. Otherwise to check for padding, the sum of the sizes of the types
//   fields would recursively need to be compared against the size of the type. Fun stuff.
//
// - Size
//
//   Finally, the length of the byte slice needs to be large enough to completely fill the header
//   type. Because slice length is dynamic, this invariant cannot be checked at compile time, so
//   instead of panicking, an Error is returned.
macro_rules! validate_header {
    ($header:ty, $bytes:ident) => {
        const _: () = if ::core::mem::align_of::<$header>() != 1 {
    panic!("{}", stringify!(align_of<$header> != 1))
        };
        if $bytes.len() < ::core::mem::size_of::<$header>() {
            return Err(Error::Truncated);
        }
    };
}

// Unsafe cast of a immutable slice of bytes to a immutable header type and payload. Before
// performing the cast, the slice and the header type must be checked by `validate_header` to
// soundly perform the cast.
macro_rules! as_header {
    ($header:ty, $bytes:ident) => {{
        crate::header::validate_header!($header, $bytes);

        let (header, payload) = $bytes.split_at(::core::mem::size_of::<$header>());

        // Safety: There are enough $bytes to fill $header and $header meets alignment and padding
        // constraints.
        #[allow(unsafe_code)]
        let header = unsafe { &*(header.as_ptr() as *const Self) };
        Ok((header, payload))
    }};
}

pub(crate) use as_header;
use validate_header;

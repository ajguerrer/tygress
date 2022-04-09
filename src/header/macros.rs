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
        crate::header::macros::validate_header!($header, $bytes);

        let (header, payload) = $bytes.split_at(::core::mem::size_of::<$header>());

        // Safety: There are enough $bytes to fill $header and $header meets alignment and padding
        // constraints.
        #[allow(unsafe_code)]
        let header = unsafe { &*(header.as_ptr() as *const Self) };
        Ok((header, payload))
    }};
}

pub(crate) use as_header;
pub(crate) use validate_header;

use super::error::{Error, Result};

// Subdivides all bytes in header into 16-bit words, and adds them up with ones' complement
// addition. A valid computed checksum equals 0.
#[inline]
pub fn verify_checksum(bytes: &[u8]) -> Result<()> {
    let sum: u32 = bytes
        .chunks_exact(2)
        // chunks_exact(2) always maps to arrays of 2 bytes as a slice so the conversion should
        // never fail.
        .map(|bytes| u32::from(u16::from_be_bytes(bytes.try_into().unwrap())))
        .sum();
    let low = sum as u16;
    let high = (sum >> 16) as u16;
    if !(high + low) == 0 {
        Ok(())
    } else {
        Err(Error::Malformed)
    }
}

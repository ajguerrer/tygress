/// Unsafe cast of a immutable slice of bytes to a immutable header type and payload.
///
/// Each header type must uphold a set of invariants in order to soundly cast between it and a slice
/// of bytes. Invariants include:
///
/// - Alignment
///
///   While it is sufficient to check if a particular slice is aligned with the minimum alignment of
///   the header type, this macro enforces that the header type be unaligned (minimum alignment  == 1
///   byte) at compile time by panicking.
///
/// - Padding
///
///   If the above alignment requirement is met, then an unaligned type, by definition, will never
///   need any padding inserted to meet it's minimum alignment requirements. Therefore, this macro
///   contains no padding check. Otherwise to check for padding, the sum of the sizes of the types
///   fields would recursively need to be compared against the size of the type. Fun stuff.
///
/// - Size
///
///   Finally, the length of the byte slice needs to be large enough to completely fill the header
///   type. Because slice length is dynamic, this invariant cannot be checked at compile time, so
///   instead of panicking, an Error is returned.
macro_rules! as_header {
    ($header:ty, $bytes:ident) => {{
        // check if header type is unaligned at compile time
        const _: () = if ::core::mem::align_of::<$header>() != 1 {
            panic!("{}", stringify!(align_of<$header> != 1))
        };

        // Safety: verify_header makes sure bytes.len() are at least
        // ::core::mem::size_of::<$header>().
        #[allow(unsafe_code)]
        if let Some((header, payload)) = $crate::header::utils::split_at($bytes, ::core::mem::size_of::<$header>()) {
            // Safety: There are enough $bytes to fill $header and $header meets alignment and padding
            // constraints.
            #[allow(unsafe_code)]
            let header = unsafe { &*(header.as_ptr() as *const $header) };
            Ok((header, payload))
        } else {
            Err($crate::header::error::HeaderTruncated)
        }
    }};
}
pub(crate) use as_header;

#[inline]
pub(crate) const fn split_word(slice: &[u8]) -> Option<(u16, &[u8])> {
    if slice.len() < core::mem::size_of::<u16>() {
        None
    } else {
        // Safety: performed length check above
        #[allow(unsafe_code)]
        Some(unsafe { split_word_unsafe(slice) })
    }
}

#[inline]
pub(crate) const fn split_at(slice: &[u8], mid: usize) -> Option<(&[u8], &[u8])> {
    if slice.len() < mid {
        None
    } else {
        // Safety: performed length check above
        #[allow(unsafe_code)]
        Some(unsafe { split_at_unsafe(slice, mid) })
    }
}

// `const` version of `core::slice::from_raw_parts` copied from
// https://github.com/rust-lang/rust/pull/100076.
// Use this until #![feature(const_slice_split_at_not_mut)] is stable.
#[allow(unsafe_code)]
#[inline]
const unsafe fn split_at_unsafe(slice: &[u8], mid: usize) -> (&[u8], &[u8]) {
    use core::slice::from_raw_parts;
    let len = slice.len();
    let ptr = slice.as_ptr();
    (
        from_raw_parts(ptr, mid),
        from_raw_parts(ptr.add(mid), len - mid),
    )
}

// variation of split_at_unsafe, used to compute checksums
#[allow(unsafe_code)]
#[inline]
const unsafe fn split_word_unsafe(slice: &[u8]) -> (u16, &[u8]) {
    let (word, rest) = split_at_unsafe(slice, 2);
    let word = word.as_ptr() as *const [u8; 2];
    (u16::from_ne_bytes(*word), rest)
}

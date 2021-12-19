pub mod arp;
pub mod ethernet;

macro_rules! validate_header {
    ($header:ty, $expected_size:literal, $bytes:ident) => {
        const _: () = if ::core::mem::align_of::<$header>() != 1 {
    panic!("{}", stringify!(align_of<$header> != 1))
        };
        const _: () = if ::core::mem::size_of::<$header>() != $expected_size {
    panic!("{}", stringify!(size_of<$header> != $expected_size))
        };
        if $bytes.len() < ::core::mem::size_of::<$header>() {
            return Err(Error::Length);
        }
    };
}

macro_rules! as_header {
    ($header:ty, $expected_size:literal, $bytes:ident) => {{
        crate::parse::validate_header!($header, $expected_size, $bytes);

        let (header, payload) = $bytes.split_at(::core::mem::size_of::<$header>());

        // Safety: There are enough $bytes to fill $header and $header has no alignment constraints
        // or padding. Assuming $expected_size is correct, this cast is safe.
        #[allow(unsafe_code)]
        let header = unsafe { &*(header.as_ptr() as *const Self) };
        Ok((header, payload))
    }};
}

macro_rules! as_header_mut {
    ($header:ty, $expected_size:literal, $bytes:ident) => {{
        crate::parse::validate_header!($header, $expected_size, $bytes);

        let (header, payload) = $bytes.split_at_mut(::core::mem::size_of::<$header>());

        // Safety: There are enough $bytes to fill $header and $header has no alignment constraints
        // or padding. Assuming $expected_size is correct, this cast is safe.
        #[allow(unsafe_code)]
        let header = unsafe { &mut *(header.as_mut_ptr() as *mut Self) };
        Ok((header, payload))
    }};
}

pub(crate) use as_header;
pub(crate) use as_header_mut;
use validate_header;

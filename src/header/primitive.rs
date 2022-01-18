use core::fmt;

macro_rules! primitive {
    ($type_name:ident, $inner_ty:ty, $num_bytes:literal) => {
        /// An array of network endian bytes with no particular meaning other than representation
        /// of an unsigned integer primitive.
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub(crate) struct $type_name([u8; $num_bytes]);

        impl From<$inner_ty> for $type_name {
            #[inline]
            fn from(value: $inner_ty) -> Self {
                Self(<$inner_ty>::to_be_bytes(value))
            }
        }

        impl From<$type_name> for $inner_ty {
            #[inline]
            fn from(value: $type_name) -> Self {
                <$inner_ty>::from_be_bytes(value.0)
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <$inner_ty>::from_be_bytes(self.0).fmt(f)
            }
        }
    };
}

primitive!(U8, u8, 1);
primitive!(U16, u16, 2);

use core::fmt;

macro_rules! non_exhaustive_enum {
    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident($ty:ty) {
            $(
                $( #[$variant_attr:meta] )*
                $variant:ident = $value:expr
            ),+,
        }
    ) => {
        #[non_exhaustive]
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        $( #[$enum_attr] )*
        pub enum $name {
            $(
                $( #[$variant_attr] )*
                $variant
            ),*,
            Unknown($ty),
        }

        impl $name {
            #[inline]
            pub const fn new(value: $ty) -> Self {
                match value {
                    $( $value => $name::$variant ),*,
                    value => $name::Unknown(value)
                }
            }

            #[inline]
            pub const fn get(&self) -> $ty {
                match self {
                    $( $name::$variant => $value ),*,
                    $name::Unknown(value) => *value
                }
            }
        }

        impl From<$name> for $ty {
            #[inline]
            fn from(value: $name) -> Self {
                value.get()
            }
        }

        impl From<$ty> for $name {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self, f)
            }
        }
    };
}
pub(crate) use non_exhaustive_enum;

macro_rules! primitive {
    ($type_name:ident, $inner_ty:ty, $num_bytes:literal) => {
        /// An array of network endian bytes with no particular meaning other than representation of
        /// an unsigned integer primitive. Use this type if you find yourself calling `to_be_bytes`
        /// or `from_be_bytes`.
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        pub(crate) struct $type_name(pub(crate) [u8; $num_bytes]);

        impl $type_name {
            #[inline]
            pub(crate) const fn new(value: $inner_ty) -> Self {
                Self(<$inner_ty>::to_be_bytes(value))
            }

            #[inline]
            pub(crate) const fn get(&self) -> $inner_ty {
                <$inner_ty>::from_be_bytes(self.0)
            }
        }

        impl From<$inner_ty> for $type_name {
            #[inline]
            fn from(value: $inner_ty) -> Self {
                <$type_name>::new(value)
            }
        }

        impl From<$type_name> for $inner_ty {
            #[inline]
            fn from(value: $type_name) -> Self {
                value.get()
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }
    };
}

primitive!(U8, u8, 1);
primitive!(U16, u16, 2);
primitive!(U32, u32, 4);

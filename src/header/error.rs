use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderTruncated;

impl fmt::Display for HeaderTruncated {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not enough bytes to represent header")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChecksumAssertion;

impl fmt::Display for ChecksumAssertion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "checksum assertion failure")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueToLarge;

impl fmt::Display for ValueToLarge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value to large to represent as bitfield")
    }
}

use super::{error::ChecksumAssertion, utils::split_word};

#[inline]
pub const fn verify_checksum(bytes: &[u8]) -> Result<(), ChecksumAssertion> {
    if compute_checksum(bytes) == !0 {
        Ok(())
    } else {
        Err(ChecksumAssertion)
    }
}

// Subdivides all bytes in header into 16-bit words, and adds them up with ones' complement
// addition. A valid computed checksum equals 0.
#[inline]
pub const fn compute_checksum(mut bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    while let Some((word, rest)) = split_word(bytes) {
        sum += word as u32;
        bytes = rest;
    }

    if let Some(value) = bytes.first() {
        sum += *value as u32
    }

    // carries are added to the sum (twice in case another carry is produced)
    sum = (sum >> 16) + (sum & 0xffff);
    (sum >> 16) as u16 + sum as u16
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_checksum() {
        let bytes = [
            0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0xb1, 0xe6, 0xac, 0x10,
            0x0a, 0x63, 0xac, 0x10, 0x0a, 0x0c,
        ];

        assert_eq!(verify_checksum(&bytes), Ok(()));

        let bytes = [
            0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0, 0xa8,
            0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
        ];

        assert_eq!(verify_checksum(&bytes), Ok(()));
    }

    #[test]
    fn valid_checksum_add_carries_twice() {
        // sum = 0xFFFF_FFFF
        //
        // after first carry, sum = 0x1_FFFE
        //
        // after second carry, sum = 0xFFFF
        //
        // Any larger input would cause a u32 overflow panic and thats fine. Packets should not get
        // this big in the first place.
        let bytes = vec![0xFF; 0x20002];
        assert_eq!(verify_checksum(&bytes), Ok(()));
    }

    #[test]
    fn invalid_checksum() {
        let bytes = [
            0x45, 0x00, 0x00, 0x3d, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0xb1, 0xe6, 0xac, 0x10,
            0x0a, 0x63, 0xac, 0x10, 0x0a, 0x0c,
        ];

        assert_eq!(verify_checksum(&bytes), Err(ChecksumAssertion));

        let bytes = [
            0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x10, 0xb8, 0x61, 0xc0, 0xa8,
            0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
        ];

        assert_eq!(verify_checksum(&bytes), Err(ChecksumAssertion));
    }

    #[test]
    fn invalid_checksum_add_carries_twice() {
        // sum = 0x1_FFFF
        //
        // after first carry, sum = 0x1_0000
        //
        // This test makes sure nothing crashes by overflow
        let bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(verify_checksum(&bytes), Err(ChecksumAssertion));
    }
}

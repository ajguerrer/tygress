use crate::header::primitive::{non_exhaustive_enum, U16};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Ieee802154<'a> {
    // Frame control  1/2
    // Sequence number 0/1

    // Dest PAN ID 0/2
    // Dest Addr 0/2/8

    // Src PAN ID 0/2
    // Src Addr 0/2/8

    // Aux security header var.
    bytes: &'a [u8],
}

/// ```text
///  0 1 2 3 4 5 6 7 8 9 A B C D E F
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |     | | | | | | | |   |   |   |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///    ┆   ┆ ┆ ┆ ┆ ┆ ┆ ┆  ┆   ┆   └ Source Addressing Mode
///    ┆   ┆ ┆ ┆ ┆ ┆ ┆ ┆  ┆   └ Frame Version
///    ┆   ┆ ┆ ┆ ┆ ┆ ┆ ┆  └ Destination Addressing Mode
///    ┆   ┆ ┆ ┆ ┆ ┆ ┆ └ IE Present
///    ┆   ┆ ┆ ┆ ┆ ┆ └ Sequence Number Suppression
///    ┆   ┆ ┆ ┆ ┆ └ Reserved
///    ┆   ┆ ┆ ┆ └ PAN ID Compression
///    ┆   ┆ ┆ └ Acknowledgement Required (AR)
///    ┆   ┆ └ Frame Pending
///    ┆   └ Security Enabled
///    └ Frame Type
/// ```
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct FrameControl(U16);

impl FrameControl {
    const FRAME_TYPE_MASK: u16 = 0b1110_0000_0000_0000;
    const FRAME_TYPE_SHIFT: usize = 13;

    const SECURITY_ENABLED_MASK: u16 = 0b0001_0000_0000_0000;
    const SECURITY_ENABLED_SHIFT: usize = 12;

    const FRAME_PENDING_MASK: u16 = 0b0000_1000_0000_0000;
    const FRAME_PENDING_SHIFT: usize = 11;

    const ACK_REQUIRED_MASK: u16 = 0b0000_0100_0000_0000;
    const ACK_REQUIRED_SHIFT: usize = 10;

    const PAN_ID_COMPRESSED_MASK: u16 = 0b0000_0010_0000_0000;
    const PAN_ID_COMPRESSED_SHIFT: usize = 9;

    const SEQ_NUM_SUPPRESSED_MASK: u16 = 0b0000_0000_1000_0000;
    const SEQ_NUM_SUPPRESSED_SHIFT: usize = 7;

    const IE_PRESENT_MASK: u16 = 0b0000_0000_0100_0000;
    const IE_PRESENT_SHIFT: usize = 6;

    const DEST_ADDRESSING_MODE_MASK: u16 = 0b0000_0000_0011_0000;
    const DEST_ADDRESSING_MODE_SHIFT: usize = 4;

    const FRAME_VERSION_MASK: u16 = 0b0000_0000_0000_1100;
    const FRAME_VERSION_SHIFT: usize = 2;

    const SRC_ADDRESSING_MASK: u16 = 0b0000_0000_0000_0011;
    const SRC_ADDRESSING_SHIFT: usize = 0;

    pub(crate) const fn frame_type(&self) -> FrameType {
        FrameType::new(((self.0.get() & Self::FRAME_TYPE_MASK) >> Self::FRAME_TYPE_SHIFT) as u8)
    }

    pub(crate) const fn security_enabled(&self) -> bool {
        ((self.0.get() & Self::SECURITY_ENABLED_MASK) >> Self::SECURITY_ENABLED_SHIFT) != 0
    }

    pub(crate) const fn frame_pending(&self) -> bool {
        ((self.0.get() & Self::FRAME_PENDING_MASK) >> Self::FRAME_PENDING_SHIFT) != 0
    }

    pub(crate) const fn ack_required(&self) -> bool {
        ((self.0.get() & Self::ACK_REQUIRED_MASK) >> Self::ACK_REQUIRED_SHIFT) != 0
    }

    pub(crate) const fn pan_id_compressed(&self) -> bool {
        ((self.0.get() & Self::PAN_ID_COMPRESSED_MASK) >> Self::PAN_ID_COMPRESSED_SHIFT) != 0
    }

    pub(crate) const fn seq_num_suppressed(&self) -> bool {
        ((self.0.get() & Self::SEQ_NUM_SUPPRESSED_MASK) >> Self::SEQ_NUM_SUPPRESSED_SHIFT) != 0
    }

    pub(crate) const fn ie_present(&self) -> bool {
        ((self.0.get() & Self::IE_PRESENT_MASK) >> Self::IE_PRESENT_SHIFT) != 0
    }

    pub(crate) const fn dest_addressing_mode(&self) -> AddressingMode {
        AddressingMode::new(
            ((self.0.get() & Self::DEST_ADDRESSING_MODE_MASK) >> Self::DEST_ADDRESSING_MODE_SHIFT)
                as u8,
        )
    }
    pub(crate) const fn frame_version(&self) -> FrameVersion {
        FrameVersion::new(
            ((self.0.get() & Self::FRAME_VERSION_MASK) >> Self::FRAME_VERSION_SHIFT) as u8,
        )
    }
    pub(crate) const fn src_addressing_mode(&self) -> AddressingMode {
        AddressingMode::new(
            ((self.0.get() & Self::SRC_ADDRESSING_MASK) >> Self::SRC_ADDRESSING_SHIFT) as u8,
        )
    }
}

non_exhaustive_enum! {
pub enum FrameType(u8) {
    Beacon = 0b000,
    Data = 0b001,
    Ack = 0b010,
    MacCmd = 0b011,
    Multi = 0b101,
    FragOrFrak = 0b110,
    Extended = 0b111,
}
}

non_exhaustive_enum! {
pub enum AddressingMode(u8) {
    Omitted = 0b00,
    Short = 0b10,
    Extended = 0b11,
}
}

non_exhaustive_enum! {
pub enum FrameVersion(u8) {
    Ieee802154_2003 = 0b00,
    Ieee802154_2006 = 0b01,
    Ieee802154 = 0b10,
}
}

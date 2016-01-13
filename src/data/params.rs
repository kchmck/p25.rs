//! Defines various parameters used for data packets.

use util;

pub trait PacketParams {
    /// Number of data bytes in a normal block.
    fn block_bytes() -> usize;

    /// Number of data bytes in the tail block.
    fn tail_bytes() -> usize;

    /// Maximum number of blocks in the packet, including the tail block.
    fn max_blocks() -> usize { 127 }

    /// Maximum number of data bytes in the packet.
    fn packet_bytes() -> usize {
        (Self::max_blocks() - 1) * Self::block_bytes() + Self::tail_bytes()
    }

    /// Calculate the total number of data blocks (normal and tail) needed to hold the
    /// given amount of bytes.
    fn blocks(bytes: usize) -> usize {
        util::div_ceil(bytes + Self::pads(bytes), Self::block_bytes())
    }

    /// Calculate the number of pads needed for the tail block (and possibly
    /// second-to-last block) for the given amount of bytes.
    fn pads(bytes: usize) -> usize {
        (Self::block_bytes() - bytes % Self::block_bytes() + Self::tail_bytes()) %
            Self::block_bytes()
    }

    /// Calculate the number of normal data blocks (tail block not included) needed to
    /// hold the given amount of bytes.
    fn full_blocks(bytes: usize) -> usize {
        Self::blocks(bytes) - 1
    }
}

/// Params for confirmed data packets.
pub struct ConfirmedParams;

impl PacketParams for ConfirmedParams {
    fn block_bytes() -> usize { 16 }
    fn tail_bytes() -> usize { 12 }
}

/// Params for unconfirmed data packets.
pub struct UnconfirmedParams;

impl PacketParams for UnconfirmedParams {
    fn block_bytes() -> usize { 12 }
    fn tail_bytes() -> usize { 8 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calcs() {
        assert_eq!(ConfirmedParams::blocks(0), 1);
        assert_eq!(ConfirmedParams::blocks(6), 1);
        assert_eq!(ConfirmedParams::blocks(12), 1);
        assert_eq!(ConfirmedParams::blocks(13), 2);
        assert_eq!(ConfirmedParams::blocks(16), 2);
        assert_eq!(ConfirmedParams::blocks(28), 2);
        assert_eq!(ConfirmedParams::blocks(29), 3);
        assert_eq!(ConfirmedParams::blocks(2028), 127);

        assert_eq!(ConfirmedParams::pads(0), 12);
        assert_eq!(ConfirmedParams::pads(1), 11);
        assert_eq!(ConfirmedParams::pads(2), 10);
        assert_eq!(ConfirmedParams::pads(5), 7);
        assert_eq!(ConfirmedParams::pads(10), 2);
        assert_eq!(ConfirmedParams::pads(11), 1);
        assert_eq!(ConfirmedParams::pads(12), 0);
        assert_eq!(ConfirmedParams::pads(13), 15);
        assert_eq!(ConfirmedParams::pads(14), 14);
        assert_eq!(ConfirmedParams::pads(15), 13);

        assert_eq!(UnconfirmedParams::blocks(0), 1);
        assert_eq!(UnconfirmedParams::blocks(8), 1);
        assert_eq!(UnconfirmedParams::blocks(9), 2);
        assert_eq!(UnconfirmedParams::blocks(20), 2);
        assert_eq!(UnconfirmedParams::blocks(21), 3);
        assert_eq!(UnconfirmedParams::blocks(1520), 127);

        assert_eq!(UnconfirmedParams::pads(0), 8);
        assert_eq!(UnconfirmedParams::pads(1), 7);
        assert_eq!(UnconfirmedParams::pads(2), 6);
        assert_eq!(UnconfirmedParams::pads(3), 5);
        assert_eq!(UnconfirmedParams::pads(4), 4);
        assert_eq!(UnconfirmedParams::pads(5), 3);
        assert_eq!(UnconfirmedParams::pads(6), 2);
        assert_eq!(UnconfirmedParams::pads(7), 1);
        assert_eq!(UnconfirmedParams::pads(8), 0);
        assert_eq!(UnconfirmedParams::pads(9), 11);
        assert_eq!(UnconfirmedParams::pads(10), 10);
        assert_eq!(UnconfirmedParams::pads(11), 9);

        assert_eq!(ConfirmedParams::packet_bytes(), 2028);
        assert_eq!(UnconfirmedParams::packet_bytes(), 1520);
    }
}

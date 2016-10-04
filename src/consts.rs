/// Symbols (dibits) per second.
pub const SYMBOL_RATE: usize = 4800;
/// Baseband samples per second
pub const SAMPLE_RATE: usize = 48000;
/// Baseband samples per symbol.
pub const SYMBOL_PERIOD: usize = SAMPLE_RATE / SYMBOL_RATE;
/// Number of symbols in the frame sync sequence.
pub const SYNC_SYMBOLS: usize = 24;
/// Number of dibits in a coded NID word.
pub const NID_DIBITS: usize = 32;
/// Number of dibits that are input to the 1/2 or 3/4-rate trellis coder.
pub const CODING_DIBITS: usize = 98;
/// Number of dibits in an uncoded TSBK packet.
pub const TSBK_DIBITS: usize = 48;
/// Number of bytes in an uncoded TSBK packet.
pub const TSBK_BYTES: usize = TSBK_DIBITS / 4;
/// Number of dibits in a coded voice frame.
pub const FRAME_DIBITS: usize = 72;
/// Number of hexbits in a coded voice header packet.
pub const HEADER_HEXBITS: usize = 36;
/// Number of bytes in an uncoded voice header packet.
pub const HEADER_BYTES: usize = 15;
/// Number of hexbits in a coded voice extra packet.
pub const EXTRA_HEXBITS: usize = 24;
/// Number of bytes in a link control word.
pub const LINK_CONTROL_BYTES: usize = 9;
/// Number of bytes in a crypto control word.
pub const CRYPTO_CONTROL_BYTES: usize = 12;
/// Number of dibits in an LC/CC piece. An LC/CC word is spread over 6 equal-sized pieces
/// in each frame group, for a total of 120 dibits.
pub const EXTRA_PIECE_DIBITS: usize = 20;
/// Number of dibits in each coded word that makes up a voice extra component.
pub const EXTRA_WORD_DIBITS: usize = 5;
/// Number of dibits in the voice data fragment.
pub const DATA_FRAG_DIBITS: usize = 8;
/// Number of dibits in each coded word that makes up the voice header packet.
pub const HEADER_WORD_DIBITS: usize = 9;
/// Number of dibits in each coded word that makes up the voice LC terminator packet.
pub const LC_TERM_WORD_DIBITS: usize = 12;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_params() {
        // Don't support non-integer period.
        assert!(SAMPLE_RATE % SYMBOL_RATE == 0);
    }
}

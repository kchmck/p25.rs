/// Number of dibits in a coded frame.
pub const FRAME_DIBITS: usize = 72;
/// Number of hexbits in a coded header packet.
pub const HEADER_HEXBITS: usize = 36;
/// Number of bytes in an uncoded header packet.
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

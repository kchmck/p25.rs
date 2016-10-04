/// Symbols (dibits) per second.
pub const SYMBOL_RATE: usize = 4800;
/// Baseband samples per second
pub const SAMPLE_RATE: usize = 48000;
/// Baseband samples per symbol.
pub const SYMBOL_PERIOD: usize = SAMPLE_RATE / SYMBOL_RATE;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_params() {
        // Don't support non-integer period.
        assert!(SAMPLE_RATE % SYMBOL_RATE == 0);
    }
}

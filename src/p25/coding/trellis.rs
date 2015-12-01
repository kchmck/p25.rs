//! Implements the "trellis" convolutional error correcting code specified by P25.

use std;

use bits;

/// Half-rate convolutional ("trellis") code state machine.
pub type DibitFSM = TrellisFSM<DibitStates>;

/// 3/4-rate convolutional ("trellis") code state machine.
pub type TribitFSM = TrellisFSM<TribitStates>;

pub trait States {
    /// Symbol type to use for states and input.
    type Symbol;

    /// Number of rows/columns in the state machine.
    fn size() -> usize;

    /// Get the "constallation point" on the transition from the current state to the next
    /// state.
    fn pair_idx(cur: usize, next: usize) -> usize;

    /// Convert the given symbol to a state.
    fn state(input: Self::Symbol) -> usize;
    /// Convert the given state to a symbol.
    fn symbol(state: usize) -> Self::Symbol;

    /// Get the "flushing" symbol fed in at the end of a stream.
    fn finisher() -> Self::Symbol;

    /// Get the dibit pair on the transition from the current state to the next state.
    fn pair(state: usize, next: usize) -> (bits::Dibit, bits::Dibit) {
        const PAIRS: [(u8, u8); 16] = [
            (0b00, 0b10),
            (0b10, 0b10),
            (0b01, 0b11),
            (0b11, 0b11),
            (0b11, 0b10),
            (0b01, 0b10),
            (0b10, 0b11),
            (0b00, 0b11),
            (0b11, 0b01),
            (0b01, 0b01),
            (0b10, 0b00),
            (0b00, 0b00),
            (0b00, 0b01),
            (0b10, 0b01),
            (0b01, 0b00),
            (0b11, 0b00),
        ];

        let (hi, lo) = PAIRS[Self::pair_idx(state, next)];
        (bits::Dibit::new(hi), bits::Dibit::new(lo))
    }
}

/// Half-rate state machine (dibit input).
pub struct DibitStates;

impl States for DibitStates {
    type Symbol = bits::Dibit;

    fn size() -> usize { 4 }

    fn pair_idx(cur: usize, next: usize) -> usize {
        const STATES: [[usize; 4]; 4] = [
            [0, 15, 12, 3],
            [4, 11, 8, 7],
            [13, 2, 1, 14],
            [9, 6, 5, 10],
        ];

        STATES[cur][next]
    }

    fn state(input: bits::Dibit) -> usize { input.bits() as usize }
    fn finisher() -> Self::Symbol { bits::Dibit::new(0b00) }
    fn symbol(state: usize) -> Self::Symbol { bits::Dibit::new(state as u8) }
}

/// 3/4-rate state machine (tribit input).
pub struct TribitStates;

impl States for TribitStates {
    type Symbol = bits::Tribit;

    fn size() -> usize { 8 }

    fn pair_idx(cur: usize, next: usize) -> usize {
        const STATES: [[usize; 8]; 8] = [
            [0,  8, 4, 12, 2, 10, 6, 14],
            [4, 12, 2, 10, 6, 14, 0,  8],
            [1,  9, 5, 13, 3, 11, 7, 15],
            [5, 13, 3, 11, 7, 15, 1,  9],
            [3, 11, 7, 15, 1,  9, 5, 13],
            [7, 15, 1,  9, 5, 13, 3, 11],
            [2, 10, 6, 14, 0,  8, 4, 12],
            [6, 14, 0,  8, 4, 12, 2, 10],
        ];

        STATES[cur][next]
    }

    fn state(input: bits::Tribit) -> usize { input.bits() as usize }
    fn finisher() -> Self::Symbol { bits::Tribit::new(0b000) }
    fn symbol(state: usize) -> Self::Symbol { bits::Tribit::new(state as u8) }
}

/// Convolutional code finite state machine with the given transition table. Each fed-in
/// symbol is used as the next state.
pub struct TrellisFSM<S: States> {
    states: std::marker::PhantomData<S>,
    /// Current state.
    state: usize,
}

impl<S: States> TrellisFSM<S> {
    /// Construct a new `TrellisFSM` at the initial state.
    pub fn new() -> TrellisFSM<S> {
        TrellisFSM {
            states: std::marker::PhantomData,
            state: 0,
        }
    }

    /// Apply the given symbol to the state machine and return the dibit pair on the
    /// transition.
    pub fn feed(&mut self, input: S::Symbol) -> (bits::Dibit, bits::Dibit) {
        let next = S::state(input);
        let pair = S::pair(self.state, next);

        self.state = next;

        pair
    }

    /// Flush the state machine with the finishing symbol and return the final transition.
    pub fn finish(&mut self) -> (bits::Dibit, bits::Dibit) {
        self.feed(S::finisher())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bits::*;

    #[test]
    fn test_dibit_code() {
        let mut fsm = DibitFSM::new();
        assert_eq!(fsm.feed(Dibit::new(0b00)), (Dibit::new(0b00), Dibit::new(0b10)));
        assert_eq!(fsm.feed(Dibit::new(0b00)), (Dibit::new(0b00), Dibit::new(0b10)));
        assert_eq!(fsm.feed(Dibit::new(0b01)), (Dibit::new(0b11), Dibit::new(0b00)));
        assert_eq!(fsm.feed(Dibit::new(0b01)), (Dibit::new(0b00), Dibit::new(0b00)));
        assert_eq!(fsm.feed(Dibit::new(0b10)), (Dibit::new(0b11), Dibit::new(0b01)));
        assert_eq!(fsm.feed(Dibit::new(0b10)), (Dibit::new(0b10), Dibit::new(0b10)));
        assert_eq!(fsm.feed(Dibit::new(0b11)), (Dibit::new(0b01), Dibit::new(0b00)));
        assert_eq!(fsm.feed(Dibit::new(0b11)), (Dibit::new(0b10), Dibit::new(0b00)));
    }

    #[test]
    fn test_tribit_code() {
        let mut fsm = TribitFSM::new();
        assert_eq!(fsm.feed(Tribit::new(0b000)), (Dibit::new(0b00), Dibit::new(0b10)));
        assert_eq!(fsm.feed(Tribit::new(0b000)), (Dibit::new(0b00), Dibit::new(0b10)));
        assert_eq!(fsm.feed(Tribit::new(0b001)), (Dibit::new(0b11), Dibit::new(0b01)));
        assert_eq!(fsm.feed(Tribit::new(0b010)), (Dibit::new(0b01), Dibit::new(0b11)));
        assert_eq!(fsm.feed(Tribit::new(0b100)), (Dibit::new(0b11), Dibit::new(0b11)));
        assert_eq!(fsm.feed(Tribit::new(0b101)), (Dibit::new(0b01), Dibit::new(0b01)));
        assert_eq!(fsm.feed(Tribit::new(0b110)), (Dibit::new(0b11), Dibit::new(0b11)));
        assert_eq!(fsm.feed(Tribit::new(0b111)), (Dibit::new(0b00), Dibit::new(0b01)));
        assert_eq!(fsm.feed(Tribit::new(0b000)), (Dibit::new(0b10), Dibit::new(0b11)));
        assert_eq!(fsm.feed(Tribit::new(0b111)), (Dibit::new(0b01), Dibit::new(0b00)));
    }
}

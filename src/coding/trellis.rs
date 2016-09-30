//! Implements encoding and decoding of the "trellis" convolutional error correcting code
//! specified by P25. Encoding is done with a state machine and decoding is done with the
//! Viterbi algorithm, adapted from \[1].
//!
//! \[1]: "Coding Theory and Cryptography: The Essentials", 2nd ed, Hankerson, Hoffman, et
//! al, 2000

use std;
use std::ops::{Deref, DerefMut};

use collect_slice::CollectSlice;

use bits;

use self::Decision::*;

/// Half-rate convolutional ("trellis") code state machine.
pub type DibitFSM = TrellisFSM<DibitStates>;

/// 3/4-rate convolutional ("trellis") code state machine.
pub type TribitFSM = TrellisFSM<TribitStates>;

/// Half-rate convolution ("trellis") code decoder.
pub type DibitDecoder<T> = ViterbiDecoder<DibitStates, DibitHistory, DibitWalks, T>;

/// 3/4-rate convolution ("trellis") code decoder.
pub type TribitDecoder<T> = ViterbiDecoder<TribitStates, TribitHistory, TribitWalks, T>;

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

pub trait WalkHistory: Copy + Clone + Default +
    Deref<Target = [Option<usize>]> + DerefMut
{
    /// The length of each walk associated with each state. This also determines the delay
    /// before the first decoded symbol is yielded.
    fn history() -> usize;
}

macro_rules! history_type {
    ($name: ident, $history: expr) => {
        #[derive(Copy, Clone, Default)]
        pub struct $name([Option<usize>; $history]);

        impl Deref for $name {
            type Target = [Option<usize>];
            fn deref<'a>(&'a self) -> &'a Self::Target { &self.0[..] }
        }

        impl DerefMut for $name {
            fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target { &mut self.0[..] }
        }

        impl WalkHistory for $name {
            fn history() -> usize { $history }
        }
    };
}

history_type!(DibitHistory, 4);
history_type!(TribitHistory, 4);

pub trait Walks<H: WalkHistory>: Copy + Clone + Default +
    Deref<Target = [Walk<H>]> + DerefMut
{
    fn states() -> usize;
}

macro_rules! impl_walks {
    ($name:ident, $hist:ident, $states:expr) => {
        #[derive(Copy, Clone)]
        pub struct $name([Walk<$hist>; $states]);

        impl Deref for $name {
            type Target = [Walk<$hist>];
            fn deref<'a>(&'a self) -> &'a Self::Target { &self.0[..] }
        }

        impl DerefMut for $name {
            fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target { &mut self.0[..] }
        }

        impl Walks<$hist> for $name {
            fn states() -> usize { $states }
        }

        impl Default for $name {
            fn default() -> Self {
                let mut walks = [Walk::default(); $states];

                (0..Self::states())
                    .map(Walk::new)
                    .collect_slice_checked(&mut walks[..]);

                $name(walks)
            }
        }
    };
}

impl_walks!(DibitWalks, DibitHistory, 4);
impl_walks!(TribitWalks, TribitHistory, 8);

/// Decodes a received convolutional code dibit stream to a nearby codeword using the
/// truncated Viterbi algorithm.
pub struct ViterbiDecoder<S, H, W, T> where
    S: States, H: WalkHistory, W: Walks<H>, T: Iterator<Item = bits::Dibit>
{
    states: std::marker::PhantomData<S>,
    history: std::marker::PhantomData<H>,
    /// Source of dibits.
    src: T,
    /// Walks associated with each state, for the current and previous tick.
    cur: usize,
    prev: usize,
    walks: [W; 2],
    /// Remaining symbols to yield.
    remain: usize,
}

impl<S, H, W, T> ViterbiDecoder<S, H, W, T> where
    S: States, H: WalkHistory, W: Walks<H>, T: Iterator<Item = bits::Dibit>
{
    /// Construct a new `ViterbiDecoder` over the given dibit source.
    pub fn new(src: T) -> ViterbiDecoder<S, H, W, T> {
        debug_assert!(S::size() == W::states());

        ViterbiDecoder {
            states: std::marker::PhantomData,
            history: std::marker::PhantomData,
            src: src,
            walks: [W::default(); 2],
            cur: 1,
            prev: 0,
            remain: 0,
        }.prime()
    }

    fn prime(mut self) -> Self {
        for _ in 1..H::history() {
            self.step();
        }

        self
    }

    fn switch_walk(&mut self) {
        std::mem::swap(&mut self.cur, &mut self.prev);
    }

    fn step(&mut self) -> bool {
        let input = Edge::new(match (self.src.next(), self.src.next()) {
            (Some(hi), Some(lo)) => (hi, lo),
            (Some(_), None) | (None, Some(_)) => panic!("dibits ended on boundary"),
            (None, None) => return false,
        });

        self.remain += 1;
        self.switch_walk();

        for s in 0..S::size() {
            let (walk, _) = self.search(s, input);
            self.walks[self.cur][s].append(walk);
        }

        true
    }

    ///
    fn search(&self, state: usize, input: Edge) -> (Walk<H>, bool) {
        self.walks[self.prev].iter()
            .enumerate()
            .map(|(i, w)| (Edge::new(S::pair(i, state)), w))
            .fold((Walk::default(), false), |(walk, amb), (e, w)| {
                match w.distance.checked_add(input.distance(e)) {
                    Some(sum) if sum < walk.distance => (walk.replace(&w, sum), false),
                    Some(sum) if sum == walk.distance => (walk.combine(&w, sum), true),
                    _ => (walk, amb),
                }
            })
    }

    ///
    fn decode(&self) -> Decision {
        self.walks[self.cur].iter().fold(Ambiguous(std::usize::MAX), |s, w| {
            match s {
                Ambiguous(min) | Definite(min, _) if w.distance < min =>
                    Definite(w.distance, w[self.remain]),
                Definite(min, state) if w.distance == min && w[self.remain] != state =>
                    Ambiguous(w.distance),
                _ => s,
            }
        })
    }
}

impl<S, H, W, T> Iterator for ViterbiDecoder<S, H, W, T> where
    S: States, H: WalkHistory, W: Walks<H>, T: Iterator<Item = bits::Dibit>
{
    type Item = Result<S::Symbol, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.step() && self.remain == 0 {
            return None;
        }

        self.remain -= 1;

        Some(match self.decode() {
            Ambiguous(_) | Definite(_, None) => Err(()),
            Definite(_, Some(state)) => Ok(S::symbol(state)),
        })
    }
}

/// Decoding decision.
enum Decision {
    Definite(usize, Option<usize>),
    Ambiguous(usize),
}

#[derive(Copy, Clone, Debug)]
pub struct Walk<H: WalkHistory>{
    history: H,
    pub distance: usize,
}

impl<H: WalkHistory> Walk<H> {
    pub fn new(state: usize) -> Walk<H> {
        Walk {
            history: H::default(),
            distance: if state == 0 {
                0
            } else {
                std::usize::MAX
            },
       }.init(state)
    }

    fn init(mut self, state: usize) -> Self {
        self.history[0] = Some(state);
        self
    }

    pub fn append(&mut self, other: Self) {
        self.distance = other.distance;
        other.iter().cloned().collect_slice(&mut self[1..]);
    }

    pub fn combine(mut self, other: &Self, distance: usize) -> Self {
        self.distance = distance;

        for (dest, src) in self.iter_mut().zip(other.iter()) {
            if src != dest {
                *dest = None;
            }
        }

        self
    }

    pub fn replace(mut self, other: &Self, distance: usize) -> Self {
        self.distance = distance;
        other.iter().cloned().collect_slice_checked(&mut self[..]);

        self
    }
}

impl<H: WalkHistory> Deref for Walk<H> {
    type Target = [Option<usize>];
    fn deref<'a>(&'a self) -> &'a Self::Target { &self.history }
}

impl<H: WalkHistory> DerefMut for Walk<H> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target { &mut self.history }
}

impl<H: WalkHistory> Default for Walk<H> {
    fn default() -> Self { Walk::new(std::usize::MAX) }
}

#[derive(Copy, Clone)]
struct Edge(u8);

impl Edge {
    pub fn new((hi, lo): (bits::Dibit, bits::Dibit)) -> Edge {
        Edge(hi.bits() << 2 | lo.bits())
    }

    pub fn distance(&self, other: Edge) -> usize {
        (self.0 ^ other.0).count_ones() as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{Edge};
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

    #[test]
    fn test_edge() {
        assert_eq!(Edge::new((
            Dibit::new(0b11), Dibit::new(0b01)
        )).distance(Edge::new((
            Dibit::new(0b11), Dibit::new(0b01)
        ))), 0);

        assert_eq!(Edge::new((
            Dibit::new(0b11), Dibit::new(0b01)
        )).distance(Edge::new((
            Dibit::new(0b00), Dibit::new(0b10)
        ))), 4);
    }

    #[test]
    fn test_dibit_decoder() {
        let bits = [1, 2, 2, 2, 2, 1, 3, 3, 0, 2];
        let stream = bits.iter().map(|&bits| Dibit::new(bits));

        let mut dibits = vec![];
        let mut fsm = DibitFSM::new();

        for dibit in stream {
            let (hi, lo) = fsm.feed(dibit);
            dibits.push(hi);
            dibits.push(lo);
        }

        dibits[2] = Dibit::new(0b10);
        dibits[4] = Dibit::new(0b10);

        let mut dec = DibitDecoder::new(dibits.iter().cloned());

        assert_eq!(dec.next().unwrap().unwrap().bits(), 1);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 1);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 3);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 3);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 0);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
    }

    #[test]
    fn test_tribit_decoder() {
        let bits = [
            1, 2, 3, 4, 5, 6, 7, 0,
            1, 2, 3, 4, 5, 6, 7, 0,
        ];
        let stream = bits.iter().map(|&bits| Tribit::new(bits));

        let mut dibits = vec![];
        let mut fsm = TribitFSM::new();

        for tribit in stream {
            let (hi, lo) = fsm.feed(tribit);
            dibits.push(hi);
            dibits.push(lo);
        }

        dibits[6] = Dibit::new(0b10);
        dibits[4] = Dibit::new(0b10);
        dibits[14] = Dibit::new(0b10);

        let mut dec = TribitDecoder::new(dibits.iter().cloned());

        assert_eq!(dec.next().unwrap().unwrap().bits(), 1);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 3);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 4);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 5);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 6);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 7);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 0);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 1);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 2);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 3);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 4);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 5);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 6);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 7);
        assert_eq!(dec.next().unwrap().unwrap().bits(), 0);
    }
}

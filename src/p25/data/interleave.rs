//! Interleaving and deinterleaving for data packet payloads.

use std;
use bits;

/// Interleaves a dibit buffer with an iterator interface.
pub type Interleaver = Redirect<InterleaveRedirector>;

/// Deinterleaves a dibit buffer with an iterator interface.
pub type Deinterleaver = Redirect<DeinterleaveRedirector>;

trait Redirector {
    /// Redirector the given index to another within the buffer.
    fn redirect(idx: usize) -> usize;
}

/// Redirects for interleaving.
struct InterleaveRedirector;

impl Redirector for InterleaveRedirector {
    fn redirect(idx: usize) -> usize {
        const REDIRECTS: [usize; 98] = [
            0,
            1,
            8,
            9,
            16,
            17,
            24,
            25,
            32,
            33,
            40,
            41,
            48,
            49,
            56,
            57,
            64,
            65,
            72,
            73,
            80,
            81,
            88,
            89,
            96,
            97,
            2,
            3,
            10,
            11,
            18,
            19,
            26,
            27,
            34,
            35,
            42,
            43,
            50,
            51,
            58,
            59,
            66,
            67,
            74,
            75,
            82,
            83,
            90,
            91,
            4,
            5,
            12,
            13,
            20,
            21,
            28,
            29,
            36,
            37,
            44,
            45,
            52,
            53,
            60,
            61,
            68,
            69,
            76,
            77,
            84,
            85,
            92,
            93,
            6,
            7,
            14,
            15,
            22,
            23,
            30,
            31,
            38,
            39,
            46,
            47,
            54,
            55,
            62,
            63,
            70,
            71,
            78,
            79,
            86,
            87,
            94,
            95,
        ];

        REDIRECTS[idx]
    }
}

/// Redirects to undo interleaving.
struct DeinterleaveRedirector;

impl Redirector for DeinterleaveRedirector {
    fn redirect(idx: usize) -> usize {
        const REDIRECTS: [usize; 98] = [
            0,
            1,
            26,
            27,
            50,
            51,
            74,
            75,
            2,
            3,
            28,
            29,
            52,
            53,
            76,
            77,
            4,
            5,
            30,
            31,
            54,
            55,
            78,
            79,
            6,
            7,
            32,
            33,
            56,
            57,
            80,
            81,
            8,
            9,
            34,
            35,
            58,
            59,
            82,
            83,
            10,
            11,
            36,
            37,
            60,
            61,
            84,
            85,
            12,
            13,
            38,
            39,
            62,
            63,
            86,
            87,
            14,
            15,
            40,
            41,
            64,
            65,
            88,
            89,
            16,
            17,
            42,
            43,
            66,
            67,
            90,
            91,
            18,
            19,
            44,
            45,
            68,
            69,
            92,
            93,
            20,
            21,
            46,
            47,
            70,
            71,
            94,
            95,
            22,
            23,
            48,
            49,
            72,
            73,
            96,
            97,
            24,
            25,
        ];


        REDIRECTS[idx]
    }
}

/// Wraps a buffer, redirecting sequential indexes with the given redirector.
struct Redirect<T: Redirector> {
    redirector: std::marker::PhantomData<T>,
    /// Wrapped buffer.
    dibits: [bits::Dibit; 98],
    /// Index into `dibits`.
    pos: std::ops::Range<usize>,
}

impl<T: Redirector> Redirect<T> {
    /// Construct a new `Redirect` over the given buffer.
    pub fn new(dibits: [bits::Dibit; 98]) -> Redirect<T> {
        Redirect {
            redirector: std::marker::PhantomData,
            dibits: dibits,
            pos: 0..98,
        }
    }
}

impl<T: Redirector> Iterator for Redirect<T> {
    type Item = bits::Dibit;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos.next() {
            Some(idx) => Some(self.dibits[T::redirect(idx)]),
            None => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bits::*;

    #[test]
    fn test_interleave() {
        let mut buf = [Dibit::default(); 98];

        for i in 0..98 {
            buf[i] = Dibit::new(i as u8 % 4);
        }

        let mut int = Interleaver::new(buf);

        for _ in 0..13 {
            assert_eq!(int.next().unwrap().bits(), 0b00);
            assert_eq!(int.next().unwrap().bits(), 0b01);
        }

        for _ in 0..12 {
            assert_eq!(int.next().unwrap().bits(), 0b10);
            assert_eq!(int.next().unwrap().bits(), 0b11);
        }

        for _ in 0..12 {
            assert_eq!(int.next().unwrap().bits(), 0b00);
            assert_eq!(int.next().unwrap().bits(), 0b01);
        }

        for _ in 0..12 {
            assert_eq!(int.next().unwrap().bits(), 0b10);
            assert_eq!(int.next().unwrap().bits(), 0b11);
        }

        assert!(int.next().is_none());
    }

    #[test]
    fn test_deinterleave() {
        let mut buf = [Dibit::default(); 98];

        for i in 0..98 {
            buf[i] = Dibit::new(i as u8 % 4);
        }

        let mut out = [Dibit::default(); 98];

        for (i, dibit) in Interleaver::new(buf).enumerate() {
            out[i] = dibit;
        }

        let mut deint = Deinterleaver::new(out);

        for _ in 0..24 {
            assert_eq!(deint.next().unwrap().bits(), 0b00);
            assert_eq!(deint.next().unwrap().bits(), 0b01);
            assert_eq!(deint.next().unwrap().bits(), 0b10);
            assert_eq!(deint.next().unwrap().bits(), 0b11);
        }

        assert_eq!(deint.next().unwrap().bits(), 0b00);
        assert_eq!(deint.next().unwrap().bits(), 0b01);

        assert!(deint.next().is_none());
    }
}

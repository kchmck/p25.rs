//! Provides `Fragments` iterator for splitting a data slice into fragment slices, whose
//! size depends on the data packet used.
//!
//! A data message is made up of unlimited data fragments, with the FSNF field of the
//! packet header determining how the message is reconstructed from the fragments.

use std;

use data::params::*;

/// Fragments for confirmed data packets.
pub type ConfirmedFragments<'a> = Fragments<'a, ConfirmedParams>;

/// Fragments for unconfirmed data packets.
pub type UnconfirmedFragments<'a> = Fragments<'a, UnconfirmedParams>;

/// Iterator over data fragments, yielding fragment-sized data slices to be turned into
/// payload blocks.
pub struct Fragments<'a, P: PacketParams> {
    params: std::marker::PhantomData<P>,
    /// data to be split.
    data: &'a [u8],
    /// Current byte index into `data`.
    pos: usize,
}

impl<'a, P: PacketParams> Fragments<'a, P> {
    /// Construct a new `Fragments` over the given data.
    pub fn new(data: &'a [u8]) -> Fragments<'a, P> {
        Fragments {
            params: std::marker::PhantomData,
            data: data,
            pos: 0,
        }
    }
}

impl<'a, P: PacketParams> Iterator for Fragments<'a, P> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            return None;
        }

        let start = self.pos;
        let stop = std::cmp::min(start + P::packet_bytes(), self.data.len());

        self.pos = stop;

        Some(&self.data[start..stop])
    }
}

#[cfg(test)]
mod test {
    use super::Fragments;
    use data::params::*;

    #[test]
    fn test_fragments() {
        struct TestParams;

        impl PacketParams for TestParams {
            fn packet_bytes() -> usize { 2 }
            fn block_bytes() -> usize{ 0 }
            fn tail_bytes() -> usize { 0 }
        }

        let bytes = [1];
        let mut f = Fragments::<TestParams>::new(&bytes);
        assert_eq!(f.next().unwrap(), &[1]);
        assert!(f.next().is_none());

        let bytes = [1, 2, 3, 4];
        let mut f = Fragments::<TestParams>::new(&bytes);
        assert_eq!(f.next().unwrap(), &[1, 2]);
        assert_eq!(f.next().unwrap(), &[3, 4]);
        assert!(f.next().is_none());

        let bytes = [1, 2, 3, 4, 5];
        let mut f = Fragments::<TestParams>::new(&bytes);
        assert_eq!(f.next().unwrap(), &[1, 2]);
        assert_eq!(f.next().unwrap(), &[3, 4]);
        assert_eq!(f.next().unwrap(), &[5]);
        assert!(f.next().is_none());
    }
}

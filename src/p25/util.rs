use std;
use std::num::One;

/// Calculate ceil(a / b).
pub fn div_ceil<T>(a: T, b: T) -> T where
    T: std::ops::Add<T, Output = T> + std::ops::Sub<T, Output = T> +
       std::ops::Div<T, Output = T> + std::num::One + Copy
{
    (a + b - T::one()) / b
}

pub trait CollectSlice: Iterator {
    /// Loop through the iterator, writing items into the given slice until either the
    /// iterator runs out or the slice fills up. Return the number of items written.
    fn collect_slice(&mut self, slice: &mut [Self::Item]) -> usize;

    /// Perform `collect_slice()` and panic if there weren't enough elements to fill up
    /// the buffer or the buffer was too small to hold all the elements.
    fn collect_slice_checked(&mut self, slice: &mut [Self::Item]);
}

impl<T, I: Iterator<Item = T>> CollectSlice for I {
    fn collect_slice(&mut self, slice: &mut [Self::Item]) -> usize {
        slice.iter_mut().zip(self).fold(0, |count, (s, i)| {
            *s = i;
            count + 1
        })
    }

    fn collect_slice_checked(&mut self, slice: &mut [Self::Item]) {
        assert_eq!(self.collect_slice(slice), slice.len());
        assert!(self.next().is_none());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_div_ceil() {
        assert_eq!(div_ceil(13, 12), 2);
        assert_eq!(div_ceil(1, 2), 1);
        assert_eq!(div_ceil(0, 3), 0);
    }

    #[test]
    fn test_collect_slice() {
        {
            let mut buf = [0; 5];

            (0..5).map(|i| {
                i + 1
            }).collect_slice(&mut buf[..]);

            assert_eq!(buf[0], 1);
            assert_eq!(buf[1], 2);
            assert_eq!(buf[2], 3);
            assert_eq!(buf[3], 4);
            assert_eq!(buf[4], 5);
        }

        {
            let mut buf = [0; 5];

            let mut iter = (0..3).map(|i| {
                i + 1
            });

            iter.collect_slice(&mut buf[2..]);

            assert_eq!(buf[0], 0);
            assert_eq!(buf[1], 0);
            assert_eq!(buf[2], 1);
            assert_eq!(buf[3], 2);
            assert_eq!(buf[4], 3);
        }

        {
            let mut buf = [0; 3];

            let mut iter = (0..5).map(|i| {
                i + 1
            });

            iter.collect_slice(&mut buf[..]);

            assert_eq!(buf[0], 1);
            assert_eq!(buf[1], 2);
            assert_eq!(buf[2], 3);

            assert_eq!(iter.next().unwrap(), 4);
            assert_eq!(iter.next().unwrap(), 5);
        }
    }

    #[test]
    #[should_panic]
    fn test_collect_slice_checked_small() {
        let mut buf = [0; 5];

        (0..3).map(|i| {
            i + 1
        }).collect_slice_checked(&mut buf[..]);
    }

    #[test]
    #[should_panic]
    fn test_collect_slice_checked_big() {
        let mut buf = [0; 3];

        (0..5).map(|i| {
            i + 1
        }).collect_slice_checked(&mut buf[..]);
    }
}

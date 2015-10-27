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
    /// iterator runs out or the slice fills up.
    fn collect_slice(&mut self, slice: &mut [Self::Item]);
}

impl<T, I: Iterator<Item = T>> CollectSlice for I {
    fn collect_slice(&mut self, slice: &mut [Self::Item]) {
        for (s, i) in slice.iter_mut().zip(self) {
            *s = i;
        }
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

            (0..3).map(|i| {
                i + 1
            }).collect_slice(&mut buf[..]);

            assert_eq!(buf[0], 1);
            assert_eq!(buf[1], 2);
            assert_eq!(buf[2], 3);
            assert_eq!(buf[3], 0);
            assert_eq!(buf[4], 0);
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
}

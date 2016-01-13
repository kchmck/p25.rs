use std;
use std::num::One;

/// Calculate ceil(a / b).
pub fn div_ceil<T>(a: T, b: T) -> T where
    T: std::ops::Add<T, Output = T> + std::ops::Sub<T, Output = T> +
       std::ops::Div<T, Output = T> + std::num::One + Copy
{
    (a + b - T::one()) / b
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
}

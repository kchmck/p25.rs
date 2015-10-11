/// Multiply the given word by the given matrix, starting from an empty word.
macro_rules! matrix_mul {
    ($word: expr, $mat:expr, $output:ty) => {
        accum_rows!(0, $word, $mat, $output)
    };
}

/// Multiply the given word by the given matrix and "prime" the output word with the input
/// word, as if the first rows of the matrix were the identity matrix.
macro_rules! matrix_mul_systematic {
    ($word: expr, $mat:expr, $output:ty) => {
        accum_rows!($word as $output, $word, $mat, $output)
    };
}

/// Generate an output word by "multiplying" the given word vector by the given matrix.
/// Each bit in the output word is formed by multiplying, in GF(2), the input vector by
/// the corresponding row in the matrix and "summing" the terms in GF(2).
macro_rules! accum_rows {
    ($init:expr, $word:expr, $mat:expr, $output:ty ) => {
        $mat.iter().fold($init, |accum, row| {
            accum << 1 | (($word & row).count_ones() % 2) as $output
        })
    };
}

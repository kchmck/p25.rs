//! Encoding and decoding of the (24, 12, 13) short, (24, 16, 9) medium, and (36, 20, 17)
//! long Reed-Solomon codes described by P25.
//!
//! These algorithms are sourced from \[1].
//!
//! \[1]: "Coding Theory and Cryptography: The Essentials", 2nd ed, Hankerson, Hoffman, et
//! al, 2000

use std;

use bits::Hexbit;
use bmcf;
use galois::{P25Codeword, Polynomial, PolynomialCoefs};
use util::CollectSlice;

/// Encoding and decoding of the (24, 12, 13) code.
pub mod short {
    use bits::Hexbit;

    /// Transpose of G_LC.
    const GEN: [[u8; 12]; 12] = [
        [0o62, 0o11, 0o03, 0o21, 0o30, 0o01, 0o61, 0o24, 0o72, 0o72, 0o73, 0o71],
        [0o44, 0o12, 0o01, 0o70, 0o22, 0o41, 0o76, 0o22, 0o42, 0o14, 0o65, 0o05],
        [0o03, 0o11, 0o05, 0o27, 0o03, 0o27, 0o21, 0o71, 0o05, 0o65, 0o36, 0o55],
        [0o25, 0o11, 0o75, 0o45, 0o75, 0o56, 0o55, 0o56, 0o20, 0o54, 0o61, 0o03],
        [0o14, 0o16, 0o14, 0o16, 0o15, 0o76, 0o76, 0o21, 0o43, 0o35, 0o42, 0o71],
        [0o16, 0o64, 0o06, 0o67, 0o15, 0o64, 0o01, 0o35, 0o47, 0o25, 0o22, 0o34],
        [0o27, 0o67, 0o20, 0o23, 0o33, 0o21, 0o63, 0o73, 0o33, 0o41, 0o17, 0o60],
        [0o03, 0o55, 0o44, 0o64, 0o15, 0o53, 0o35, 0o42, 0o56, 0o16, 0o04, 0o11],
        [0o53, 0o01, 0o66, 0o73, 0o51, 0o04, 0o30, 0o57, 0o01, 0o15, 0o44, 0o74],
        [0o04, 0o76, 0o06, 0o33, 0o03, 0o25, 0o13, 0o74, 0o16, 0o40, 0o20, 0o02],
        [0o36, 0o26, 0o70, 0o44, 0o53, 0o01, 0o64, 0o43, 0o13, 0o71, 0o25, 0o41],
        [0o47, 0o73, 0o66, 0o21, 0o50, 0o12, 0o70, 0o76, 0o76, 0o26, 0o05, 0o50],
    ];

    /// Calculate the 12 parity hexbits for the 12 data hexbits.
    pub fn encode(buf: &mut [Hexbit; 24]) {
        let (data, parity) = buf.split_at_mut(12);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]));
    }

    /// Try to decode the 24-hexbit word to the nearest codeword, correcting up to 6
    /// errors. Return `Some((data, err))`, where `data` are the 12 data hexbits and `err`
    /// is the number of errors corrected, on success and `None` on an unrecoverable
    /// error.
    pub fn decode(buf: &mut [Hexbit; 24]) -> Option<(&[Hexbit], usize)> {
        let (poly, err) = match super::decode::<super::ShortCoefs>(buf) {
            Some(x) => x,
            None => return None,
        };

        Some((super::copy_data(poly, &mut buf[..12]), err))
    }
}

/// Encoding and decoding of the (24, 16, 9) code.
pub mod medium {
    use bits::Hexbit;

    /// Transpose of G_ES.
    const GEN: [[u8; 16]; 8] = [
        [0o51, 0o57, 0o05, 0o73, 0o75, 0o20, 0o02, 0o24, 0o42, 0o32, 0o65, 0o64, 0o62, 0o55, 0o24, 0o67],
        [0o45, 0o25, 0o01, 0o07, 0o15, 0o32, 0o75, 0o74, 0o64, 0o32, 0o36, 0o06, 0o63, 0o43, 0o23, 0o75],
        [0o67, 0o63, 0o31, 0o47, 0o51, 0o14, 0o43, 0o15, 0o07, 0o55, 0o25, 0o54, 0o74, 0o34, 0o23, 0o45],
        [0o15, 0o73, 0o04, 0o14, 0o51, 0o42, 0o05, 0o72, 0o22, 0o41, 0o07, 0o32, 0o70, 0o71, 0o05, 0o60],
        [0o64, 0o71, 0o16, 0o41, 0o17, 0o75, 0o01, 0o24, 0o61, 0o57, 0o50, 0o76, 0o05, 0o57, 0o50, 0o57],
        [0o67, 0o22, 0o54, 0o77, 0o67, 0o42, 0o40, 0o26, 0o20, 0o66, 0o16, 0o46, 0o27, 0o76, 0o70, 0o24],
        [0o52, 0o40, 0o25, 0o47, 0o17, 0o70, 0o12, 0o74, 0o40, 0o21, 0o40, 0o14, 0o37, 0o50, 0o42, 0o06],
        [0o12, 0o15, 0o76, 0o11, 0o57, 0o54, 0o64, 0o61, 0o65, 0o77, 0o51, 0o36, 0o46, 0o64, 0o23, 0o26],
    ];

    /// Calculate the 8 parity hexbits for the 16 data hexbits.
    pub fn encode(buf: &mut [Hexbit; 24]) {
        let (data, parity) = buf.split_at_mut(16);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]));
    }

    /// Try to decode the 24-hexbit word to the nearest codeword, correcting up to 4
    /// errors. Return `Some((data, err))`, where `data` are the 16 data hexbits and `err`
    /// is the number of errors corrected, on success and `None` on an unrecoverable
    /// error.
    pub fn decode(buf: &mut [Hexbit; 24]) -> Option<(&[Hexbit], usize)> {
        let (poly, err) = match super::decode::<super::MedCoefs>(buf) {
            Some(x) => x,
            None => return None,
        };

        Some((super::copy_data(poly, &mut buf[..16]), err))
    }
}

/// Encoding and decoding of the (36, 20, 17) code.
pub mod long {
    use bits::Hexbit;

    /// Transpose of P_HDR.
    const GEN: [[u8; 20]; 16] = [
        [0o74, 0o04, 0o07, 0o26, 0o23, 0o24, 0o52, 0o55, 0o54, 0o74, 0o54, 0o51, 0o01, 0o11, 0o06, 0o34, 0o63, 0o71, 0o02, 0o34],
        [0o37, 0o17, 0o23, 0o05, 0o73, 0o51, 0o33, 0o62, 0o51, 0o41, 0o70, 0o07, 0o65, 0o70, 0o02, 0o31, 0o43, 0o21, 0o01, 0o35],
        [0o34, 0o50, 0o37, 0o07, 0o73, 0o25, 0o14, 0o56, 0o32, 0o30, 0o11, 0o72, 0o32, 0o05, 0o65, 0o01, 0o25, 0o70, 0o53, 0o02],
        [0o06, 0o24, 0o46, 0o63, 0o41, 0o23, 0o02, 0o25, 0o65, 0o41, 0o03, 0o30, 0o70, 0o10, 0o11, 0o15, 0o44, 0o44, 0o74, 0o23],
        [0o02, 0o11, 0o56, 0o63, 0o72, 0o22, 0o20, 0o73, 0o77, 0o43, 0o13, 0o65, 0o13, 0o65, 0o41, 0o44, 0o77, 0o56, 0o02, 0o21],
        [0o07, 0o05, 0o75, 0o27, 0o34, 0o41, 0o06, 0o60, 0o12, 0o22, 0o22, 0o54, 0o44, 0o24, 0o20, 0o64, 0o63, 0o04, 0o14, 0o27],
        [0o44, 0o30, 0o43, 0o63, 0o21, 0o74, 0o14, 0o15, 0o54, 0o51, 0o16, 0o06, 0o73, 0o15, 0o45, 0o16, 0o17, 0o30, 0o52, 0o22],
        [0o64, 0o57, 0o45, 0o40, 0o51, 0o66, 0o25, 0o30, 0o13, 0o06, 0o57, 0o21, 0o24, 0o77, 0o42, 0o24, 0o17, 0o74, 0o74, 0o33],
        [0o26, 0o33, 0o55, 0o06, 0o67, 0o74, 0o52, 0o13, 0o35, 0o64, 0o03, 0o36, 0o12, 0o22, 0o46, 0o52, 0o64, 0o04, 0o12, 0o64],
        [0o14, 0o03, 0o21, 0o04, 0o16, 0o65, 0o23, 0o17, 0o32, 0o33, 0o45, 0o63, 0o52, 0o24, 0o54, 0o16, 0o14, 0o23, 0o57, 0o42],
        [0o26, 0o02, 0o50, 0o40, 0o31, 0o70, 0o35, 0o20, 0o56, 0o03, 0o72, 0o50, 0o21, 0o24, 0o35, 0o06, 0o40, 0o71, 0o24, 0o05],
        [0o44, 0o02, 0o31, 0o45, 0o74, 0o36, 0o74, 0o02, 0o12, 0o47, 0o31, 0o61, 0o55, 0o74, 0o12, 0o62, 0o74, 0o70, 0o63, 0o73],
        [0o54, 0o15, 0o45, 0o47, 0o11, 0o67, 0o75, 0o70, 0o75, 0o27, 0o30, 0o64, 0o12, 0o07, 0o40, 0o20, 0o31, 0o63, 0o15, 0o51],
        [0o13, 0o16, 0o27, 0o30, 0o21, 0o45, 0o75, 0o55, 0o01, 0o12, 0o56, 0o52, 0o35, 0o44, 0o64, 0o13, 0o72, 0o45, 0o42, 0o46],
        [0o77, 0o25, 0o71, 0o75, 0o12, 0o64, 0o43, 0o14, 0o72, 0o55, 0o35, 0o01, 0o14, 0o07, 0o65, 0o55, 0o54, 0o56, 0o52, 0o73],
        [0o05, 0o26, 0o62, 0o07, 0o21, 0o01, 0o27, 0o47, 0o63, 0o47, 0o22, 0o60, 0o72, 0o46, 0o33, 0o57, 0o06, 0o43, 0o33, 0o60],
    ];

    /// Calculate the 16 parity hexbits for the 20 data hexbits.
    pub fn encode(buf: &mut [Hexbit; 36]) {
        let (data, parity) = buf.split_at_mut(20);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]))
    }

    /// Try to decode the 36-hexbit word to the nearest codeword, correcting up to 8
    /// errors. Return `Some((data, err))`, where `data` are the 20 data hexbits and `err`
    /// is the number of errors corrected, on success and `None` on an unrecoverable
    /// error.
    pub fn decode(buf: &mut [Hexbit; 36]) -> Option<(&[Hexbit], usize)> {
        let (poly, err) = match super::decode::<super::LongCoefs>(buf) {
            Some(x) => x,
            None => return None,
        };

        Some((super::copy_data(poly, &mut buf[..20]), err))
    }
}

/// Encode the given data with the given generator matrix and place the resulting parity
/// symbols in the given destination.
fn encode<'g, G>(data: &[Hexbit], parity: &mut [Hexbit], gen: G) where
    G: Iterator<Item = &'g [u8]>
{
    for (p, row) in parity.iter_mut().zip(gen) {
        *p = Hexbit::new(
            row.iter()
                .zip(data.iter())
                .fold(P25Codeword::default(), |s, (&col, &d)| {
                    s + P25Codeword::new(d.bits()) * P25Codeword::new(col)
                }).bits()
        );
    }
}

/// Try to fix any errors in the given word.
fn decode<P: PolynomialCoefs>(word: &[Hexbit]) -> Option<(Polynomial<P>, usize)> {
    // In a received hexbit word, the least most significant hexbit symbol (the first data
    // symbol) maps to the highest degree.
    let mut poly = Polynomial::<P>::new(word.iter().rev().map(|&b| {
        P25Codeword::new(b.bits())
    }));

    let syn = syndromes(&poly);
    let dec = bmcf::BerlMasseyDecoder::new(syn).decode();
    let errors = dec.degree().expect("invalid error polynomial");

    let fixed = bmcf::Errors::new(dec, syn)
        .take(errors)
        .fold(0, |count, (loc, val)| {
            poly[loc] = poly[loc] + val;
            count + 1
        });

    if fixed == errors {
        Some((poly, fixed))
    } else {
        None
    }
}

/// Calculate the syndrome polynomial for the given word.
fn syndromes<P: PolynomialCoefs>(word: &Polynomial<P>) -> Polynomial<P> {
    Polynomial::new(std::iter::once(P25Codeword::for_power(0))
        .chain((1..P::distance()).map(|pow| {
            word.eval(P25Codeword::for_power(pow))
        }))
    )
}

/// Copy the data symbols in the given polynomial to the destination as bytes.
fn copy_data<P: PolynomialCoefs>(poly: Polynomial<P>, data: &mut [Hexbit]) -> &[Hexbit] {
    poly.iter().rev().map(|coef| Hexbit::new(coef.bits())).collect_slice(data);
    data
}

/// Polynomial coefficients for the short code.
#[derive(Copy, Clone, Debug, Default)]
struct ShortCoefs([P25Codeword; 24]);

impl PolynomialCoefs for ShortCoefs {
    fn distance() -> usize { 13 }
}

impl std::ops::Deref for ShortCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl std::ops::DerefMut for ShortCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

/// Polynomial coefficients for the medium code.
#[derive(Copy, Clone, Debug, Default)]
struct MedCoefs([P25Codeword; 24]);

impl PolynomialCoefs for MedCoefs {
    fn distance() -> usize { 9 }
}

impl std::ops::Deref for MedCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl std::ops::DerefMut for MedCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

/// Polynomial coefficients for the long code.
#[derive(Copy)]
struct LongCoefs([P25Codeword; 36]);

impl PolynomialCoefs for LongCoefs {
    fn distance() -> usize { 17 }
}

impl std::fmt::Debug for LongCoefs {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", &self.0[..])
    }
}

impl Clone for LongCoefs {
    fn clone(&self) -> Self {
        let mut coefs = [P25Codeword::default(); 36];
        self.0.iter().cloned().collect_slice(&mut coefs[..]);
        LongCoefs(coefs)
    }
}

impl Default for LongCoefs {
    fn default() -> LongCoefs { LongCoefs([P25Codeword::default(); 36]) }
}

impl std::ops::Deref for LongCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl std::ops::DerefMut for LongCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{MedCoefs, ShortCoefs, LongCoefs};
    use galois::{PolynomialCoefs};
    use bits::Hexbit;
    use util::CollectSlice;

    #[test]
    fn validate_coefs() {
        ShortCoefs::default().validate();
        MedCoefs::default().validate();
        LongCoefs::default().validate();
    }

    #[test]
    fn test_decode_short() {
        let mut buf = [Hexbit::default(); 24];
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].iter()
             .map(|&b| Hexbit::new(b)).collect_slice(&mut buf[..]);

        short::encode(&mut buf);

        buf[0] = Hexbit::new(0o00);
        buf[2] = Hexbit::new(0o60);
        buf[7] = Hexbit::new(0o42);
        buf[13] = Hexbit::new(0o14);
        buf[18] = Hexbit::new(0o56);
        buf[23] = Hexbit::new(0o72);

        let dec = short::decode(&mut buf);
        let exp = [
           Hexbit::new(1),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
        ];

        assert_eq!(dec, Some((&exp[..], 6)));
    }

    #[test]
    fn test_decode_med() {
        let mut buf = [Hexbit::default(); 24];
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].iter()
             .map(|&b| Hexbit::new(b)).collect_slice(&mut buf[..]);

        medium::encode(&mut buf);

        buf[0] = Hexbit::new(0o00);
        buf[10] = Hexbit::new(0o60);
        buf[16] = Hexbit::new(0o42);
        buf[23] = Hexbit::new(0o14);

        let dec = medium::decode(&mut buf);
        let exp = [
           Hexbit::new(1),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
        ];

        assert_eq!(dec, Some((&exp[..], 4)));
    }

    #[test]
    fn test_decode_long() {
        let mut buf = [Hexbit::default(); 36];
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].iter()
            .map(|&b| Hexbit::new(b)).collect_slice(&mut buf[..]);

        long::encode(&mut buf);

        buf[0] = Hexbit::new(0o00);
        buf[2] = Hexbit::new(0o43);
        buf[5] = Hexbit::new(0o21);
        buf[10] = Hexbit::new(0o11);
        buf[18] = Hexbit::new(0o67);
        buf[22] = Hexbit::new(0o04);
        buf[27] = Hexbit::new(0o12);
        buf[30] = Hexbit::new(0o32);

        let dec = long::decode(&mut buf);
        let exp = [
           Hexbit::new(1),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
           Hexbit::new(0),
        ];

        assert_eq!(dec, Some((&exp[..], 8)));
    }
}

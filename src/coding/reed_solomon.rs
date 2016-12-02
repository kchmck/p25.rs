//! Encoding and decoding of the (24, 12, 13) short, (24, 16, 9) medium, and (36, 20, 17)
//! long Reed-Solomon codes described by P25.
//!
//! These algorithms are sourced from *Coding Theory and Cryptography: The Essentials*,
//! Hankerson, Hoffman, et al, 2000.

use std;
use std::ops::{Deref, DerefMut};

use collect_slice::CollectSlice;

use bits::Hexbit;
use coding::bmcf;
use coding::galois::{P25Codeword, Polynomial, PolynomialCoefs};

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

    /// Calculate the 12 parity hexbits for the first 12 data hexbits in the given buffer,
    /// placing the parity hexbits at the end of the buffer.
    pub fn encode(buf: &mut [Hexbit; 24]) {
        let (data, parity) = buf.split_at_mut(12);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]));
    }

    /// Try to decode the given 24-hexbit word to the nearest codeword, correcting up to 6
    /// hexbit errors (up to 36 bit errors.)
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 12
    /// data hexbits and `err` is the number of corrected hexbits. Otherwise, return
    /// `None` to indicate an unrecoverable error.
    pub fn decode(buf: &mut [Hexbit; 24]) -> Option<(&[Hexbit], usize)> {
        super::decode::<super::ShortCoefs>(buf).map(move |(poly, err)| {
            (super::extract_data(poly, &mut buf[..12]), err)
        })
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

    /// Calculate the 8 parity hexbits for the first 16 data hexbits in the given buffer,
    /// placing the parity hexbits at the end of the buffer.
    pub fn encode(buf: &mut [Hexbit; 24]) {
        let (data, parity) = buf.split_at_mut(16);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]));
    }

    /// Try to decode the given 24-hexbit word to the nearest codeword, correcting up to 4
    /// hexbit errors (up to 24 bit errors.)
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 16
    /// data hexbits and `err` is the number of corrected hexbits. Otherwise, return
    /// `None` to indicate an unrecoverable error.
    pub fn decode(buf: &mut [Hexbit; 24]) -> Option<(&[Hexbit], usize)> {
        super::decode::<super::MedCoefs>(buf).map(move |(poly, err)| {
            (super::extract_data(poly, &mut buf[..16]), err)
        })
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

    /// Calculate the 16 parity hexbits for the first 20 data hexbits in the given buffer,
    /// placing the parity hexbits at the end of the buffer.
    pub fn encode(buf: &mut [Hexbit; 36]) {
        let (data, parity) = buf.split_at_mut(20);
        super::encode(data, parity, GEN.iter().map(|r| &r[..]))
    }

    /// Try to decode the given 36-hexbit word to the nearest codeword, correcting up to 8
    /// hexbit errors (up to 48 bit errors.)
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 20
    /// data hexbits and `err` is the number of corrected hexbits. Otherwise, return
    /// `None` to indicate an unrecoverable error.
    pub fn decode(buf: &mut [Hexbit; 36]) -> Option<(&[Hexbit], usize)> {
        super::decode::<super::LongCoefs>(buf).map(move |(poly, err)| {
            (super::extract_data(poly, &mut buf[..20]), err)
        })
    }
}

/// Encode the given data with the given generator matrix and place the resulting parity
/// symbols in the given destination.
fn encode<'g, G>(data: &[Hexbit], parity: &mut [Hexbit], gen: G)
    where G: Iterator<Item = &'g [u8]>
{
    gen.map(|row| {
        row.iter()
           .zip(data.iter())
           .fold(P25Codeword::default(), |s, (&col, &d)| {
               s + P25Codeword::new(d.bits()) * P25Codeword::new(col)
           }).bits()
    }).map(Hexbit::new).collect_slice_checked(parity);
}

/// Try to fix any errors in the given word.
///
/// On success, return `Some((poly, err))`, where `poly` is the polynomial representation
/// of the corrected word (with the last data symbol as the degree-0 coefficient) and
/// `err` is the number of corrected hexbit symbols. Otherwise, return `None` to indicate
/// an unrecoverable error.
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
            match poly.get_mut(loc) {
                Some(coef) => *coef = *coef + val,
                None => {},
            }

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
    Polynomial::new((1...P::syndromes()).map(|pow| {
        word.eval(P25Codeword::for_power(pow))
    }))
}

/// Extract the data symbols from the given word in polynomial form and write them to the
/// given slice.
fn extract_data<P>(poly: Polynomial<P>, data: &mut [Hexbit]) -> &[Hexbit]
    where P: PolynomialCoefs
{
    poly.iter().rev().map(|coef| Hexbit::new(coef.bits())).collect_slice_fill(data);
    data
}

/// Polynomial coefficients for the short code.
#[derive(Copy, Clone, Default)]
struct ShortCoefs([P25Codeword; 24]);

impl PolynomialCoefs for ShortCoefs {
    fn distance() -> usize { 13 }
}

impl Deref for ShortCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl DerefMut for ShortCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

/// Polynomial coefficients for the medium code.
#[derive(Copy, Clone, Default)]
struct MedCoefs([P25Codeword; 24]);

impl PolynomialCoefs for MedCoefs {
    fn distance() -> usize { 9 }
}

impl Deref for MedCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl DerefMut for MedCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

/// Polynomial coefficients for the long code.
#[derive(Copy)]
struct LongCoefs([P25Codeword; 36]);

impl PolynomialCoefs for LongCoefs {
    fn distance() -> usize { 17 }
}

impl Clone for LongCoefs {
    fn clone(&self) -> Self {
        let mut coefs = [P25Codeword::default(); 36];
        coefs.copy_from_slice(&self.0[..]);
        LongCoefs(coefs)
    }
}

impl Default for LongCoefs {
    fn default() -> LongCoefs { LongCoefs([P25Codeword::default(); 36]) }
}

impl Deref for LongCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl DerefMut for LongCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{MedCoefs, ShortCoefs, LongCoefs};
    use coding::galois::{PolynomialCoefs, P25Codeword, Polynomial};
    use bits::Hexbit;
    use collect_slice::CollectSlice;

    #[test]
    fn validate_coefs() {
        ShortCoefs::default().validate();
        MedCoefs::default().validate();
        LongCoefs::default().validate();
    }

    #[test]
    fn verify_short_gen() {
        let p = Polynomial::<ShortCoefs>::new([
            P25Codeword::for_power(1),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(2),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(3),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(4),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(5),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(6),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(7),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(8),
            P25Codeword::for_power(0),
        ].iter().cloned());

        assert_eq!(p.degree().unwrap(), 8);
        assert_eq!(p.coef(0).bits(), 0o26);
        assert_eq!(p.coef(1).bits(), 0o06);
        assert_eq!(p.coef(2).bits(), 0o24);
        assert_eq!(p.coef(3).bits(), 0o57);
        assert_eq!(p.coef(4).bits(), 0o60);
        assert_eq!(p.coef(5).bits(), 0o45);
        assert_eq!(p.coef(6).bits(), 0o75);
        assert_eq!(p.coef(7).bits(), 0o67);
        assert_eq!(p.coef(8).bits(), 0o01);
    }

    #[test]
    fn verify_med_gen() {
        let p = Polynomial::<MedCoefs>::new([
            P25Codeword::for_power(1),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(2),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(3),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(4),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(5),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(6),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(7),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(8),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(9),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(10),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(11),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(12),
            P25Codeword::for_power(0),
        ].iter().cloned());

        assert_eq!(p.degree().unwrap(), 12);
        assert_eq!(p.coef(0).bits(), 0o50);
        assert_eq!(p.coef(1).bits(), 0o41);
        assert_eq!(p.coef(2).bits(), 0o02);
        assert_eq!(p.coef(3).bits(), 0o74);
        assert_eq!(p.coef(4).bits(), 0o11);
        assert_eq!(p.coef(5).bits(), 0o60);
        assert_eq!(p.coef(6).bits(), 0o34);
        assert_eq!(p.coef(7).bits(), 0o71);
        assert_eq!(p.coef(8).bits(), 0o03);
        assert_eq!(p.coef(9).bits(), 0o55);
        assert_eq!(p.coef(10).bits(), 0o05);
        assert_eq!(p.coef(11).bits(), 0o71);
        assert_eq!(p.coef(12).bits(), 0o01);
    }

    #[test]
    fn verify_long_gen() {
        let p = Polynomial::<LongCoefs>::new([
            P25Codeword::for_power(1),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(2),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(3),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(4),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(5),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(6),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(7),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(8),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(9),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(10),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(11),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(12),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(13),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(14),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(15),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(16),
            P25Codeword::for_power(0),
        ].iter().cloned());

        assert_eq!(p.degree().unwrap(), 16);
        assert_eq!(p.coef(0).bits(), 0o60);
        assert_eq!(p.coef(1).bits(), 0o73);
        assert_eq!(p.coef(2).bits(), 0o46);
        assert_eq!(p.coef(3).bits(), 0o51);
        assert_eq!(p.coef(4).bits(), 0o73);
        assert_eq!(p.coef(5).bits(), 0o05);
        assert_eq!(p.coef(6).bits(), 0o42);
        assert_eq!(p.coef(7).bits(), 0o64);
        assert_eq!(p.coef(8).bits(), 0o33);
        assert_eq!(p.coef(9).bits(), 0o22);
        assert_eq!(p.coef(10).bits(), 0o27);
        assert_eq!(p.coef(11).bits(), 0o21);
        assert_eq!(p.coef(12).bits(), 0o23);
        assert_eq!(p.coef(13).bits(), 0o02);
        assert_eq!(p.coef(14).bits(), 0o35);
        assert_eq!(p.coef(15).bits(), 0o34);
        assert_eq!(p.coef(16).bits(), 0o01);
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

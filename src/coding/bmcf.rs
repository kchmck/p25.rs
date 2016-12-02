//! Decodes Reed Solomon and BCH codes using the Berlekamp-Massey, Chien Search, and
//! Forney algorithms.
//!
//! # Decoding Procedure
//!
//! The standard [1]-[11] procedure for Reed Solomon/BCH error correction has the
//! following steps:
//!
//! 1. Generate the syndrome polynomial s(x) = s<sub>1</sub> + s<sub>2</sub>x + ··· +
//!    s<sub>2t</sub>x<sup>2t-1</sup>, where s<sub>i</sub> = r(α<sup>i</sup>) using the
//!    received word polynomial r(x).
//! 2. Use s(x) to build the error locator polynomial Λ(x) = (1 + a<sub>1</sub>x) ··· (1 +
//!    a<sub>e</sub>x), where deg(Λ(x)) = e ≤ t is the number of detected errors.
//! 3. Find the roots a<sub>1</sub><sup>-1</sup>, ..., a<sub>E</sub><sup>-1</sup> of Λ(x),
//!    such that Λ(a<sub>i</sub><sup>-1</sup>) = 0. Then for each, if
//!    a<sub>i</sub><sup>-1</sup> = α<sup>k<sub>i</sub></sup>, the error location within
//!    the received word is taken as m<sub>i</sub> in α<sup>m<sub>i</sub></sup> ≡
//!    α<sup>-k<sub>i</sub></sup> (modulo the field).
//! 4. Verify that e = E.
//! 5. Construct the error evaluator polynomial Ω(x) = Λ(x)s(x) mod x<sup>2t</sup> and
//!    compute each error pattern b<sub>i</sub> = Ω(a<sub>i</sub><sup>-1</sup>) /
//!    Λ'(a<sub>i</sub><sup>-1</sup>).
//! 6. For each (m<sub>i</sub>, b<sub>i</sub>) pair, correct the symbol at location
//!    m<sub>i</sub> using the bit pattern b<sub>i</sub>.
//!
//! This module implements steps 2 through 5. The implementation uses several well-known
//! techniques exist to perform these steps relatively efficiently: the Berlekamp-Massey
//! algorithm for step 2, Chien Search for step 3, and the Forney algorithm for step 5.
//!
//! # Berlekamp-Massey Algorithm
//!
//! The Berlekamp-Massey algorithm has many variants [1], [2], [9], [12], [13] mostly with
//! subtle differences. The key observation from Massey's generalization is to view Λ(x)
//! as the "connection polynomial" of a linear feedback shift register (LFSR) that
//! generates the sequence of syndromes s<sub>1</sub>, ..., s<sub>2t</sub>. The algorithm
//! then synthesizes Λ(x) when constructing the corresponding unique shortest LFSR that
//! generates those syndromes.
//!
//! # Chien Search
//!
//! With Λ(x) = Λ<sub>0</sub> + Λ<sub>1</sub>x + Λ<sub>2</sub>x<sup>2</sup> + ··· +
//! Λ<sub>e</sub>x<sup>e</sup> (where Λ<sub>0</sub> = 1.), let P<sub>i</sub> =
//! [p<sub>0</sub>, ..., p<sub>e</sub>], 0 ≤ i < n, which is indexed as P<sub>i</sub>[k],
//! 0 ≤ k ≤ e.
//!
//! Starting with the base case i = 0, let P<sub>0</sub>[k] = Λ<sub>k</sub> so that
//! Λ(α<sup>0</sup>) = Λ(1) = Λ<sub>0</sub> + Λ<sub>1</sub> + ··· + Λ<sub>e</sub> =
//! sum(P<sub>0</sub>).
//!
//! Then for i > 0, let P<sub>i</sub>[k] = P<sub>i-1</sub>[k]⋅α<sup>k</sup> so that
//! Λ(α<sup>i</sup>) = sum(P<sub>i</sub>).
//!
//! # Forney Algorithm
//!
//! The Forney algorithm reduces the problem of computing error patterns to evaluating
//! Ω(x) / Λ'(x), where Ω(x) = s(x)Λ(x) mod x<sup>2t</sup>. This requires no polynomial
//! long division, just a one-time polynomial multiplication and derivative evaluation
//! to create Ω(x), then two polynomial evaluations and one codeword division for each
//! error.

use std;

use collect_slice::CollectSlice;

use coding::galois::{Polynomial, PolynomialCoefs, P25Codeword, P25Field, GaloisField};

/// Finds the error location polynomial Λ(x) from the syndrome polynomial s(x).
///
/// This uses Hankerson et al's version of the Berlekamp-Massey algorithm, with the result
/// being Λ(x) = p<sub>2t</sub>(x) = σ<sub>R</sub>(x).
pub struct ErrorLocator<P: PolynomialCoefs> {
    /// Saved p polynomial: p<sub>zi-1</sub>.
    p_saved: Polynomial<P>,
    /// Previous iteration's p polynomial: p<sub>i-1</sub>.
    p_cur: Polynomial<P>,
    /// Saved q polynomial: q<sub>zi-1</sub>.
    q_saved: Polynomial<P>,
    /// Previous iteration's q polynomial: q<sub>i-1</sub>.
    q_cur: Polynomial<P>,
    /// Degree-related term of saved p polynomial: D<sub>zi-1</sub>
    deg_saved: usize,
    /// Degree-related term of previous p polynomial: D<sub>i-1</sub>.
    deg_cur: usize,
}

impl<P: PolynomialCoefs> ErrorLocator<P> {
    /// Construct a new `ErrorLocator` from the given syndrome polynomial s(x).
    pub fn new(syn: Polynomial<P>) -> ErrorLocator<P> {
        ErrorLocator {
            // Compute 1 + s(x).
            q_saved: Polynomial::new(
                std::iter::once(P25Codeword::for_power(0))
                    .chain(syn.iter().take(P::syndromes()).cloned())
            ),
            q_cur: syn,
            // Compute x^{2t+1}.
            p_saved: Polynomial::unit_power(P::syndromes() + 1),
            // Compute x^{2t}.
            p_cur: Polynomial::unit_power(P::syndromes()),
            deg_saved: 0,
            deg_cur: 1,
        }
    }

    /// Construct the error locator polynomial Λ(x).
    pub fn build(mut self) -> Polynomial<P> {
        for _ in 0..P::syndromes() {
            self.step();
        }

        self.p_cur
    }

    /// Perform one iterative step of the algorithm, updating the state polynomials and
    /// degrees.
    fn step(&mut self) {
        let (save, q, p, d) = if self.q_cur.constant().zero() {
            self.reduce()
        } else {
            self.transform()
        };

        if save {
            self.q_saved = self.q_cur;
            self.p_saved = self.p_cur;
            self.deg_saved = self.deg_cur;
        }

        self.q_cur = q;
        self.p_cur = p;
        self.deg_cur = d;
    }

    /// Shift the polynomials since they have no degree-0 term.
    fn reduce(&mut self) -> (bool, Polynomial<P>, Polynomial<P>, usize) {
        (
            false,
            self.q_cur.shift(),
            self.p_cur.shift(),
            2 + self.deg_cur,
        )
    }

    /// Normalize out the degree-0 terms and shift the polynomials.
    fn transform(&mut self) -> (bool, Polynomial<P>, Polynomial<P>, usize) {
        let mult = self.q_cur.constant() / self.q_saved.constant();

        (
            self.deg_cur >= self.deg_saved,
            (self.q_cur + self.q_saved * mult).shift(),
            (self.p_cur + self.p_saved * mult).shift(),
            2 + std::cmp::min(self.deg_cur, self.deg_saved),
        )
   }
}

/// Finds the roots of the given error locator polynomial Λ(x).
///
/// This performs the standard brute force method, evaluating each Λ(α<sup>i</sup>) for 0
/// ≤ i < 2<sup>r</sup> - 1, with the Chien Search optimization.
pub struct PolynomialRoots<P: PolynomialCoefs> {
    /// Error locator polynomial: Λ(x).
    ///
    /// This field isn't exactly interpreted as a polynomial, more like a list of
    /// coefficient A = [Λ<sub>0</sub>, ..., Λ<sub>e</sub>] such that Λ(α<sup>i</sup>) =
    /// sum(A) for the current power i.
    loc: Polynomial<P>,
    /// Current codeword power the polynomial is being evaluated with.
    pow: std::ops::Range<usize>,
}

impl<P: PolynomialCoefs> PolynomialRoots<P> {
    /// Construct a new `PolynomialRoots` from the given error locator polynomial Λ(x).
    pub fn new(loc: Polynomial<P>) -> Self {
        PolynomialRoots {
            loc: loc,
            pow: 0..P25Field::size(),
        }
    }

    /// Update each term's coefficient to its value when evaluated for the next codeword
    /// power.
    fn update_terms(&mut self) {
        for (pow, term) in self.loc.iter_mut().enumerate() {
            *term = *term * P25Codeword::for_power(pow);
        }
    }

    /// Compute Λ(α<sup>i</sup>), where i is the current power.
    fn eval(&self) -> P25Codeword {
        self.loc.iter().fold(P25Codeword::default(), |sum, &x| sum + x)
    }
}

/// Iterate over all roots α<sup>i</sup> of Λ(x).
impl<P: PolynomialCoefs> Iterator for PolynomialRoots<P> {
    type Item = P25Codeword;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Current codeword power: i in α^i.
            let pow = match self.pow.next() {
                Some(pow) => pow,
                None => return None,
            };

            // Compute Λ(α^i).
            let eval = self.eval();
            // Update to Λ(α^{i+1}).
            self.update_terms();

            // Yield α^i if Λ(α^i) = 0.
            if eval.zero() {
                return Some(P25Codeword::for_power(pow));
            }
        }
    }
}

/// Computes error locations and patterns from the roots of the error locator polynomial
/// Λ(x).
///
/// This uses the Forney algorithm for error pattern evaluation, which avoids polynomial
/// long division.
pub struct ErrorDescriptions<P: PolynomialCoefs> {
    /// Derivative of error locator polynomial: Λ'(x).
    deriv: Polynomial<P>,
    /// Error evaluator polynomial: Ω(x) = Λ(x)s(x) mod x<sup>2t</sup>.
    vals: Polynomial<P>,
}

impl<P: PolynomialCoefs> ErrorDescriptions<P> {
    /// Create a new `ErrorDescriptions` from the given syndrome polynomial s(x) and error
    /// locator polynomial Λ(x).
    pub fn new(syn: Polynomial<P>, loc: Polynomial<P>) -> Self {
        ErrorDescriptions {
            // Compute Λ'(x).
            deriv: loc.deriv(),
            // Compute Λ(x)s(x) mod x^{2t}.
            vals: (loc * syn).truncate(P::syndromes() - 1),
        }
    }

    /// Compute the error location and pattern for the given root
    /// a<sub>i</sub><sup>-1</sup> of Λ(x).
    pub fn for_root(&self, root: P25Codeword) -> (usize, P25Codeword) {
        (
            // If Λ(α^i) = 0, then the error location is m ≡ -i (modulo the field.)
            root.invert().power().unwrap(),
            // Compute Ω(α^i) / Λ'(α^i).
            self.vals.eval(root) / self.deriv.eval(root),
        )
    }
}

/// Decodes and iterates over codeword errors.
pub struct Errors<P: PolynomialCoefs> {
    /// Roots of the error locator polynomial.
    ///
    /// Note that this field isn't interpreted as a polynomial -- the `Polynomial` type
    /// just provides a conveniently sized buffer for root codewords.
    roots: Polynomial<P>,
    /// Computes location and pattern for each error.
    descs: ErrorDescriptions<P>,
    /// Current error being evaluated in iteration.
    pos: std::ops::Range<usize>,
}

impl<P: PolynomialCoefs> Errors<P> {
    /// Create a new `Errors` decoder from the given syndrome polynomial s(x).
    ///
    /// If decoding was sucessful, return `Some((nerr, errs))`, where `nerr` is the number
    /// of detected errors and `errs` is the error iterator. Otherwise, return `None` to
    /// indicate an unrecoverable error.
    pub fn new(syn: Polynomial<P>) -> Option<(usize, Self)> {
        // Compute error locator polynomial Λ(x).
        let loc = ErrorLocator::new(syn).build();
        // If e = deg(Λ), then e ≤ t and e represents the number of detected errors.
        let errors = loc.degree().expect("invalid error polynomial");

        // Find the roots a_i of Λ(x). These are buffered before processing them because
        // if the number of found roots ends up unequal to deg(Λ(x)), all the roots are
        // invalid, and processing them before checking this can cause behavior like
        // divide-by-zero.
        let mut roots = Polynomial::<P>::default();
        let nroots = PolynomialRoots::new(loc).collect_slice_exhaust(&mut roots[..]);

        // If the number of computed roots is different than deg(Λ), then the roots are
        // invalid and the codeword is unrecoverable [1, p3], [2, p48], [3, p22].
        if nroots != errors {
            return None;
        }

        Some((errors, Errors {
            roots: roots,
            descs: ErrorDescriptions::new(syn, loc),
            pos: 0..errors,
        }))
    }
}

/// Iterate over detected errors, yielding the location and pattern of each error.
impl<P: PolynomialCoefs> Iterator for Errors<P> {
    type Item = (usize, P25Codeword);

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|i| self.descs.for_root(self.roots[i]))
    }
}

#[cfg(test)]
mod test {
    use std;
    use collect_slice::CollectSlice;
    use super::*;
    use coding::galois::{P25Codeword, PolynomialCoefs, Polynomial};

    impl_polynomial_coefs!(TestCoefs, 9);
    type TestPolynomial = Polynomial<TestCoefs>;

    #[test]
    fn test_roots() {
        // p(x) = (1+α^42x)(1+α^13x)(1+α^57x)
        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(42),
        ].iter().cloned()) * TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(13),
        ].iter().cloned()) * TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(57),
        ].iter().cloned());

        let mut r = PolynomialRoots::new(p);
        let mut roots = [P25Codeword::default(); 3];
        r.collect_slice_checked(&mut roots[..]);

        assert!(roots.contains(&P25Codeword::for_power(42).invert()));
        assert!(roots.contains(&P25Codeword::for_power(13).invert()));
        assert!(roots.contains(&P25Codeword::for_power(57).invert()));

        let p = TestPolynomial::unit_power(0);

        let mut r = PolynomialRoots::new(p);
        assert!(r.next().is_none());
    }
}

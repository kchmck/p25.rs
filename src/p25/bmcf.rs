use std;

use galois::{Polynomial, PolynomialCoefs, P25Codeword, P25Field, GaloisField};

/// Implements the iterative part of the Berlekamp-Massey algorithm.
pub struct BerlMasseyDecoder<P: PolynomialCoefs> {
    /// Saved p polynomial: p_{z_i-1}.
    p_saved: Polynomial<P>,
    /// Previous iteration's p polynomial: p_{i-1}.
    p_cur: Polynomial<P>,
    /// Saved q polynomial: q_{z_i-1}.
    q_saved: Polynomial<P>,
    /// Previous iteration's q polynomial: q_{i-1}.
    q_cur: Polynomial<P>,
    /// Degree-related term of saved p polynomial: D_{z_i-1}.
    deg_saved: usize,
    /// Degree-related term of previous p polynomial: D_{i-1}.
    deg_cur: usize,
}

impl<P: PolynomialCoefs> BerlMasseyDecoder<P> {
    /// Construct a new `BerlMasseyDecoder` from the given syndrome polynomial.
    pub fn new(syn: Polynomial<P>) -> BerlMasseyDecoder<P> {
        // 2t zeroes followed by a one.
        let p = Polynomial::new((0..P::syndromes()+1).map(|_| P25Codeword::default())
                                    .chain(std::iter::once(P25Codeword::for_power(0))));

        BerlMasseyDecoder {
            q_saved: syn,
            q_cur: syn.shift(),
            p_saved: p,
            p_cur: p.shift(),
            deg_saved: 0,
            deg_cur: 1,
        }
    }

    /// Perform the iterative steps to get the error-location polynomial Λ(x) wih deg(Λ)
    /// <= t.
    pub fn decode(mut self) -> Polynomial<P> {
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

    /// Simply shift the polynomials since they have no degree-0 term.
    fn reduce(&mut self) -> (bool, Polynomial<P>, Polynomial<P>, usize) {
        (
            false,
            self.q_cur.shift(),
            self.p_cur.shift(),
            2 + self.deg_cur,
        )
    }

    /// Remove the degree-0 terms and shift the polynomials.
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

/// Uses Chien search to find the roots in GF(2^6) of an error-locator polynomial and
/// produce an iterator of error bit positions. The Forney algorithm is used to find the
/// associated error values.
pub struct Errors<P: PolynomialCoefs> {
    /// Error location polynomial.
    epoly: Polynomial<P>,
    /// Derivative of above.
    deriv: Polynomial<P>,
    /// Error value polynomial.
    vpoly: Polynomial<P>,
    /// Current exponent power of the iteration.
    pow: std::ops::Range<usize>,
}

impl<P: PolynomialCoefs> Errors<P> {
    /// Construct a new `Errors` from the given error and syndrome polynomials.
    pub fn new(mut epoly: Polynomial<P>, syn: Polynomial<P>) -> Errors<P> {
        let deriv = epoly.deriv();
        let vpoly = (epoly * syn).truncate(syn.len() - 2);

        for (pow, cur) in epoly.iter_mut().enumerate() {
            // Since the first call to `update_terms()` multiplies by `pow` and the
            // coefficients should equal themselves on the first iteration, divide by
            // `pow` here.
            *cur = *cur / P25Codeword::for_power(pow)
        }

        Errors {
            epoly: epoly,
            deriv: deriv,
            vpoly: vpoly,
            pow: 0..P25Field::size(),
        }
    }

    /// Perform the term-updating step of the algorithm: x_{j,i} = x_{j,i-1} * α^j.
    fn update_terms(&mut self) {
        for (pow, term) in self.epoly.iter_mut().enumerate() {
            *term = *term * P25Codeword::for_power(pow);
        }
    }

    /// Calculate the sum of the terms: x_{0,i} + x_{1,i} + ... + x_{t,i} -- evaluate the
    /// error-locator polynomial at Λ(α^i).
    fn sum_terms(&self) -> P25Codeword {
        self.epoly.iter().fold(P25Codeword::default(), |s, &x| s + x)
    }

    /// Determine the error value for the given error location/root.
    fn value(&self, loc: P25Codeword, root: P25Codeword) -> P25Codeword {
        self.vpoly.eval(root) / self.deriv.eval(root) * loc
    }
}

impl<P: PolynomialCoefs> Iterator for Errors<P> {
    type Item = (usize, P25Codeword);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let pow = match self.pow.next() {
                Some(pow) => pow,
                None => return None,
            };

            self.update_terms();

            if self.sum_terms().zero() {
                let root = P25Codeword::for_power(pow);
                let loc = root.invert();

                return Some((loc.power().unwrap(), self.value(loc, root)));
            }
        }
    }
}

use std;

const GEN: &'static [u16] = &[
    0b1000000000000000,
    0b0100000000000000,
    0b0010000000000000,
    0b0001000000000000,
    0b0000100000000000,
    0b0000010000000000,
    0b0000001000000000,
    0b0000000100000000,
    0b0000000010000000,
    0b0000000001000000,
    0b0000000000100000,
    0b0000000000010000,
    0b0000000000001000,
    0b0000000000000100,
    0b0000000000000010,
    0b0000000000000001,
    0b1110110001000111,
    0b1001101001100100,
    0b0100110100110010,
    0b0010011010011001,
    0b1111111100001011,
    0b1001001111000010,
    0b0100100111100001,
    0b1100100010110111,
    0b1000100000011100,
    0b0100010000001110,
    0b0010001000000111,
    0b1111110101000100,
    0b0111111010100010,
    0b0011111101010001,
    0b1111001111101111,
    0b1001010110110000,
    0b0100101011011000,
    0b0010010101101100,
    0b0001001010110110,
    0b0000100101011011,
    0b1110100011101010,
    0b0111010001110101,
    0b1101011001111101,
    0b1000011101111001,
    0b1010111111111011,
    0b1011101110111010,
    0b0101110111011101,
    0b1100001010101001,
    0b1000110100010011,
    0b1010101011001110,
    0b0101010101100111,
    0b1100011011110100,
    0b0110001101111010,
    0b0011000110111101,
    0b1111010010011001,
    0b1001011000001011,
    0b1010011101000010,
    0b0101001110100001,
    0b1100010110010111,
    0b1000111010001100,
    0b0100011101000110,
    0b0010001110100011,
    0b1111110110010110,
    0b0111111011001011,
    0b1101001100100010,
    0b0110100110010001,
    0b1101100010001111,
    0b0000000000000011,
];

pub fn encode(word: u16) -> u64 {
    GEN.iter().fold(0, |accum, row| {
        accum << 1 | ((word & row).count_ones() % 2) as u64
    })
}

pub fn decode(mut word: u64) -> Option<(u64, usize)> {
    let syns = syndromes(word);

    // 2t+1 = 23 => t = 11
    for err in (1..12).rev() {
        let matrix = match SyndromeMatrix::new(&syns, err).invert() {
            Some(m) => m,
            None => continue,
        };

        let coefs = ErrorCoefs::new(matrix, err).solve(&syns);
        let locs = ErrorLocations::new(coefs.iter().cloned().rev());

        for loc in locs.take(err) {
            word ^= 1 << loc.power().unwrap();
        }

        return Some((word, err));
    }

    Some((word, 0))
}

// word has r_{n-1} as MSB and r_0 as LSB
fn syndromes(word: u64) -> Vec<Codeword> {
    (1..23).map(|t| {
        (0..63).fold(Codeword::new(0), |s, b| {
            if word >> b & 1 == 0 {
                s
            } else {
                s + Codeword::for_power(b * t)
            }
        })
    }).collect()
}

struct SyndromeMatrix {
    cells: Vec<Vec<Codeword>>,
    rows: usize,
    cols: usize,
}

impl SyndromeMatrix {
    pub fn new(syndromes: &[Codeword], dim: usize) -> SyndromeMatrix {
        assert!(syndromes.len() >= dim * 2 - 2);

        SyndromeMatrix {
            cells: Self::build(syndromes, dim),
            rows: dim,
            cols: dim * 2,
        }
    }

    fn build(syndromes: &[Codeword], dim: usize) -> Vec<Vec<Codeword>> {
        (0..dim).map(|row| {
            (0..dim).map(|col| {
                syndromes[row + col]
            }).chain((0..dim).map(|col| {
                if col == row {
                    Codeword::for_power(0)
                } else {
                    Codeword::new(0)
                }
            })).collect()
        }).collect()
    }

    pub fn invert(mut self) -> Option<Vec<Vec<Codeword>>> {
        let rows = self.rows;
        let cols = self.cols;

        for pos in 0..rows {
            let max = (pos..rows).fold(pos, |max, next| {
                if self.cells[next][pos] > self.cells[max][pos] {
                    next
                } else {
                    max
                }
            });

            if self.cells[max][pos].zero() {
                return None;
            }

            self.cells.swap(pos, max);
            self.reduce((pos+1)..rows, pos);
        }

        for pos in (0..rows).rev() {
            self.reduce(0..pos, pos);
            let div = self.cells[pos][pos];

            for col in 0..cols {
                self.cells[pos][col] = self.cells[pos][col] / div;
            }
        }

        Some(self.cells)
    }

    fn reduce(&mut self, rows: std::ops::Range<usize>, target: usize) {
        for row in rows {
            let ratio = self.cells[row][target] / self.cells[target][target];

            for col in target..self.cols {
                self.cells[row][col] = self.cells[row][col] -
                    self.cells[target][col] * ratio;
            }
        }
    }
}

#[derive(Debug)]
struct ErrorCoefs {
    cells: Vec<Vec<Codeword>>,
    dim: usize,
}

impl ErrorCoefs {
    fn new(cells: Vec<Vec<Codeword>>, dim: usize) -> Self {
        ErrorCoefs {
            cells: cells,
            dim: dim,
        }
    }

    pub fn solve(self, syndromes: &[Codeword]) -> Vec<Codeword> {
        let rhs = &syndromes[self.dim..];

        self.cells.iter().map(|row| {
            row.iter()
                .skip(self.dim)
                .zip(rhs)
                .fold(Codeword::new(0), |s, (&a, &b)| {
                    s + a * b
                })
        }).collect()
    }
}

struct ErrorLocations {
    terms: Vec<Codeword>,
    pow: usize,
}

impl ErrorLocations {
    // Λ(x) = coefs[0]*x + coefs[1]*x^2 + ...
    pub fn new<T: Iterator<Item = Codeword>>(coefs: T) -> ErrorLocations {
        ErrorLocations {
            terms: coefs.enumerate().map(|(p, c)| {
                c / Codeword::for_power(p + 1)
            }).collect(),
            pow: 0,
        }
    }

    fn update_terms(&mut self) {
        for (j, term) in self.terms.iter_mut().enumerate() {
            *term = *term * Codeword::for_power(j + 1);
        }
    }

    fn eval(&self) -> Codeword {
        self.terms.iter().fold(Codeword::for_power(0), |s, &x| {
            s + x

        })
    }
}

impl Iterator for ErrorLocations {
    type Item = Codeword;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pow < POWERS.len() {
            let pow = self.pow;
            self.pow += 1;

            self.update_terms();

            if self.eval().zero() {
                return Some(Codeword::for_power(0) / Codeword::for_power(pow));
            }
        }

        None
    }
}

#[derive(Copy, Clone)]
struct Codeword(u8);

impl std::fmt::Debug for Codeword {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "Codeword({:?})", self.power())
    }
}

impl Codeword {
    pub fn new(codeword: u8) -> Codeword {
        Codeword(codeword)
    }

    pub fn zero(&self) -> bool { self.0 == 0 }

    pub fn power(&self) -> Option<usize> {
        if self.zero() {
            None
        } else {
            Some(POWERS[self.0 as usize - 1])
        }
    }

    fn for_power(power: usize) -> Codeword {
        Codeword::new(CODEWORDS[power % POWERS.len()])
    }
}

impl std::ops::Mul for Codeword {
    type Output = Codeword;

    fn mul(self, rhs: Codeword) -> Self::Output {
        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Codeword::for_power(p + q),
            _ => Codeword::new(0),
        }
    }
}

impl std::ops::Div for Codeword {
    type Output = Codeword;

    fn div(self, rhs: Codeword) -> Self::Output {
        match (self.power(), rhs.power()) {
            // min(power) = -62 => 63+min(power) > 0
            (Some(p), Some(q)) => Codeword::for_power(p + POWERS.len() - q),
            (None, Some(_)) => Codeword::new(0),
            (_, None) => panic!("divide by zero"),
        }
    }
}

impl std::ops::Add for Codeword {
    type Output = Codeword;

    fn add(self, rhs: Codeword) -> Self::Output {
        Codeword::new(self.0 ^ rhs.0)
    }
}

impl std::ops::Sub for Codeword {
    type Output = Codeword;

    fn sub(self, rhs: Codeword) -> Self::Output {
        self + rhs
    }
}

impl std::cmp::PartialEq for Codeword {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::Eq for Codeword {}

impl std::cmp::PartialOrd for Codeword {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;

        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Some(p.cmp(&q)),
            (Some(_), None) => Some(Greater),
            (None, Some(_)) => Some(Less),
            (None, None) => Some(Equal),
        }
    }
}

impl std::cmp::Ord for Codeword {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

// Maps α^i to codewords.
const CODEWORDS: &'static [u8] = &[
    0b100000,
    0b010000,
    0b001000,
    0b000100,
    0b000010,
    0b000001,
    0b110000,
    0b011000,
    0b001100,
    0b000110,
    0b000011,
    0b110001,
    0b101000,
    0b010100,
    0b001010,
    0b000101,
    0b110010,
    0b011001,
    0b111100,
    0b011110,
    0b001111,
    0b110111,
    0b101011,
    0b100101,
    0b100010,
    0b010001,
    0b111000,
    0b011100,
    0b001110,
    0b000111,
    0b110011,
    0b101001,
    0b100100,
    0b010010,
    0b001001,
    0b110100,
    0b011010,
    0b001101,
    0b110110,
    0b011011,
    0b111101,
    0b101110,
    0b010111,
    0b111011,
    0b101101,
    0b100110,
    0b010011,
    0b111001,
    0b101100,
    0b010110,
    0b001011,
    0b110101,
    0b101010,
    0b010101,
    0b111010,
    0b011101,
    0b111110,
    0b011111,
    0b111111,
    0b101111,
    0b100111,
    0b100011,
    0b100001
];

// Maps codewords to α^i.
const POWERS: &'static [usize] = &[
    5,
    4,
    10,
    3,
    15,
    9,
    29,
    2,
    34,
    14,
    50,
    8,
    37,
    28,
    20,
    1,
    25,
    33,
    46,
    13,
    53,
    49,
    42,
    7,
    17,
    36,
    39,
    27,
    55,
    19,
    57,
    0,
    62,
    24,
    61,
    32,
    23,
    45,
    60,
    12,
    31,
    52,
    22,
    48,
    44,
    41,
    59,
    6,
    11,
    16,
    30,
    35,
    51,
    38,
    21,
    26,
    47,
    54,
    43,
    18,
    40,
    56,
    58,
];

#[cfg(test)]
mod test {
    use super::{encode, decode, syndromes, Codeword};

    #[test]
    fn test_for_power() {
        assert_eq!(Codeword::for_power(0).0, 0b100000);
        assert_eq!(Codeword::for_power(62).0, 0b100001);
        assert_eq!(Codeword::for_power(63).0, 0b100000);
    }

    #[test]
    fn test_add_sub() {
        assert_eq!((Codeword::new(0b100000) + Codeword::new(0b010000)).0, 0b110000);
        assert_eq!((Codeword::new(0b100000) - Codeword::new(0b010000)).0, 0b110000);
        assert_eq!((Codeword::new(0b100001) + Codeword::new(0b100001)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) - Codeword::new(0b100001)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) + Codeword::new(0b110100)).0, 0b010101);
        assert_eq!((Codeword::new(0b100001) - Codeword::new(0b110100)).0, 0b010101);
    }

    #[test]
    fn test_mul() {
        assert_eq!((Codeword::new(0b011000) * Codeword::new(0b101000)).0, 0b011110);
        assert_eq!((Codeword::new(0b000000) * Codeword::new(0b101000)).0, 0b000000);
        assert_eq!((Codeword::new(0b011000) * Codeword::new(0b000000)).0, 0b000000);
        assert_eq!((Codeword::new(0b000000) * Codeword::new(0b000000)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) * Codeword::new(0b100000)).0, 0b100001);
        assert_eq!((Codeword::new(0b100001) * Codeword::new(0b010000)).0, 0b100000);
        assert_eq!((Codeword::new(0b110011) * Codeword::new(0b110011)).0, 0b100111);
        assert_eq!((Codeword::new(0b111101) * Codeword::new(0b111101)).0, 0b011001);
    }


    #[test]
    fn test_div() {
        assert_eq!((Codeword::new(0b000100) / Codeword::new(0b101000)).0, 0b111010);
        assert_eq!((Codeword::new(0b000000) / Codeword::new(0b101000)).0, 0b000000);
        assert_eq!((Codeword::new(0b011110) / Codeword::new(0b100000)).0, 0b011110);
        assert_eq!((Codeword::new(0b011110) / Codeword::new(0b011110)).0, 0b100000);
    }

    #[test]
    fn test_cmp() {
        assert!(Codeword::new(0b100000) > Codeword::new(0b000000));
        assert!(Codeword::new(0b000000) == Codeword::new(0b000000));
        assert!(Codeword::new(0b010000) > Codeword::new(0b100000));
        assert!(Codeword::new(0b100001) > Codeword::new(0b100000));
    }

    #[test]
    fn test_encode() {
        assert_eq!(encode(0b1111111100000000), 0b1111111100000000100100110001000011000010001100000110100001101000);
        assert_eq!(encode(0b0011)&1, 0);
        assert_eq!(encode(0b0101)&1, 1);
        assert_eq!(encode(0b1010)&1, 1);
        assert_eq!(encode(0b1100)&1, 0);
        assert_eq!(encode(0b1111)&1, 0);
    }

    #[test]
    fn test_syndromes() {
        let w = encode(0b1111111100000000)>>1;

        assert!(syndromes(w).iter().all(|s| s.zero()));
        assert!(!syndromes(w ^ 1<<60).iter().all(|s| s.zero()));
    }
}

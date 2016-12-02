//! Encoding and decoding for the several error correction coding schemes used in P25.
//!
//! # References
//!
//! 1. *Coding Theory and Cryptography*, Hankerson, et al, 2000.
//! 2. *Error-Control Block Codes for Communication Engineers*, Lee, 2000.
//! 3. ["Implementation of a Reed-Solomon Encoder and Decoder in MATLAB"]
//!    (http://www.ee.iitm.ac.in/~ee11b130/RS_report.pdf), Ramesh.
//! 4. ["Reed Solomon Decoder"](http://www.ti.com/lit/an/spra686/spra686.pdf), Sankaran,
//!    Texas Instruments, 2000.
//! 5. ["Decoding BCH/RS Codes"](http://web.ntpu.edu.tw/~yshan/BCH_decoding.pdf), Han,
//!    National Taipei University.
//! 6. ["A Decoding Procedure for the Reed-Solomon Codes"]
//!    (https://ntrs.nasa.gov/archive/nasa/casi.ntrs.nasa.gov/19780022919.pdf), Lim, NASA
//!    Ames, 1978.
//! 7. ["Reed-Solomon error correction"]
//!    (http://downloads.bbc.co.uk/rd/pubs/whp/ whp-pdf-files/WHP031.pdf), Clarke, BBC,
//!    2002.
//! 8. ["Lecture 18: Decoding of Nonbinary BCH and RS Codes"]
//!     (http://www.site.uottawa.ca/~damours/courses/ELG_5372/Lecture18.pdf), D'Amours,
//!     University of Ottowa.
//! 9. ["EE 387, Notes 19"](http://web.stanford.edu/class/ee387/handouts/notes19.pdf),
//!     Gill, Stanford University.
//! 10. ["EE 387, Notes 20"](http://web.stanford.edu/class/ee387/handouts/notes20.pdf),
//!     Gill, Stanford University.
//! 11. ["Implementing Reed-Solomon"]
//!     (https://www.cs.duke.edu/courses/spring11/cps296.3/decoding_rs.pdf), Brown, Duke
//!     University.
//! 12. "Nonbinary BCH Decoding", Berlekamp, 1966.
//! 13. "Shift-Register Synthesis and BCH Decoding", Massey, 1969.
//! 14. "Cyclic decoding procedure for BCH codes", Chien, 1964.
//! 15. "On decoding BCH codes", Forney, 1965.
//! 16. *Error Control Coding*, Lin and Costello, 1983.

#[macro_use]
mod macros;

#[macro_use]
pub mod galois;

pub mod bch;
pub mod bmcf;
pub mod cyclic;
pub mod golay;
pub mod hamming;
pub mod reed_solomon;
pub mod trellis;

use crate::types::FiniteField;
use rand::{Rng, rng};

impl FiniteField {
    // Create a new finite field GF(2^m) with an irreducible polynomial
    pub fn new(m: u8) -> Self {
        assert!(m > 1 && m <= 16, "Field degree must be between 2 and 16");
        // Choose a standard irreducible polynomial for common field sizes
        let poly: u32 = match m {
            2 => 0b111,       // x^2 + x + 1
            3 => 0b1011,      // x^3 + x + 1
            4 => 0b10011,     // x^4 + x + 1
            5 => 0b100101,    // x^5 + x^2 + 1
            6 => 0b1000011,   // x^6 + x + 1
            7 => 0b10001001,  // x^7 + x^3 + 1
            8 => 0b100011101, // x^8 + x^4 + x^3 + x^2 + 1
            9 => 0x100000b,   // x^9 + x^5 + x^3 + x + 1
            10 => 0x100003,   // x^10 + x^3 + x^2 + 1
            11 => 0x105,      // x^11 + x^2 + 1
            12 => 0x1053,     // x^12 + x^6 + x^4 + x + 1
            13 => 0x201b,     // x^13 + x^9 + x^3 + x + 1
            14 => 0x100b,     // x^14 + x^4 + x^3 + x + 1
            15 => 0x100d,     // x^15 + x^5 + x^3 + x + 1
            16 => 0x1002d,    // x^16 + x^12 + x^3 + x + 1
            _ => panic!("Unsupported field size"),
        };

        FiniteField { m, poly }
    }

    pub fn get_m(&self) -> u8 {
        self.m
    }

    // Addition in GF(2^m) is bitwise XOR
    pub fn field_add(&self, a: u32, b: u32) -> u32 {
        a ^ b
    }

    // Multiplication in GF(2^m)
    pub fn field_multiply(&self, a: u32, b: u32) -> u32 {
        if a == 0 || b == 0 {
            return 0;
        }

        let mut result = 0u32;
        let mut a_temp = a;
        let mut b_temp = b;

        while b_temp > 0 {
            if b_temp & 1 == 1 {
                result ^= a_temp;
            }

            // Check if the leading bit will be shifted out
            let highest_bit_set = a_temp & (1 << self.m) != 0;

            a_temp <<= 1;

            if highest_bit_set {
                a_temp ^= self.poly;
            }

            a_temp &= (1 << (self.m + 1)) - 1; // Keep only relevant bits
            b_temp >>= 1;
        }

        result
    }

    // Helper functions for bit-level field operations
    fn bit_polynomial_multiply(&self, a: u32, b: u32) -> u32 {
        let mut result = 0u32;
        let mut b_temp = b;
        let mut i = 0;

        while b_temp > 0 {
            if b_temp & 1 != 0 {
                result ^= a << i;
            }
            b_temp >>= 1;
            i += 1;
        }

        result
    }

    fn bit_polynomial_divide(&self, a: u32, b: u32) -> u32 {
        let b_deg = 31 - b.leading_zeros() as u8;
        let a_deg = 31 - a.leading_zeros() as u8;

        if a_deg < b_deg {
            return 0;
        }

        let mut result = 0u32;
        let mut tmp = a;

        for i in (0..=(a_deg - b_deg)).rev() {
            if (tmp & (1 << (b_deg + i))) != 0 {
                result |= 1 << i;
                tmp ^= b << i;
            }
        }

        result
    }

    fn bit_polynomial_mod(&self, a: u32, b: u32) -> u32 {
        let b_deg = 31 - b.leading_zeros() as u8;
        let mut tmp = a;

        if b_deg == 0 {
            return 0;
        }

        while tmp >= b {
            let tmp_deg = 31 - tmp.leading_zeros() as u8;
            if tmp_deg < b_deg {
                break;
            }
            tmp ^= b << (tmp_deg - b_deg);
        }

        tmp
    }

    // Find the multiplicative inverse of an element in GF(2^m)
    pub fn inverse(&self, a: u32) -> u32 {
        assert!(a != 0, "Cannot invert zero");

        // Using Extended Euclidean Algorithm for GF(2^m)
        let mut r0 = self.poly;
        let mut r1 = a;
        let mut s0 = 1u32;
        let mut s1 = 0u32;
        let mut t0 = 0u32;
        let mut t1 = 1u32;

        while r1 != 0 {
            let q = self.bit_polynomial_divide(r0, r1);
            let r2 = self.bit_polynomial_mod(r0, r1);
            let s2 = s0 ^ self.bit_polynomial_multiply(q, s1);
            let t2 = t0 ^ self.bit_polynomial_multiply(q, t1);

            r0 = r1;
            r1 = r2;
            s0 = s1;
            s1 = s2;
            t0 = t1;
            t1 = t2;
        }

        t0
    }
}

//-------------------------------------------------------------
// Polynomial operations over finite fields
//-------------------------------------------------------------

/// Trims leading zeros from a polynomial
pub fn trim_polynomial(poly: &mut Vec<u32>) {
    while poly.len() > 1 && poly[poly.len() - 1] == 0 {
        poly.pop();
    }
}

/// Evaluate a polynomial at a point in GF(2^m)
pub fn evaluate_poly(poly: &[u32], x: u32, field: &FiniteField) -> u32 {
    if x == 0 {
        return poly.first().copied().unwrap_or(0);
    }

    let mut result = *poly.last().unwrap_or(&0);
    for i in (0..poly.len() - 1).rev() {
        result = field.field_add(field.field_multiply(result, x), poly[i]);
    }

    result
}

/// Generate a random irreducible polynomial of degree t
pub fn random_irreducible_poly(t: usize, field: &FiniteField) -> Vec<u32> {
    let mut rng = rng();

    // Create a monic polynomial (highest coefficient is 1)
    let mut poly = vec![0u32; t + 1];
    poly[t] = 1; // Make it monic

    // Generate random coefficients for the other terms
    for coefficient in poly.iter_mut().take(t) {
        *coefficient = rng.random_range(0..(1 << field.get_m())) as u32;
    }

    // Ensure the constant term is non-zero for irreducibility
    if poly[0] == 0 {
        poly[0] = 1;
    }

    poly
}

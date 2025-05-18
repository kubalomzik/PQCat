use crate::types::FiniteField;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::min;

impl FiniteField {
    // Create a new finite field GF(2^m) with an irreducible polynomial
    pub fn new(m: u8) -> Self {
        assert!(m > 1 && m <= 8, "Field degree must be between 2 and 8");
        // Choose a standard irreducible polynomial for common field sizes
        let poly = match m {
            2 => 0b111,       // x^2 + x + 1
            3 => 0b1011,      // x^3 + x + 1
            4 => 0b10011,     // x^4 + x + 1
            5 => 0b100101,    // x^5 + x^2 + 1
            6 => 0b1000011,   // x^6 + x + 1
            7 => 0b10001001,  // x^7 + x^3 + 1
            8 => 0b100011101, // x^8 + x^4 + x^3 + x^2 + 1
            _ => panic!("Unsupported field size"),
        };

        FiniteField { m, poly }
    }

    pub fn get_m(&self) -> u8 {
        self.m
    }

    // Addition in GF(2^m) is bitwise XOR
    pub fn field_add(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    // Multiplication in GF(2^m)
    pub fn field_multiply(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }

        let mut result = 0u16;
        let mut a_temp = a as u16;
        let mut b_temp = b as u16;

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

        result as u8
    }

    // Helper functions for bit-level field operations
    fn bit_polynomial_multiply(&self, a: u16, b: u16) -> u16 {
        let mut result = 0u16;
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

    fn bit_polynomial_divide(&self, a: u16, b: u16) -> u16 {
        let b_deg = 31 - b.leading_zeros() as u8;
        let a_deg = 31 - a.leading_zeros() as u8;

        if a_deg < b_deg {
            return 0;
        }

        let mut result = 0u16;
        let mut tmp = a;

        for i in (0..=(a_deg - b_deg)).rev() {
            if (tmp & (1 << (b_deg + i))) != 0 {
                result |= 1 << i;
                tmp ^= b << i;
            }
        }

        result
    }

    fn bit_polynomial_mod(&self, a: u16, b: u16) -> u16 {
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
    pub fn inverse(&self, a: u8) -> u8 {
        assert!(a != 0, "Cannot invert zero");

        // Using Extended Euclidean Algorithm for GF(2^m)
        let mut r0 = self.poly;
        let mut r1 = a as u16;
        let mut s0 = 1u16;
        let mut s1 = 0u16;
        let mut t0 = 0u16;
        let mut t1 = 1u16;

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

        t0 as u8
    }

    // Generate a set of distinct field elements
    pub fn random_support(&self, size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let field_size = 1 << self.get_m();

        // Make sure we don't request more elements than exist in the field
        let real_size = min(size, field_size - 1);

        // Generate all non-zero field elements
        let mut all_elements: Vec<u8> = (1..field_size as u8).collect();
        all_elements.shuffle(&mut rng);

        // Take the first 'real_size' elements
        all_elements[0..real_size].to_vec()
    }
}

//-------------------------------------------------------------
// Polynomial operations over finite fields
//-------------------------------------------------------------

/// Trims leading zeros from a polynomial
pub fn trim_polynomial(poly: &mut Vec<u8>) {
    while poly.len() > 1 && poly[poly.len() - 1] == 0 {
        poly.pop();
    }
}

/// Polynomial addition in GF(2^m)
pub fn polynomial_add(a: &mut Vec<u8>, b: &[u8]) {
    // Ensure a is at least as long as b
    if a.len() < b.len() {
        a.resize(b.len(), 0);
    }

    // Add coefficients (XOR in binary field)
    for i in 0..b.len() {
        a[i] ^= b[i];
    }

    // Remove leading zeros
    trim_polynomial(a);
}

/// Polynomial multiplication in GF(2^m)
pub fn polynomial_multiply(a: &[u8], b: &[u8], field: &FiniteField) -> Vec<u8> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }

    let deg_a = a.len() - 1;
    let deg_b = b.len() - 1;
    let deg_result = deg_a + deg_b;
    let mut result = vec![0; deg_result + 1];

    for i in 0..=deg_a {
        for j in 0..=deg_b {
            let product = field.field_multiply(a[i], b[j]);
            result[i + j] ^= product; // XOR since we're in GF(2)
        }
    }

    // Remove leading zeros
    let mut result_mut = result;
    trim_polynomial(&mut result_mut);

    result_mut
}

/// Polynomial division in GF(2^m)
pub fn polynomial_divide(a: &[u8], b: &[u8], field: &FiniteField) -> (Vec<u8>, Vec<u8>) {
    if b.is_empty() || b.iter().all(|&x| x == 0) {
        panic!("Division by zero polynomial");
    }

    // Ensure the leading coefficient is non-zero
    let mut b_copy = b.to_vec();
    trim_polynomial(&mut b_copy);

    let mut a_copy = a.to_vec();
    let deg_a = a_copy.len() - 1;
    let deg_b = b_copy.len() - 1;

    if deg_a < deg_b {
        return (vec![], a_copy);
    }

    let mut quotient = vec![0; deg_a - deg_b + 1];
    let b_leading = b_copy[deg_b];
    let b_leading_inv = field.inverse(b_leading);

    for i in (0..=deg_a - deg_b).rev() {
        if a_copy.len() <= i + deg_b {
            continue;
        }

        let coef = field.field_multiply(a_copy[i + deg_b], b_leading_inv);
        quotient[i] = coef;

        for j in 0..=deg_b {
            if j + i < a_copy.len() {
                a_copy[j + i] ^= field.field_multiply(coef, b_copy[j]);
            }
        }
    }

    // Truncate remainder to proper degree
    trim_polynomial(&mut a_copy);

    (quotient, a_copy)
}

/// Polynomial modulo operation in GF(2^m)
pub fn polynomial_mod(a: &mut Vec<u8>, m: &[u8], field: &FiniteField) {
    if a.len() < m.len() {
        return;
    }

    let (_, r) = polynomial_divide(a, m, field);
    *a = r;
}

/// Evaluate a polynomial at a point in GF(2^m)
pub fn evaluate_poly(poly: &[u8], x: u8, field: &FiniteField) -> u8 {
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
pub fn random_irreducible_poly(t: usize, field: &FiniteField) -> Vec<u8> {
    let mut rng = rand::thread_rng();

    // Create a monic polynomial (highest coefficient is 1)
    let mut poly = vec![0u8; t + 1];
    poly[t] = 1; // Make it monic

    // Generate random coefficients for the other terms
    for coefficient in poly.iter_mut().take(t) {
        *coefficient = rng.gen_range(0..(1 << field.get_m())) as u8;
    }

    // Ensure the constant term is non-zero for irreducibility
    if poly[0] == 0 {
        poly[0] = 1;
    }

    poly
}

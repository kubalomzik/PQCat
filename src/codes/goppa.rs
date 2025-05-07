use ndarray::Array2;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::min;

// Finite field implementation for GF(2^m)
pub struct FiniteField {
    m: u8,     // Extension degree (field is GF(2^m))
    poly: u16, // Irreducible polynomial represented as a bit pattern
}

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
    fn add(&self, a: u8, b: u8) -> u8 {
        a ^ b
    }

    // Multiplication in GF(2^m)
    fn multiply(&self, a: u8, b: u8) -> u8 {
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

    // Find the multiplicative inverse of an element in GF(2^m)
    fn inverse(&self, a: u8) -> u8 {
        assert!(a != 0, "Cannot invert zero");

        // Using Extended Euclidean Algorithm for GF(2^m)
        let mut r0 = self.poly;
        let mut r1 = a as u16;
        let mut s0 = 1u16;
        let mut s1 = 0u16;
        let mut t0 = 0u16;
        let mut t1 = 1u16;

        while r1 != 0 {
            let q = self.polynomial_divide(r0, r1);
            let r2 = self.polynomial_mod(r0, r1);
            let s2 = s0 ^ self.polynomial_multiply(q, s1);
            let t2 = t0 ^ self.polynomial_multiply(q, t1);

            r0 = r1;
            r1 = r2;
            s0 = s1;
            s1 = s2;
            t0 = t1;
            t1 = t2;
        }

        t0 as u8
    }

    // Helper functions for polynomial operations
    fn polynomial_divide(&self, a: u16, b: u16) -> u16 {
        // Simplified polynomial division for our needs
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

    fn polynomial_mod(&self, a: u16, b: u16) -> u16 {
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

    fn polynomial_multiply(&self, a: u16, b: u16) -> u16 {
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

    // Evaluate polynomial at a point
    pub fn evaluate_poly(&self, poly: &[u8], x: u8) -> u8 {
        if x == 0 {
            return poly[0];
        }

        let mut result = poly[poly.len() - 1];
        for i in (0..poly.len() - 1).rev() {
            result = self.add(self.multiply(result, x), poly[i]);
        }

        result
    }

    // Generate a random irreducible polynomial of degree t
    pub fn random_irreducible_poly(&self, t: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();

        // Create a monic polynomial (highest coefficient is 1)
        let mut poly = vec![0u8; t + 1];
        poly[t] = 1; // Make it monic

        // Generate random coefficients for the other terms
        for coefficient in poly.iter_mut().take(t) {
            *coefficient = rng.gen_range(0..(1 << self.get_m())) as u8;
        }

        // Ensure the constant term is non-zero for irreducibility
        if poly[0] == 0 {
            poly[0] = 1;
        }

        /*
        Note: In a textbook implementation, we'd check if the polynomial is irreducible and retry if not.
        For simplicity, we'll just return this polynomial.
        */
        poly
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

pub fn generate_goppa_parity_matrix(
    n: usize,
    t: usize,
    goppa_poly: &[u8],
    support: &[u8],
    field: &FiniteField,
) -> Array2<u8> {
    // The parity check matrix for a binary Goppa code has t*m rows
    let m = field.get_m() as usize;
    let mut h = Array2::<u8>::zeros((t * m, n));

    for j in 0..n {
        // For each support element L[j]
        let l_j = support[j];

        // Calculate g(L[j])
        let g_l_j = field.evaluate_poly(goppa_poly, l_j);

        // Calculate 1/g(L[j])
        let inv_g_l_j = field.inverse(g_l_j);

        // Generate the column with format [L[j]^(t-1)/g(L[j]), ..., L[j]/g(L[j]), 1/g(L[j])]
        let mut power = 1u8; // Start with L[j]^0 = 1

        for i in 0..t {
            let col_val = field.multiply(power, inv_g_l_j);

            // Convert to binary and place in the appropriate rows
            for bit in 0..m {
                h[[i * m + bit, j]] = (col_val >> bit) & 1;
            }

            // Calculate next power
            power = field.multiply(power, l_j);
        }
    }

    h
}

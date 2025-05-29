use crate::codes::polynomial_utils::{evaluate_poly, random_irreducible_poly};
use crate::types::FiniteField;
use ndarray::Array2;
use rand::rng;
use rand::seq::SliceRandom;

pub fn generate_valid_goppa_params(n: usize, t: usize) -> (Vec<u8>, Vec<u8>, FiniteField) {
    let m = (n as f64).log2().ceil() as u8;
    let field = FiniteField::new(m);

    // Check if n is too close to maximum support size
    let max_support_size = (1 << m) - 1; // 2^m - 1 (excluding zero)

    // Special case: For t=1, a full support is impossible (linear polynomial must have 1 root)
    if t == 1 && n == max_support_size {
        return generate_valid_goppa_params(max_support_size - 1, t);
    }

    if n > max_support_size {
        return generate_valid_goppa_params(max_support_size, t);
    }

    // For safety with nearly-full support (like n=63 in GF(2^6)), ensure our Goppa polynomial has minimal roots in the field
    let mut best_poly = Vec::new();
    let mut min_roots = max_support_size;

    // Try more polynomials for larger field sizes or when we need a nearly-full support
    let attempts = if n > max_support_size - 10 { 20 } else { 10 }; // Increased attempts

    // Try multiple polynomials and choose the one with fewest roots
    for _ in 0..attempts {
        let poly = random_irreducible_poly(t, &field);
        let mut root_count = 0;

        for x in 1..(1 << m) {
            if evaluate_poly(&poly, x as u8, &field) == 0 {
                root_count += 1;
            }
        }

        if root_count < min_roots {
            min_roots = root_count;
            best_poly = poly.clone();

            // If we found a polynomial with few enough roots, use it
            if root_count <= max_support_size - n {
                break;
            }
        }
    }

    // If our best polynomial still has too many roots to create a support of size n
    if min_roots > max_support_size - n {
        let adjusted_n = max_support_size - min_roots;
        // If we can't create a support of reasonable size, try a different t value
        if adjusted_n < n / 2 && t > 1 {
            return generate_valid_goppa_params(n, t - 1);
        }

        return generate_valid_goppa_params(adjusted_n, t);
    }

    // Identify all non-roots to build our support from
    let mut non_roots = Vec::with_capacity(max_support_size);
    for x in 1..(1 << m) {
        let x_byte = x as u8;
        if evaluate_poly(&best_poly, x_byte, &field) != 0 {
            non_roots.push(x_byte);
        }
    }

    if non_roots.len() < n {
        // Instead of panicking, adjust n to the number of non-roots we found
        return generate_valid_goppa_params(non_roots.len(), t);
    }

    // Shuffle the non-roots to get a random support
    let mut rng = rng();
    non_roots.shuffle(&mut rng);

    // Take the first n elements as our support
    let valid_support = non_roots[0..n].to_vec();

    (best_poly, valid_support, field)
}

pub fn generate_goppa_parity_matrix(
    n: usize,
    t: usize,
    goppa_poly: &[u8],
    support: &[u8],
    field: &FiniteField,
) -> Array2<u8> {
    // Verify that support has enough elements
    if support.len() < n {
        panic!(
            "Support vector too small: has {} elements but need {}",
            support.len(),
            n
        );
    }

    // The parity check matrix for a binary Goppa code has t*m rows
    let m = field.get_m() as usize;
    let mut h = Array2::<u8>::zeros((t * m, n));

    for j in 0..n {
        // For each support element L[j]
        let l_j = support[j]; // This is safe now that we check support.len() >= n

        // Calculate g(L[j])
        let g_l_j = evaluate_poly(goppa_poly, l_j, field);

        // Ensure g(L[j]) is not zero
        if g_l_j == 0 {
            panic!("Invalid support: g(L[{}])=0", j);
        }

        // Calculate 1/g(L[j])
        let inv_g_l_j = field.inverse(g_l_j);

        // Generate the column
        let mut power = 1u8; // Start with L[j]^0 = 1

        for i in 0..t {
            let col_val = field.field_multiply(power, inv_g_l_j);

            // Convert to binary and place in the appropriate rows
            for bit in 0..m {
                h[[i * m + bit, j]] = (col_val >> bit) & 1;
            }

            // Calculate next power
            power = field.field_multiply(power, l_j);
        }
    }

    h
}
